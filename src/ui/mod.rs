use crate::sequencer::Sequencer;
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
    sequencer: Sequencer,
}

impl Default for LibretaktUI {
    fn default() -> Self {
        Self {
            server_url: "http://localhost:8081".to_string(),
            websocket: None,
            error: None,
            sequencer: Sequencer::default(),
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
    // fn add_step(ui: egui::Ui) {}

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
            ui.label("CONNECTED: OFFLINE, JAMMERS: 1");
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let width = ui.min_size().x / 4.0;
            let height = 60.0;

            ui.label("SEQ");

            egui::Grid::new("some_unique_id")
                .spacing(egui::Vec2 { x: 0.0, y: 0.0 })
                .show(ui, |ui| {
                    let current_track = &mut self.sequencer.tracks[0];
                    for (step_num, step) in current_track.steps.iter_mut().enumerate() {
                        if step_num != 0 && step_num % 4 == 0 {
                            ui.end_row();
                        }

                        let step_text = egui::widget_text::RichText::new(format!("{step_num}"))
                            .color(if step.set {
                                egui::Color32::LIGHT_RED
                            } else {
                                egui::Color32::WHITE
                            });

                        if ui
                            .add_sized(
                                [width, height],
                                egui::widgets::Button::new(step_text).fill(
                                    if current_track.current_step_num == step_num {
                                        egui::Color32::DARK_GRAY
                                    } else {
                                        egui::Color32::TRANSPARENT
                                    },
                                ),
                            )
                            .clicked()
                        {
                            step.set = !step.set;
                            if step.set {
                                info!("step {step_num} set");
                            } else {
                                info!("step {step_num} unset");
                            }
                        }
                    }
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
