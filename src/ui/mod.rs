use egui::Align;
use egui::Direction;
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
    fn add_step(ui: egui::Ui) {}

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::bottom("bottom_menu").show(ctx, |ui| {
            ui.with_layout(
                egui::Layout {
                    main_dir: Direction::LeftToRight,
                    main_wrap: false,
                    main_align: Align::Min,
                    main_justify: false,
                    cross_align: Align::Min,
                    cross_justify: false,
                },
                |ui| {
                    ui.button("T1");
                    ui.button("T2");
                    ui.button("T3");
                    ui.button("Mixer");
                },
            );
        });

        egui::TopBottomPanel::top("my_panel").show(ctx, |ui| {
            ui.label("Hello World! From `TopBottomPanel`, that must be before `CentralPanel`!");
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let width = ui.min_size().x / 4.0;
            let height = 60.0;

            egui::Grid::new("some_unique_id")
                .spacing(egui::Vec2 { x: 0.0, y: 0.0 })
                .show(ui, |ui| {
                    ui.add_sized([width, height], egui::widgets::Button::new("asasdsa"));
                    ui.add_sized([width, height], egui::widgets::Button::new("asasdsa"));
                    ui.add_sized([width, height], egui::widgets::Button::new("asasdsa"));
                    ui.add_sized([width, height], egui::widgets::Button::new("asasdsa"));
                    ui.end_row();
                    ui.add_sized([width, height], egui::widgets::Button::new("asasdsa"));
                    ui.add_sized([width, height], egui::widgets::Button::new("asasdsa"));
                    ui.add_sized([width, height], egui::widgets::Button::new("asasdsa"));
                    ui.add_sized([width, height], egui::widgets::Button::new("asasdsa"));
                    ui.end_row();
                });
        });

        // ui.with_layout(egui::Layout::left_to_right(Align::Center), |ui| {
        //     if ui.button("T1").clicked() {
        //         info!("Quit");
        //     }

        //     if ui.button("T2").clicked() {
        //         info!("Quit");
        //     }

        //     if ui.button("T3").clicked() {
        //         info!("Quit");
        //     }

        //     if ui.button("T4").clicked() {
        //         info!("Quit");
        //     }

        //     if ui.button("Mixer").clicked() {
        //         info!("Quit");
        //     }
        // });
    }
}
