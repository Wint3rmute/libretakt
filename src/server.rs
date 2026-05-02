use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
    routing::{any, get},
    Router,
};
use libretakt::shared::{ClientCommand, ClientId, SequencerState, ServerMessage};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use tokio::sync::{broadcast, Mutex};

#[derive(Clone)]
struct AppState {
    sequencer: Arc<Mutex<SequencerState>>,
    broadcast: broadcast::Sender<ServerMessage>,
}

static NEXT_CLIENT_ID: AtomicU64 = AtomicU64::new(1);

fn to_ws_text(msg: &ServerMessage) -> Message {
    Message::Text(
        serde_json::to_string(msg)
            .expect("serialization failed")
            .into(),
    )
}

async fn websocket_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    let client_id = NEXT_CLIENT_ID.fetch_add(1, Ordering::Relaxed);
    tracing::info!(client_id, "New WebSocket connection");

    // Acquire lock, snapshot state, release lock — no await while holding the guard.
    let init_msg = {
        let seq = state.sequencer.lock().await;
        let snapshot = seq.clone();
        ServerMessage::Init {
            client_id,
            state: snapshot,
        }
        // seq dropped here, before any await
    };

    if socket.send(to_ws_text(&init_msg)).await.is_err() {
        tracing::warn!(client_id, "Failed to send Init message, closing");
        return;
    }

    let mut rx = state.broadcast.subscribe();

    loop {
        tokio::select! {
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        match serde_json::from_str::<ClientCommand>(&text) {
                            Ok(cmd) => {
                                handle_command(cmd, client_id, &state, &mut socket).await;
                            }
                            Err(e) => {
                                tracing::warn!(client_id, error = %e, "Failed to parse ClientCommand");
                            }
                        }
                    }
                    Some(Ok(Message::Ping(data))) => {
                        if socket.send(Message::Pong(data)).await.is_err() {
                            tracing::warn!(client_id, "Failed to send Pong, closing");
                            break;
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        tracing::info!(client_id, "Client disconnected");
                        break;
                    }
                    Some(Err(e)) => {
                        tracing::warn!(client_id, error = %e, "WebSocket error, closing");
                        break;
                    }
                    Some(Ok(_)) => {
                        // Binary, Pong — ignore
                    }
                }
            }
            Ok(msg) = rx.recv() => {
                if socket.send(to_ws_text(&msg)).await.is_err() {
                    tracing::warn!(client_id, "Failed to forward broadcast message, closing");
                    break;
                }
            }
        }
    }

    release_all_locks(client_id, &state).await;
}

async fn handle_command(
    cmd: ClientCommand,
    client_id: ClientId,
    state: &AppState,
    socket: &mut WebSocket,
) {
    match cmd {
        ClientCommand::RequestLock { track } => {
            // Acquire, mutate, clone what we need, drop — then do async work.
            let maybe_track_state = {
                let mut seq = state.sequencer.lock().await;
                if seq.tracks[track as usize].locked_by.is_none() {
                    seq.tracks[track as usize].locked_by = Some(client_id);
                    Some(seq.tracks[track as usize].clone())
                } else {
                    None
                }
                // seq dropped here
            };

            match maybe_track_state {
                Some(track_state) => {
                    tracing::info!(client_id, track, "Lock acquired");
                    let _ = state.broadcast.send(ServerMessage::TrackUpdate {
                        track,
                        state: track_state,
                    });
                }
                None => {
                    tracing::debug!(client_id, track, "Lock denied — track already held");
                    let _ = socket
                        .send(to_ws_text(&ServerMessage::LockDenied { track }))
                        .await;
                }
            }
        }

        ClientCommand::ReleaseLock { track } => {
            let maybe_track_state = {
                let mut seq = state.sequencer.lock().await;
                if seq.tracks[track as usize].locked_by == Some(client_id) {
                    seq.tracks[track as usize].locked_by = None;
                    Some(seq.tracks[track as usize].clone())
                } else {
                    tracing::warn!(
                        client_id,
                        track,
                        "ReleaseLock ignored: client does not hold the lock"
                    );
                    None
                }
                // seq dropped here
            };

            if let Some(track_state) = maybe_track_state {
                tracing::info!(client_id, track, "Lock released");
                let _ = state.broadcast.send(ServerMessage::TrackUpdate {
                    track,
                    state: track_state,
                });
            }
        }

        ClientCommand::ToggleStep { track, step } => {
            let maybe_track_state = {
                let mut seq = state.sequencer.lock().await;
                if seq.tracks[track as usize].locked_by == Some(client_id) {
                    seq.tracks[track as usize].steps[step as usize] =
                        !seq.tracks[track as usize].steps[step as usize];
                    Some(seq.tracks[track as usize].clone())
                } else {
                    tracing::warn!(
                        client_id,
                        track,
                        step,
                        "ToggleStep ignored: client does not hold the lock"
                    );
                    None
                }
                // seq dropped here
            };

            if let Some(track_state) = maybe_track_state {
                tracing::debug!(client_id, track, step, "Step toggled");
                let _ = state.broadcast.send(ServerMessage::TrackUpdate {
                    track,
                    state: track_state,
                });
            }
        }
    }
}

async fn release_all_locks(client_id: ClientId, state: &AppState) {
    let released = {
        let mut seq = state.sequencer.lock().await;
        let mut pairs: Vec<(u32, _)> = Vec::new();
        for (idx, track) in seq.tracks.iter_mut().enumerate() {
            if track.locked_by == Some(client_id) {
                track.locked_by = None;
                pairs.push((idx as u32, track.clone()));
            }
        }
        pairs
        // seq dropped here
    };

    for (track, track_state) in released {
        tracing::info!(client_id, track, "Lock released on disconnect");
        let _ = state.broadcast.send(ServerMessage::TrackUpdate {
            track,
            state: track_state,
        });
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

    let (broadcast_tx, _) = broadcast::channel::<ServerMessage>(64);
    let sequencer = Arc::new(Mutex::new(SequencerState::new(8, 16)));
    let app_state = AppState {
        sequencer,
        broadcast: broadcast_tx,
    };

    tracing::info!("Building app & router");
    let app = Router::new()
        .route("/", get(|| async { "libretakt server" }))
        .route("/ws", any(websocket_handler))
        .with_state(app_state);

    const BIND_ADDR: &str = "0.0.0.0:3000";
    tracing::info!("Binding listener to {}...", BIND_ADDR);
    let listener = tokio::net::TcpListener::bind(BIND_ADDR).await.unwrap();
    tracing::info!("Application running at {}", BIND_ADDR);
    axum::serve(listener, app).await.unwrap();
}
