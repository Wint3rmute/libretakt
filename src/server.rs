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
    while let Some(msg) = socket.recv().await {
        let msg = if let Ok(msg) = msg {
            msg
        } else {
            // client disconnected
            return;
        };

        if socket.send(msg).await.is_err() {
            // client disconnected
            return;
        }
    }
}

#[tokio::main]
pub async fn main() {
    // use libretakt::persistence::{load_project, save_project};
    use env_logger::Env;
    use std::io::Write;
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .format(|buf, record| {
            let level_style = buf.default_level_style(record.level());
            writeln!(
                buf,
                "{ts} {level_style}{level}{level_style:#} {file}:{line} - {msg}",
                ts = buf.timestamp(),
                level = record.level(),
                file = record.file().unwrap_or("unknown"),
                line = record.line().unwrap_or(0),
                msg = record.args(),
            )
        })
        .init();

    //***SAMPLER***
    // let tracks = load_project();

    //To be honest i haven't been looking at this code yet but Bączek wrote it
    //so i guess its something important and i trust him 👉👈.
    // let provider = Arc::new(SampleProvider::default());

    // build our application with a single route
    log::info!("Building app & router");
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/ws", any(websocket_handler));

    log::info!("Binding listener...");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    log::info!("Application running!");
    axum::serve(listener, app).await.unwrap();
}
