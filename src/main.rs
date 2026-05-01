#[cfg(not(target_arch = "wasm32"))]
mod server;

// When compiling the PC/laptop sound engine server
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    server::main();
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    use ewebsock::WsEvent;
    use libretakt::app_state::create_channels;
    use std::time::Duration;

    // Redirect tracing events to the browser console:
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

    let (app_state, ws_channels) = create_channels();

    let web_options = eframe::WebOptions {
        follow_system_theme: true,
        // default_theme: eframe::Theme::Light,
        ..eframe::WebOptions::default()
    };

    tracing::info!("Spawning UI thread...");
    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "the_canvas_id", // hardcode it
                web_options,
                Box::new(|cc| Box::new(libretakt::ui::LibretaktUI::new(cc, app_state))),
            )
            .await
            .expect("failed to start eframe");
    });

    tracing::info!("Spawning websocket thread...");
    wasm_bindgen_futures::spawn_local(async move {
        loop {
            let options = ewebsock::Options::default();

            ws_channels
                .to_ui
                .unbounded_send("Connecting to websocket...".into())
                .ok();
            let (mut sender, receiver) =
                ewebsock::connect("ws://localhost:3000/ws", options).unwrap();

            loop {
                // TODO: ugly, how can I get asynchronous events
                // from the websocket without polling?
                gloo_timers::future::sleep(Duration::from_millis(10)).await;
                if let Some(event) = receiver.try_recv() {
                    tracing::debug!("WebSocket event: {:?}", event);
                    match event {
                        WsEvent::Opened => {
                            tracing::info!("Websocket opened!");
                            ws_channels
                                .to_ui
                                .unbounded_send("Connected to server".into())
                                .ok();
                        }
                        WsEvent::Closed => {
                            tracing::info!("Websocket closed, disconnecting!");
                            break;
                        }
                        WsEvent::Message(message) => {
                            tracing::info!("WebSocket message: {:?}", message);
                            match message {
                                ewebsock::WsMessage::Binary(_items) => todo!(),
                                ewebsock::WsMessage::Text(_) => todo!(),
                                ewebsock::WsMessage::Unknown(_) => todo!(),
                                ewebsock::WsMessage::Ping(items) => {
                                    use ewebsock::WsMessage;
                                    tracing::debug!("Ping received: {:?}", items);
                                    sender.send(WsMessage::Pong(items));
                                }
                                ewebsock::WsMessage::Pong(items) => {
                                    tracing::error!("Unexpected pong: {:?}", items);
                                }
                            }
                        }
                        WsEvent::Error(error) => {
                            tracing::error!("Websocket error, disconnecting: {:?}", error);
                            break;
                        }
                    }
                    // Forward to UI via channel:
                    // ws_channels.to_ui.unbounded_send(...).ok();
                }
            }

            ws_channels
                .to_ui
                .unbounded_send("Websocket disconnected!".into())
                .ok();
            tracing::error!("Websocket disconnected! Attempting to reconnect...");
            gloo_timers::future::sleep(Duration::from_secs(5)).await;
        }
    });
}
