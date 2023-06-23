use anyhow::{bail, Error, Result};
use async_recursion::async_recursion;
use axum::{
    body,
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::Response,
    routing::get,
    Router,
};
use base64::{prelude::BASE64_URL_SAFE_NO_PAD, Engine};
use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha2::Sha256;
use std::{collections::VecDeque, env, net::SocketAddr, ops::Deref, sync};
use tex::proto;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    task, time,
};

#[tokio::main]
async fn main() {
    let render_endpoint = env::var("RENDER_ENDPOINT").unwrap_or("localhost:5000".to_string());
    let priority_pool_size = env::var("PRIORITY_POOL_SIZE")
        .unwrap_or("3".to_string())
        .parse::<usize>()
        .unwrap();
    let public_pool_size = env::var("PUBLIC_POOL_SIZE")
        .unwrap_or("3".to_string())
        .parse::<usize>()
        .unwrap();
    let hmac_key = BASE64_URL_SAFE_NO_PAD
        .decode(env::var("HMAC_KEY").unwrap())
        .unwrap();
    let app = sync::Arc::new(App {
        priority_pool: RenderPool::new(priority_pool_size, render_endpoint.as_str())
            .await
            .unwrap(),
        public_pool: RenderPool::new(public_pool_size, render_endpoint.as_str())
            .await
            .unwrap(),
        hmac_key,
    });
    serve(
        Router::new()
            .route("/health", get(|| async { "ok" }))
            .route("/render/:tex", get(render))
            .with_state(app),
        3000,
    )
    .await
}

struct App {
    priority_pool: RenderPool,
    public_pool: RenderPool,
    hmac_key: Vec<u8>,
}

#[derive(Deserialize)]
struct RenderQuery {
    token: Option<String>,
}

fn verify_hmac(hmac_key: &Vec<u8>, token: &String, tex: body::Bytes) -> Result<()> {
    let mut hmac = Hmac::<Sha256>::new_from_slice(hmac_key.as_slice())?;
    hmac.update(tex.deref());
    Ok(hmac.verify_slice(BASE64_URL_SAFE_NO_PAD.decode(token.as_bytes())?.as_slice())?)
}

async fn render(
    State(app): State<sync::Arc<App>>,
    Path(tex): Path<String>,
    query: Query<RenderQuery>,
) -> Result<Response<body::Full<body::Bytes>>, (StatusCode, String)> {
    let tex = body::Bytes::from(tex);
    let pool = if let Some(token) = &query.token {
        if let Err(_) = verify_hmac(&app.hmac_key, &token, tex.clone()) {
            return Err((StatusCode::FORBIDDEN, "".to_string()));
        }
        &app.priority_pool
    } else {
        &app.public_pool
    };
    match pool.render(tex).await {
        Ok(bytes) => Ok(Response::builder()
            .header(header::CONTENT_TYPE, "image/png")
            .body(bytes.into())
            .unwrap()),
        Err(err) => match err.downcast::<RenderError>() {
            Ok(RenderError::Tex(err)) => Err((StatusCode::BAD_REQUEST, err)),
            Ok(RenderError::Timeout) => Err((StatusCode::BAD_REQUEST, "timeout".to_string())),
            Err(err) => {
                eprintln!("render: {}", err);
                Err((StatusCode::INTERNAL_SERVER_ERROR, "".to_string()))
            }
        },
    }
}

async fn serve(router: Router, port: u16) {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("listening {}", addr);
    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await
        .unwrap();
}

#[derive(thiserror::Error, Debug)]
enum RenderError {
    #[error("tex: {0}")]
    Tex(String),
    #[error("timeout")]
    Timeout,
}

struct RenderPool {
    streams: sync::Mutex<VecDeque<task::JoinHandle<Result<TcpStream>>>>,
    size: usize,
    addr: String,
}

impl RenderPool {
    async fn new(size: usize, addr: &str) -> Result<Self> {
        let pool = Self {
            streams: sync::Mutex::new(VecDeque::with_capacity(size)),
            size,
            addr: addr.to_string(),
        };
        for _ in 0..size {
            pool.connect();
        }
        Ok(pool)
    }

    fn connect(&self) {
        let addr = self.addr.clone();
        let handle = task::spawn(async move {
            for _ in 0..3 {
                let Ok(mut stream) = TcpStream::connect(addr.clone()).await else {
                    time::sleep(time::Duration::from_secs(1)).await;
                    continue;
                };
                stream.write_all(b"\\begin{document}\n").await?;
                return Ok(stream);
            }
            bail!("failed to connect");
        });
        self.streams.lock().unwrap().push_back(handle);
    }

    #[async_recursion]
    async fn do_render(&self, content: body::Bytes, tries: usize) -> Result<body::Bytes> {
        if tries > self.size {
            bail!("too many tries");
        }

        self.connect();
        let handle = self.streams.lock().unwrap().pop_front().unwrap();
        let response = match time::timeout(time::Duration::from_secs(5), async {
            let mut stream = handle.await??;
            stream.write_all_buf(&mut content.clone()).await?;
            stream.write_all(b"\n\\end{document}\n").await?;

            let code = stream.read_u32().await?;
            let mut data = vec![0; stream.read_u32().await? as usize];
            stream.read_exact(&mut data[..]).await?;
            Ok::<proto::Response, Error>(proto::Response {
                code: code.try_into()?,
                data,
            })
        })
        .await
        {
            Ok(Ok(response)) => response,
            Ok(Err(_)) => return self.do_render(content, tries + 1).await,
            Err(_) => return Err(RenderError::Timeout.into()),
        };
        match response.code {
            proto::Code::Ok => Ok(response.data.into()),
            proto::Code::ErrTex => Err(RenderError::Tex(String::from_utf8(response.data)?).into()),
            _ => bail!("internal error: {:?}", response.code),
        }
    }

    async fn render(&self, content: body::Bytes) -> Result<body::Bytes> {
        self.do_render(content, 0).await
    }
}
