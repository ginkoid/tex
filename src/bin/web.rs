use anyhow::{bail, Result};
use async_recursion::async_recursion;
use axum::{
    body,
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use base64::{prelude::BASE64_URL_SAFE_NO_PAD, Engine};
use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha2::Sha256;
use std::{collections::VecDeque, env, mem::size_of, net::SocketAddr, ops::Deref, sync};
use tex::proto::Code;
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
) -> impl IntoResponse {
    let tex = body::Bytes::from(tex);
    let pool = if let Some(token) = &query.token {
        if let Err(_) = verify_hmac(&app.hmac_key, &token, tex.clone()) {
            return make_response(StatusCode::UNAUTHORIZED, "");
        }
        &app.priority_pool
    } else {
        &app.public_pool
    };
    match pool.render(tex).await {
        Ok(bytes) => Response::builder()
            .header(header::CONTENT_TYPE, "image/png")
            .body(bytes.into())
            .unwrap(),
        Err(err) => match err.downcast::<RenderError>() {
            Ok(RenderError::Tex(err)) => make_response(StatusCode::BAD_REQUEST, err),
            Ok(RenderError::Timeout) => make_response(StatusCode::BAD_REQUEST, "timeout"),
            Err(err) => {
                eprintln!("{}", err);
                make_response(StatusCode::INTERNAL_SERVER_ERROR, "")
            }
        },
    }
}

fn make_response(
    status: StatusCode,
    content: impl Into<body::Bytes>,
) -> Response<body::Full<body::Bytes>> {
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "text/plain")
        .body(body::Full::from(content.into()))
        .unwrap()
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
        let mut stream = handle.await??;
        stream.write_all_buf(&mut content.clone()).await?;
        stream.write_all(b"\n\\end{document}\n").await?;

        let mut buf = Vec::new();
        match time::timeout(time::Duration::from_secs(5), stream.read_to_end(&mut buf)).await {
            Ok(v) => v,
            Err(_) => return Err(RenderError::Timeout.into()),
        }?;

        if buf.len() < size_of::<u32>() {
            return self.do_render(content, tries + 1).await;
        }
        let code = u32::from_be_bytes(
            buf.split_off(buf.len() - size_of::<u32>())
                .try_into()
                .unwrap(),
        );

        if code == Code::ErrTex as u32 {
            return Err(RenderError::Tex(String::from_utf8(buf)?).into());
        }
        if code != Code::Ok as u32 {
            bail!("invalid response {}", code);
        }
        Ok(buf.into())
    }

    async fn render(&self, content: body::Bytes) -> Result<body::Bytes> {
        self.do_render(content, 0).await
    }
}
