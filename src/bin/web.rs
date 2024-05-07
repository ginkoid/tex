use anyhow::{bail, Error, Result};
use axum::{
    body,
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use std::{collections::VecDeque, env, sync};
use tex::proto;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::Mutex,
    task::{self, JoinHandle},
    time,
};

#[tokio::main]
async fn main() -> Result<()> {
    let render_endpoint =
        env::var("RENDER_ENDPOINT").unwrap_or_else(|_| "localhost:5000".to_string());
    let pool_size = env::var("POOL_SIZE").map_or(2, |p| {
        p.parse::<usize>().expect("POOL_SIZE should be a number")
    });
    let app = sync::Arc::new(App {
        pool: RenderPool::new(pool_size, render_endpoint.as_str()).await?,
    });
    let router = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/render", post(render_post))
        .route("/render/:tex", get(render_get))
        .with_state(app);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, router).await?;
    Ok(())
}

struct App {
    pool: RenderPool,
}

async fn render_response(
    pool: &RenderPool,
    tex: body::Bytes,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match pool.render(tex).await {
        Ok(bytes) => Ok(Response::builder()
            .header(header::CONTENT_TYPE, "image/png")
            .body::<body::Body>(bytes.into())
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

async fn render_post(
    State(app): State<sync::Arc<App>>,
    body: body::Bytes,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    render_response(&app.pool, body).await
}

async fn render_get(
    State(app): State<sync::Arc<App>>,
    Path(tex): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    render_response(&app.pool, body::Bytes::from(tex)).await
}

#[derive(thiserror::Error, Debug)]
enum RenderError {
    #[error("tex: {0}")]
    Tex(String),
    #[error("timeout")]
    Timeout,
}

struct RenderPool {
    streams: Mutex<VecDeque<task::JoinHandle<Result<TcpStream>>>>,
    size: usize,
    addr: String,
}

impl RenderPool {
    async fn new(size: usize, addr: &str) -> Result<Self> {
        let pool = Self {
            streams: Mutex::new(VecDeque::with_capacity(size)),
            size,
            addr: addr.to_string(),
        };
        for _ in 0..size {
            pool.connect().await;
        }
        Ok(pool)
    }

    async fn connect(&self) {
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
        self.streams.lock().await.push_back(handle);
    }

    async fn get_response(
        handle: JoinHandle<Result<TcpStream, Error>>,
        content: &mut body::Bytes,
    ) -> Result<proto::Response, Error> {
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
    }

    async fn render(&self, mut content: body::Bytes) -> Result<body::Bytes> {
        for _ in 0..self.size + 1 {
            self.connect().await;
            let handle = self.streams.lock().await.pop_front().unwrap();

            let response = match time::timeout(
                time::Duration::from_secs(5),
                RenderPool::get_response(handle, &mut content),
            )
            .await
            {
                Ok(Ok(response)) => response,
                Ok(Err(_)) => continue,
                Err(_) => return Err(RenderError::Timeout.into()),
            };

            return match response.code {
                proto::Code::Ok => Ok(response.data.into()),
                proto::Code::ErrTex => {
                    Err(RenderError::Tex(String::from_utf8(response.data)?).into())
                }
                _ => bail!("internal error: {:?}", response.code),
            };
        }
        bail!("too many tries")
    }
}
