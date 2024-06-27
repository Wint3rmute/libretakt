use crate::sequencer::Sequencer;
use crate::state::{ProjectData, State, UiState};
use egui::Direction;
use egui::{Align, Context, Ui};
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
    state: State,
    sequencer: Sequencer,
}

impl Default for LibretaktUI {
    fn default() -> Self {
        Self {
            server_url: "http://localhost:8081".to_string(),
            websocket: None,
            // state: State::Disconnected("Connecting..".to_string()),
            state: State::Connected(ProjectData, UiState::PlayerSelection),
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

        // Uncomment to bring back websocket
        // match ewebsock::connect_with_wakeup(&self.server_url, Default::default(), wakeup) {
        //     Ok((ws_sender, ws_receiver)) => {
        //         self.websocket = Some(WebSocketConnection::new(ws_sender, ws_receiver));
        //     }
        //     Err(error) => {
        //         log::error!("Failed to connect to {:?}: {}", &self.server_url, error);
        //         self.state = State::Disconnected("Failed to connect to server".to_string());
        //     }
        // }
    }

    fn show_mixing_console(&mut self, ctx: &Context, ui: &mut Ui) {
        let width = ui.min_size().x / 4.0;
        let height = 60.0;
        let mut my_f32 = 30.5;

        ui.add(egui::Slider::new(&mut my_f32, 0.0..=100.0).text("bASS"));
        ui.add(egui::Slider::new(&mut my_f32, 0.0..=100.0).text("Treble"));
    }

    fn show_sequencer(&mut self, ctx: &Context, ui: &mut Ui) {
        let width = ui.min_size().x / 4.0;
        let height = 60.0;

        ui.label("SEQ");

        egui::Grid::new("sequencer_grid_id")
            .spacing(egui::Vec2 { x: 0.0, y: 0.0 })
            .show(ui, |ui| {
                let current_track = &mut self.sequencer.tracks[0];
                for (step_num, step) in current_track.steps.iter_mut().enumerate() {
                    if step_num != 0 && step_num % 4 == 0 {
                        ui.end_row();
                    }

                    let step_text = egui::widget_text::RichText::new(format!("{step_num}")).color(
                        if step.set {
                            egui::Color32::LIGHT_RED
                        } else {
                            egui::Color32::WHITE
                        },
                    );

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
    }
}

impl eframe::App for LibretaktUI {
    // fn add_step(ui: egui::Ui) {}

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // egui::TopBottomPanel::bottom("bottom_menu").show(ctx, |ui| {
        //     ui.with_layout(
        //         egui::Layout {
        //             main_dir: Direction::LeftToRight,
        //             main_wrap: false,
        //             main_align: Align::Min,
        //             main_justify: false,
        //             cross_align: Align::Min,
        //             cross_justify: false,
        //         },
        //         |ui| {
        //             ui.button("T1");
        //             ui.button("T2");
        //             ui.button("T3");
        //             ui.button("Mixer");
        //         },
        //     );
        // });

        egui::TopBottomPanel::top("my_panel").show(ctx, |ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                match &mut self.state {
                    State::Connected(_project_data, ref mut ui_state) => {
                        if ui.add(egui::Button::new("Back").wrap(true)).clicked() {
                            ui_state.back();
                        }
                    }
                    State::Disconnected(_error_message) => {}
                };

                ui.with_layout(
                    egui::Layout::centered_and_justified(egui::Direction::TopDown),
                    |ui| {
                        let status = self.state.summary_string();
                        ui.label(status);
                    },
                );
                // do_stuff();
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| match &mut self.state {
            State::Connected(project_data, ref mut ui_state) => match ui_state {
                UiState::PlayerSelection => {
                    show_player_selection(ui_state, ctx, ui);
                }
                UiState::AudioTrack_T1 => {
                    self.show_sequencer(ctx, ui);
                }
                UiState::MixingConsole_T0 => {
                    self.show_mixing_console(ctx, ui);
                }
                _ => {}
            },
            State::Disconnected(error_message) => {
                ui.label(error_message.to_string());
            }
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

fn show_player_selection(ui_state: &mut UiState, ctx: &Context, ui: &mut Ui) {
    let width = ui.min_size().x / 2.0;
    let height = 60.0;
    let BUTTON_COLOR = egui::Color32::TRANSPARENT;

    egui::Grid::new("player_selection_id")
        .spacing(egui::Vec2 { x: 0.0, y: 0.0 })
        .show(ui, |ui| {
            if ui
                .add_sized(
                    [width, height],
                    egui::widgets::Button::new("Mixing Console").fill(BUTTON_COLOR),
                )
                .clicked()
            {
                *ui_state = UiState::MixingConsole_T0;
            }

            if ui
                .add_sized(
                    [width, height],
                    egui::widgets::Button::new("Track 1").fill(BUTTON_COLOR),
                )
                .clicked()
            {
                *ui_state = UiState::AudioTrack_T1;
            }

            ui.end_row();

            if ui
                .add_sized(
                    [width, height],
                    egui::widgets::Button::new("Track 2").fill(BUTTON_COLOR),
                )
                .clicked()
            {
                *ui_state = UiState::AudioTrack_T2;
            }

            if ui
                .add_sized(
                    [width, height],
                    egui::widgets::Button::new("Track 3").fill(BUTTON_COLOR),
                )
                .clicked()
            {
                *ui_state = UiState::AudioTrack_T3;
            }
        });
}
