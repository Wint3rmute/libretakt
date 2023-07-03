use egui::Align;
use ewebsock::{WsEvent, WsMessage, WsReceiver, WsSender};
use log::info;

struct WebSocketConnection {
    ws_sender: WsSender,
    ws_receiver: WsReceiver,
    events: Vec<WsEvent>,
    text_to_send: String,
}

impl WebSocketConnection {
    fn new(ws_sender: WsSender, ws_receiver: WsReceiver) -> Self {
        Self {
            ws_sender,
            ws_receiver,
            events: Default::default(),
            text_to_send: Default::default(),
        }
    }

    fn ui(&mut self, ctx: &egui::Context) {
        while let Some(event) = self.ws_receiver.try_recv() {
            self.events.push(event);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Message to send:");
                if ui.text_edit_singleline(&mut self.text_to_send).lost_focus()
                    && ui.input(|i| i.key_pressed(egui::Key::Enter))
                {
                    info!("wololo");
                    self.ws_sender
                        .send(WsMessage::Text(std::mem::take(&mut self.text_to_send)));
                }
            });

            ui.separator();
            ui.heading("Received events:");
            for event in &self.events {
                ui.label(format!("{:?}", event));
            }
        });
    }
}

pub struct LibretaktUI {
    server_url: String,
    websocket: Option<WebSocketConnection>,
    error: Option<String>,
}

impl Default for LibretaktUI {
    fn default() -> Self {
        Self {
            server_url: "http://localhost:8081".to_string(),
            websocket: None,
            error: None,
        }
    }
}

impl LibretaktUI {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        Default::default()
    }

    fn connect(&mut self, ctx: egui::Context) {
        let wakeup = move || ctx.request_repaint(); // wake up UI thread on new message
        match ewebsock::connect_with_wakeup(&self.server_url, wakeup) {
            Ok((ws_sender, ws_receiver)) => {
                self.websocket = Some(WebSocketConnection::new(ws_sender, ws_receiver));
                self.error = None;
            }
            Err(error) => {
                log::error!("Failed to connect to {:?}: {}", &self.server_url, error);
                self.error = Some(error);
            }
        }
    }
}

impl eframe::App for LibretaktUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::bottom("bottom_menu").show(ctx, |ui| {
            ui.with_layout(egui::Layout::left_to_right(Align::Center), |ui| {
                if ui.button("T1").clicked() {
                    info!("Quit");
                }

                if ui.button("T2").clicked() {
                    info!("Quit");
                }

                if ui.button("T3").clicked() {
                    info!("Quit");
                }

                if ui.button("T4").clicked() {
                    info!("Quit");
                }

                if ui.button("Mixer").clicked() {
                    info!("Quit");
                }
            });
        });
    }
}
