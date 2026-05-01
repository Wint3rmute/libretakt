use ewebsock::WsEvent;
use std::time::Duration;

use super::app_state::create_channels;
use crate::shared::ServerMessage;

pub fn main() {
    // Redirect tracing events to the browser console.
    use tracing_subscriber::prelude::*;
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .without_time()
        .with_writer(tracing_web::MakeWebConsoleWriter::new());
    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(tracing_subscriber::filter::LevelFilter::DEBUG)
        .init();

    tracing::info!("Trying to get URL params...");
    if let Some(search) = web_sys::window()
        .and_then(|w| w.location().search().ok())
        .filter(|s| !s.is_empty())
    {
        for pair in search.trim_start_matches('?').split('&') {
            let mut parts = pair.splitn(2, '=');
            let key = parts.next().unwrap_or("");
            let val = parts.next().unwrap_or("");
            tracing::info!("URL param: {} = {}", key, val);
        }
    }

    let (app_state, mut ws_channels) = create_channels();

    let web_options = eframe::WebOptions {
        follow_system_theme: true,
        ..eframe::WebOptions::default()
    };

    tracing::info!("Spawning UI task...");
    wasm_bindgen_futures::spawn_local(async move {
        eframe::WebRunner::new()
            .start(
                "the_canvas_id",
                web_options,
                Box::new(|cc| Box::new(super::LibretaktUI::new(cc, app_state))),
            )
            .await
            .expect("failed to start eframe");
    });

    tracing::info!("Spawning WebSocket task...");
    wasm_bindgen_futures::spawn_local(async move {
        loop {
            let options = ewebsock::Options::default();

            // TODO: make the WebSocket URL configurable (e.g. via a URL parameter).
            let ws_result = ewebsock::connect("ws://localhost:3000/ws", options);
            let (mut sender, receiver) = match ws_result {
                Ok(pair) => pair,
                Err(e) => {
                    tracing::error!("Failed to open WebSocket connection: {:?}", e);
                    gloo_timers::future::sleep(Duration::from_secs(5)).await;
                    continue;
                }
            };

            loop {
                // Yield to the browser event loop. WASM is single-threaded and
                // cooperative, so we must .await periodically.
                gloo_timers::future::sleep(Duration::from_millis(10)).await;

                // Drain outbound commands queued by the UI and send over the socket.
                while let Ok(Some(cmd)) = ws_channels.from_ui.try_next() {
                    match serde_json::to_string(&cmd) {
                        Ok(text) => {
                            sender.send(ewebsock::WsMessage::Text(text));
                        }
                        Err(e) => {
                            tracing::warn!("Failed to serialise outbound command: {:?}", e);
                        }
                    }
                }

                if let Some(event) = receiver.try_recv() {
                    tracing::debug!("WebSocket event: {:?}", event);
                    match event {
                        WsEvent::Opened => {
                            tracing::info!("WebSocket opened!");
                        }
                        WsEvent::Closed => {
                            tracing::info!("WebSocket closed, will reconnect.");
                            break;
                        }
                        WsEvent::Message(message) => match message {
                            ewebsock::WsMessage::Text(text) => {
                                match serde_json::from_str::<ServerMessage>(&text) {
                                    Ok(msg) => {
                                        ws_channels.to_ui.unbounded_send(msg).ok();
                                    }
                                    Err(e) => {
                                        tracing::warn!("Failed to parse server message: {:?}", e);
                                    }
                                }
                            }
                            ewebsock::WsMessage::Binary(items) => {
                                tracing::warn!(
                                    "Unexpected binary WebSocket message ({} bytes), ignoring",
                                    items.len()
                                );
                            }
                            ewebsock::WsMessage::Unknown(s) => {
                                tracing::warn!("Unknown WebSocket message type, ignoring: {}", s);
                            }
                            ewebsock::WsMessage::Ping(items) => {
                                tracing::debug!("Ping received, sending Pong");
                                sender.send(ewebsock::WsMessage::Pong(items));
                            }
                            ewebsock::WsMessage::Pong(items) => {
                                tracing::warn!("Unexpected Pong received, ignoring: {:?}", items);
                            }
                        },
                        WsEvent::Error(error) => {
                            tracing::error!("WebSocket error, disconnecting: {:?}", error);
                            break;
                        }
                    }
                }
            }

            tracing::error!("WebSocket disconnected! Attempting to reconnect in 5 s...");
            gloo_timers::future::sleep(Duration::from_secs(5)).await;
        }
    });
}
