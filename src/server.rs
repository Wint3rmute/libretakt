use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::any,
    Router,
};
use libretakt::shared::{ClientCommand, ClientId, SequencerState, ServerMessage};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use tokio::sync::{broadcast, Mutex};

#[derive(rust_embed::RustEmbed)]
#[folder = "dist/"]
struct StaticAssets;

async fn static_handler(uri: axum::http::Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };

    match StaticAssets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime.essence_str())], content.data).into_response()
        }
        None => match StaticAssets::get("index.html") {
            Some(index) => (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "text/html")],
                index.data,
            )
                .into_response(),
            None => (
                StatusCode::NOT_FOUND,
                [(header::CONTENT_TYPE, "text/plain")],
                b"404 not found",
            )
                .into_response(),
        },
    }
}

/// Shared server state, cloned cheaply into each connection handler via [`Arc`] internals.
#[derive(Clone)]
struct AppState {
    /// The canonical sequencer state, shared across all connected clients.
    ///
    /// Guarded by a [`Mutex`] so that command handlers can read-modify-write
    /// atomically. Always lock, mutate, clone what you need, then drop the guard
    /// before any `.await` point to avoid holding the lock across yield points.
    sequencer: Arc<Mutex<SequencerState>>,
    /// Broadcast channel used to fan out [`ServerMessage`]s to every connected client.
    ///
    /// Each connection handler holds a [`broadcast::Receiver`] subscribed to this
    /// sender. Sending is fire-and-forget: lagging receivers are dropped by Tokio
    /// automatically.
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
                                if let Some(reply) = handle_command(cmd, client_id, &state).await {
                                    if socket.send(to_ws_text(&reply)).await.is_err() {
                                        tracing::warn!(client_id, "Failed to send reply, closing");
                                        break;
                                    }
                                }
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

/// Process a [`ClientCommand`] and mutate shared state.
///
/// Returns `Some(reply)` for messages that should be sent directly back to the
/// requesting client (e.g. [`ServerMessage::LockDenied`]), or `None` when the
/// response (if any) was already dispatched via the broadcast channel.
async fn handle_command(
    cmd: ClientCommand,
    client_id: ClientId,
    state: &AppState,
) -> Option<ServerMessage> {
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
                    None
                }
                None => {
                    tracing::debug!(client_id, track, "Lock denied — track already held");
                    Some(ServerMessage::LockDenied { track })
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
            None
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
            None
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

fn make_app_state(num_tracks: usize, steps_per_track: usize) -> AppState {
    let (broadcast_tx, _) = broadcast::channel::<ServerMessage>(64);
    AppState {
        sequencer: Arc::new(Mutex::new(SequencerState::new(num_tracks, steps_per_track))),
        broadcast: broadcast_tx,
    }
}

#[tokio::main]
pub async fn main() {
    const BIND_ADDR: &str = "0.0.0.0:3000";

    tracing_subscriber::fmt()
        .with_file(true)
        .with_line_number(true)
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(tracing::Level::INFO.into())
                .from_env_lossy(),
        )
        .init();

    let app_state = make_app_state(8, 16);

    tracing::info!("Building app & router");
    let app = Router::new()
        .route("/ws", any(websocket_handler))
        .fallback(static_handler)
        .with_state(app_state);

    tracing::info!("Binding listener to {}...", BIND_ADDR);
    let listener = tokio::net::TcpListener::bind(BIND_ADDR).await.unwrap();
    tracing::info!("Application running at {}", BIND_ADDR);
    axum::serve(listener, app).await.unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Run a command as `client_id` against `state` and return the optional direct reply.
    async fn cmd(
        state: &AppState,
        client_id: ClientId,
        command: ClientCommand,
    ) -> Option<ServerMessage> {
        handle_command(command, client_id, state).await
    }

    #[tokio::test]
    async fn request_lock_grants_then_denies() {
        let state = make_app_state(2, 8);
        let mut broadcast = state.broadcast.subscribe();

        // Client 1 acquires the lock - no direct reply, broadcast carries the update.
        let reply = cmd(&state, 1, ClientCommand::RequestLock { track: 0 }).await;
        assert!(
            reply.is_none(),
            "successful lock should not produce a direct reply"
        );

        let msg = broadcast
            .try_recv()
            .expect("broadcast should carry TrackUpdate");
        assert!(
            matches!(msg, ServerMessage::TrackUpdate { track: 0, .. }),
            "expected TrackUpdate for track 0, got {msg:?}"
        );

        // Client 2 tries to acquire the same track - should get LockDenied directly.
        let reply = cmd(&state, 2, ClientCommand::RequestLock { track: 0 }).await;
        assert!(
            matches!(reply, Some(ServerMessage::LockDenied { track: 0 })),
            "expected LockDenied, got {reply:?}"
        );
        assert!(
            broadcast.try_recv().is_err(),
            "a denied lock should not broadcast anything"
        );
    }

    #[tokio::test]
    async fn toggle_step_requires_lock() {
        let state = make_app_state(2, 8);
        let mut broadcast = state.broadcast.subscribe();

        // Toggle without holding the lock - silently ignored, nothing broadcast.
        let reply = cmd(&state, 1, ClientCommand::ToggleStep { track: 0, step: 3 }).await;
        assert!(reply.is_none());
        assert!(
            broadcast.try_recv().is_err(),
            "ignored toggle should not broadcast"
        );

        // Acquire the lock, then toggle.
        cmd(&state, 1, ClientCommand::RequestLock { track: 0 }).await;
        let _ = broadcast.try_recv(); // consume the RequestLock TrackUpdate

        let reply = cmd(&state, 1, ClientCommand::ToggleStep { track: 0, step: 3 }).await;
        assert!(reply.is_none());

        let msg = broadcast
            .try_recv()
            .expect("toggle should broadcast TrackUpdate");
        if let ServerMessage::TrackUpdate {
            track: 0,
            state: track_state,
        } = msg
        {
            assert!(track_state.steps[3], "step 3 should now be true");
        } else {
            panic!("expected TrackUpdate for track 0, got {msg:?}");
        }
    }

    #[tokio::test]
    async fn release_lock_frees_track_for_others() {
        let state = make_app_state(2, 8);
        let mut broadcast = state.broadcast.subscribe();

        cmd(&state, 1, ClientCommand::RequestLock { track: 0 }).await;
        let _ = broadcast.try_recv();

        cmd(&state, 1, ClientCommand::ReleaseLock { track: 0 }).await;
        let msg = broadcast
            .try_recv()
            .expect("release should broadcast TrackUpdate");
        if let ServerMessage::TrackUpdate {
            track: 0,
            state: track_state,
        } = msg
        {
            assert!(
                track_state.locked_by.is_none(),
                "track should be free after release"
            );
        } else {
            panic!("expected TrackUpdate for track 0, got {msg:?}");
        }

        // Another client can now acquire the lock.
        let reply = cmd(&state, 2, ClientCommand::RequestLock { track: 0 }).await;
        assert!(
            reply.is_none(),
            "lock should be granted to client 2 after release"
        );
    }
}
