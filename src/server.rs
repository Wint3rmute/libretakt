use axum::{
    extract::ws::{WebSocket, WebSocketUpgrade},
    response::Response,
    routing::{any, get},
    Router,
};

async fn websocket_handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    use axum::extract::ws::Message;
    use tokio::time::{sleep, Duration};

    tracing::info!("New WebSocket connection");

    loop {
        tokio::select! {
            msg = socket.recv() => {
                match msg {
                    Some(Ok(msg)) => {
                        tracing::debug!("WebSocket message: {:?}", msg);
                        match msg {
                            Message::Text(ref text) => {
                                tracing::info!("Text message: {:?}", text);
                            },
                            Message::Binary(ref data) => {
                                tracing::warn!("Unexpected binary message: {:?}", data);
                            },
                            Message::Ping(ref data) => {
                                tracing::warn!("Unexpected ping: {:?}", data);
                            },
                            Message::Pong(_) => {
                                tracing::debug!("Pong from client");
                            },
                            Message::Close(_) => {
                                tracing::info!("Client closed connection");
                                return
                            },
                        }
                        if socket.send(msg).await.is_err() {
                            tracing::warn!("WebSocket connection closed by client");
                            return;
                        }
                    }
                    Some(Err(_)) | None => {
                        tracing::warn!("WebSocket connection closed by client");
                        return;
                    }
                }
            }
            _ = sleep(Duration::from_millis(5000)) => {
                tracing::debug!("No message received for 5000ms, sending ping");
                if socket.send(Message::Ping(vec![].into())).await.is_err() {
                    tracing::warn!("WebSocket connection closed while sending ping");
                    return;
                }
            }
        }
    }
}

#[tokio::main]
pub async fn main() {
    tracing_subscriber::fmt()
        .with_file(true)
        .with_line_number(true)
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(tracing::Level::INFO.into())
                .from_env_lossy(),
        )
        .init();

    //***SAMPLER***
    // let tracks = load_project();

    //To be honest i haven't been looking at this code yet but Bączek wrote it
    //so i guess its something important and i trust him 👉👈.
    // let provider = Arc::new(SampleProvider::default());

    // build our application with a single route
    tracing::info!("Building app & router");
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/ws", any(websocket_handler));

    const BIND_ADDR: &str = "0.0.0.0:3000";
    tracing::info!("Binding listener to {}...", BIND_ADDR);
    let listener = tokio::net::TcpListener::bind(BIND_ADDR).await.unwrap();
    tracing::info!("Application running at {}", BIND_ADDR);
    axum::serve(listener, app).await.unwrap();
}
