use libretakt::engine::{Engine, Voice};
use libretakt::sample_provider::SampleProvider;
use libretakt::sequencer::{
    CurrentStepData, Parameter, Sequencer, SequencerMutation, SynchronisationController,
    NUM_OF_PARAMETERS,
};
use macroquad::prelude::*;

use flume::{bounded, Receiver};
use log::{debug, error, info, log_enabled, Level};
use macroquad::ui::{hash, root_ui, widgets::Group, Skin};
use rodio::{OutputStream, Sink};
use std::sync::Arc;

pub struct Context {
    //(temporary) variables for UI windows dimensions
    pub track_choice_panel_w: f32,
    pub user_panel_w: f32,
    pub track_panel_h: f32,
    pub title_banner_h: f32,

    //Sampler state variables
    pub current_track: i32,
    pub current_step_play: i32,

    //Step selection
    pub selected_step: i32,
    pub parameter_vals: [u8; NUM_OF_PARAMETERS],
    pub parameter_vals_float: [f32; NUM_OF_PARAMETERS],

    //Keyboard varbiales
    pub is_edit_note_pressed: bool,
}

impl Context {
    //Tu są cenne rzeczy
    //jakieś to zjebane to zrobie metode obok xd.

    fn check_user_input(&mut self) {
        self.is_edit_note_pressed = false;

        if is_key_down(KeyCode::LeftControl) {
            self.is_edit_note_pressed = true;
        }
    }
}

pub fn param_of_idx(i: usize) -> Parameter {
    if i == 0 {
        return Parameter::Note;
    }
    if i == 1 {
        return Parameter::PitchShift;
    }
    if i == 2 {
        return Parameter::Sample;
    } else {
        return Parameter::Sample;
    }
}

pub fn assing_context_param(sequencer: &Sequencer, context: &mut Context, param_index: usize) {
    //Pobranie pojedyńczego parametru
    let _temp = sequencer.tracks[0].patterns[0].steps[context.selected_step as usize]
        .as_ref()
        .unwrap()
        .parameters[param_index];

    let mut is_param = false;
    let mut param_val = 0;

    match _temp {
        Some(x) => {
            is_param = true;
            param_val = x;
        }

        None => {}
    }

    if is_param == false {
        context.parameter_vals_float[param_index] = 7 as f32;
        return;
    }

    context.parameter_vals_float[param_index] = param_val as f32;
}

pub fn compare_params_floats_with_original(
    synchronisation_controller: &mut SynchronisationController,
    sequencer: &Sequencer,
    context: &mut Context,
) {
    if context.selected_step == -1 {
        return;
    }

    //Pobranie pojedyńczego parametru
    for i in 0..NUM_OF_PARAMETERS {
        let _temp = sequencer.tracks[0].patterns[0].steps[context.selected_step as usize]
            .as_ref()
            .unwrap()
            .parameters[i as usize];

        let mut is_param = false;
        let mut param_val = 0;

        match _temp {
            Some(x) => {
                is_param = true;
                param_val = x;
            }

            None => {}
        }

        if !is_param {
            continue;
        }

        let eps = 1.0;
        if (context.parameter_vals_float[i] < param_val as f32 - eps)
            || (context.parameter_vals_float[i] > param_val as f32 + eps)
        {
            synchronisation_controller.mutate(SequencerMutation::SetParam(
                0,
                0,
                context.selected_step as usize,
                param_of_idx(i),
                (context.parameter_vals_float[i] as usize)
                    .try_into()
                    .unwrap(),
            ));
        }
    }
}

pub fn select_step(sequencer: &Sequencer, context: &mut Context, step_index: i32) {
    context.selected_step = step_index;

    //Initialize proper params values
    for i in 0..NUM_OF_PARAMETERS {
        assing_context_param(sequencer, context, i as usize);
    }
}

#[macroquad::main("LibreTakt")]
async fn main() {
    //***SAMPLER***
    env_logger::init();

    //To be honest i haven't been looking at this code yet but Bączek wrote it
    //so i guess its something important and i trust him.
    let provider = Arc::new(SampleProvider::default());
    let mut synchronisation_controller = SynchronisationController::default();

    let (current_step_tx, current_step_rx) = bounded::<CurrentStepData>(64);

    let voice = Voice::new(&provider);
    let engine = Engine {
        sequencer: Sequencer::new(
            synchronisation_controller.register_new(),
            current_step_tx.clone(),
        ),
        voices: vec![voice],
    };

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    sink.append(engine);
    sink.play();

    let sequencer = Sequencer::new(synchronisation_controller.register_new(), current_step_tx);

    ui_main(sequencer, synchronisation_controller, current_step_rx).await;
}

async fn ui_main(
    mut sequencer: Sequencer,
    mut synchronisation_controller: SynchronisationController,
    step_data_receiver: Receiver<CurrentStepData>,
) {
    let _sample = 0.0;

    //***UI Skins***
    //There is probably way to edit ui elements properties more efficiently but
    //im too stupid to figure it out from documentation and i found examples
    //of doing it so this way uwu
    let titlebanner_skin = {
        let label_style = root_ui()
            .style_builder()
            .font(include_bytes!("../fff-forward/MinimalPixel v2.ttf"))
            .unwrap()
            .text_color(BLACK)
            .font_size(50)
            .build();

        Skin {
            label_style,
            ..root_ui().default_skin()
        }
    };

    let empty_note_skin = {
        let button_style = root_ui()
            .style_builder()
            .background(Image::from_file_with_format(
                include_bytes!("../uigraphics/note_background_0.png"),
                None,
            ))
            .background_margin(RectOffset::new(8.0, 8.0, 8.0, 8.0))
            .background_hovered(Image::from_file_with_format(
                include_bytes!("../uigraphics/note_hovered_background_0.png"),
                None,
            ))
            .background_clicked(Image::from_file_with_format(
                include_bytes!("../uigraphics/note_clicked_background_0.png"),
                None,
            ))
            .font(include_bytes!("../fff-forward/MinimalPixel v2.ttf"))
            .unwrap()
            .text_color(Color::from_rgba(180, 180, 100, 0))
            .font_size(40)
            .build();

        Skin {
            button_style,
            ..root_ui().default_skin()
        }
    };

    let note_placed_skin = {
        let button_style = root_ui()
            .style_builder()
            .background(Image::from_file_with_format(
                include_bytes!("../uigraphics/note_background_1.png"),
                None,
            ))
            .background_margin(RectOffset::new(8.0, 8.0, 8.0, 8.0))
            .background_hovered(Image::from_file_with_format(
                include_bytes!("../uigraphics/note_background_1.png"),
                None,
            ))
            .background_clicked(Image::from_file_with_format(
                include_bytes!("../uigraphics/note_clicked_background_1.png"),
                None,
            ))
            .font(include_bytes!("../fff-forward/MinimalPixel v2.ttf"))
            .unwrap()
            .text_color(Color::from_rgba(180, 180, 100, 0))
            .font_size(40)
            .build();

        Skin {
            button_style,
            ..root_ui().default_skin()
        }
    };

    let note_playing_skin = {
        let button_style = root_ui()
            .style_builder()
            .background(Image::from_file_with_format(
                include_bytes!("../uigraphics/note_background_2.png"),
                None,
            ))
            .background_margin(RectOffset::new(8.0, 8.0, 8.0, 8.0))
            .background_hovered(Image::from_file_with_format(
                include_bytes!("../uigraphics/note_background_2.png"),
                None,
            ))
            .background_clicked(Image::from_file_with_format(
                include_bytes!("../uigraphics/note_clicked_background_0.png"),
                None,
            ))
            .font(include_bytes!("../fff-forward/MinimalPixel v2.ttf"))
            .unwrap()
            .text_color(Color::from_rgba(180, 180, 100, 0))
            .font_size(40)
            .build();

        Skin {
            button_style,
            ..root_ui().default_skin()
        }
    };

    let note_selected_skin = {
        let button_style = root_ui()
            .style_builder()
            .background(Image::from_file_with_format(
                include_bytes!("../uigraphics/note_background_3.png"),
                None,
            ))
            .background_margin(RectOffset::new(8.0, 8.0, 8.0, 8.0))
            .background_hovered(Image::from_file_with_format(
                include_bytes!("../uigraphics/note_background_3.png"),
                None,
            ))
            .background_clicked(Image::from_file_with_format(
                include_bytes!("../uigraphics/note_clicked_background_0.png"),
                None,
            ))
            .font(include_bytes!("../fff-forward/MinimalPixel v2.ttf"))
            .unwrap()
            .text_color(Color::from_rgba(180, 180, 100, 0))
            .font_size(40)
            .build();

        Skin {
            button_style,
            ..root_ui().default_skin()
        }
    };

    //UI Skins Load
    let _default_skin = root_ui().default_skin().clone();
    let titlebanner_skin_clone = titlebanner_skin.clone();
    let note_empty_skin_clone = empty_note_skin.clone();
    let note_placed_skin_clone = note_placed_skin.clone();
    let note_playing_skin_clone = note_playing_skin.clone();
    let note_selected_skin_clone = note_selected_skin.clone();

    //Building Context
    //This struck will change but im too lazy to fix it right now
    let mut context = Context {
        track_choice_panel_w: 100.,
        user_panel_w: 100.,
        track_panel_h: 300.,
        title_banner_h: 60.,

        current_track: -1,
        current_step_play: 0,

        selected_step: -1,
        parameter_vals: [0u8; NUM_OF_PARAMETERS],
        parameter_vals_float: [0f32; NUM_OF_PARAMETERS],

        is_edit_note_pressed: false,
    };

    loop {
        clear_background(WHITE);

        if let Ok(step_data) = step_data_receiver.try_recv() {
            sequencer.tracks[0].current_step = step_data[0];
        }

        {
            //Assigning main variables from sequencer.
            //Not sure if they should be assinged to some context or exist freely this way
            let sequencer = &sequencer;
            let num_of_steps = sequencer.tracks[0].patterns[0].steps.len();
            // sequencer.tracks[0].default_parameters.parameters[Parameter::Sample as usize] =
            // sample as u8;

            // TODO BACZEK FIX
            context.current_step_play = sequencer.tracks[0].current_step as i32;

            //READ USER INPUT
            context.check_user_input();

            //Jakiś extra space na logike kodu
            compare_params_floats_with_original(
                &mut synchronisation_controller,
                &sequencer,
                &mut context,
            );

            //***DRAWING UI PANELS***
            //***TITLE PANEL***
            //This panel is purely for aesthetic reason and shows the title of
            //app in fancy way (hopefully in the future...)
            root_ui().push_skin(&titlebanner_skin_clone);
            root_ui().window(
                hash!("Titlewindow"),
                vec2(0., 0.),
                vec2(screen_width(), context.title_banner_h),
                |ui| {
                    ui.label(Vec2::new(0., 0.), "TURBO SAMPLER");
                },
            );
            root_ui().pop_skin();

            //***MAIN TRACK PANEL***
            //This panel shows the track currently selected by user.
            //Clicking displayed notes allows user to edit their sound.
            root_ui().window(
                hash!("MainWindow"),
                vec2(context.track_choice_panel_w, context.title_banner_h),
                vec2(
                    screen_width() - context.track_choice_panel_w - context.user_panel_w,
                    context.track_panel_h,
                ),
                |ui| {
                    Group::new(hash!("GRP1"), Vec2::new(screen_width() - 210., 40.)).ui(ui, |ui| {
                        if context.current_track != -1 {
                            ui.label(
                                Vec2::new(0., 0.),
                                &("TRACK #".to_owned() + &(context.current_track + 1).to_string()),
                            );
                        } else {
                            ui.label(Vec2::new(0., 0.), "SELECT TRACK");
                        }
                        ui.label(Vec2::new(100., 0.), &context.current_step_play.to_string());
                    });

                    if context.current_track != -1 {
                        for i in 0..num_of_steps {
                            Group::new(hash!("Tracks", i), Vec2::new(70., 60.)).ui(ui, |ui| {
                                if context.selected_step == i as i32 {
                                    ui.push_skin(&note_selected_skin_clone);
                                } else if context.current_step_play == i as i32 {
                                    ui.push_skin(&note_playing_skin_clone);
                                } else if sequencer.tracks[0].patterns[0].steps[i].is_some() {
                                    ui.push_skin(&note_placed_skin_clone);
                                } else {
                                    ui.push_skin(&note_empty_skin_clone);
                                }

                                if ui.button(Vec2::new(0., 0.), "....") {
                                    //im not sure if this kind of if/else chain is valid
                                    //i would use some "returns" and tide it up a bit but i think i cant coz its not a method
                                    context.selected_step = -1;
                                    if sequencer.tracks[0].patterns[0].steps[i].is_some() {
                                        //EDIT MODE:
                                        if context.is_edit_note_pressed {
                                            select_step(&sequencer, &mut context, i as i32);
                                        } else {
                                            synchronisation_controller
                                                .mutate(SequencerMutation::RemoveStep(0, 0, i))
                                            // sequencer.tracks[0].patterns[0].steps[i] = None;
                                        }
                                    } else {
                                        synchronisation_controller
                                            .mutate(SequencerMutation::CreateStep(0, 0, i))
                                        // sequencer.tracks[0].patterns[0].steps[i] = Some(Step::default());
                                    }
                                }

                                ui.pop_skin();
                            });
                        }
                    }
                },
            );

            //***TRACK CHOICE PANEL***
            //This panel lists all available tracks. Clicking on one of them shows its content
            //on the main Panel.
            //Todo: Tracks in use have different colors. They can not be selected by user.
            root_ui().window(
                hash!("ChoiceWindow"),
                vec2(0., context.title_banner_h),
                vec2(context.track_choice_panel_w, context.track_panel_h),
                |ui| {
                    Group::new(hash!("GRP3"), Vec2::new(90., 20.)).ui(ui, |ui| {
                        ui.label(Vec2::new(0., 0.), "TRACKS");
                    });

                    for i in 0..sequencer.tracks.len() {
                        Group::new(hash!("Track2", i), Vec2::new(90., 30.)).ui(ui, |ui| {
                            if ui.button(Vec2::new(30., 0.), (i + 1).to_string()) {
                                //TODO - dodać warunek że track nie jest zalockowany przez innego uzytkownika!!!
                                context.current_track = i as i32;
                            }
                        });
                    }
                },
            );

            //***USER PANEL***
            //Displays current users in the jam session, their nick with the corresponding colour.
            root_ui().window(
                hash!("USERWINDOW"),
                vec2(
                    screen_width() - context.user_panel_w,
                    context.title_banner_h,
                ),
                vec2(context.user_panel_w, context.track_panel_h),
                |ui| {
                    Group::new(hash!("GRP6"), Vec2::new(90., 20.)).ui(ui, |ui| {
                        ui.label(Vec2::new(0., 0.), "USERS");
                    });
                },
            );

            //***SETTINGS/EDIT PANEL***
            //This panel allows user to edit parameters of currently selected note.
            root_ui().window(
                hash!("Settings"),
                vec2(0., context.title_banner_h + context.track_panel_h),
                vec2(
                    screen_width(),
                    screen_height() - context.title_banner_h - context.track_panel_h,
                ),
                |ui| {
                    //Jeżeli jest wybrany step w trybie edycji zrób całą magię
                    if context.selected_step == -1 {
                        //UI Lable in top left
                        ui.label(Vec2::new(0., 0.), "NO STEP SELECTED!");
                    }

                    //Option BOX
                    if context.selected_step != -1 {
                        //Utwórz 4 slidery do edycji parametrów:
                        for i in 0..sequencer.tracks[0].patterns[0].steps
                            [context.selected_step as usize]
                            .as_ref()
                            .unwrap()
                            .parameters
                            .len()
                        {
                            //Pobranie pojedyńczego parametru
                            let _temp = sequencer.tracks[0].patterns[0].steps
                                [context.selected_step as usize]
                                .as_ref()
                                .unwrap()
                                .parameters[i];

                            let mut is_param = false;
                            let mut param_val = 0;

                            match _temp {
                                Some(x) => {
                                    is_param = true;
                                    param_val = x;
                                }

                                None => {}
                            }

                            Group::new(hash!("PanelSettings", i), Vec2::new(700., 70.)).ui(
                                ui,
                                |ui| {
                                    Group::new(hash!("Group LAbel", i), Vec2::new(680., 20.)).ui(
                                        ui,
                                        |ui| {
                                            ui.label(
                                                Vec2::new(0., 0.),
                                                &("Parameter #".to_owned() + &(i + 1).to_string()),
                                            );
                                        },
                                    );

                                    Group::new(hash!("Group Button", i), Vec2::new(40., 38.)).ui(
                                        ui,
                                        |ui| {
                                            if ui.button(
                                                Vec2::new(0., 0.),
                                                if is_param { "X" } else { "." },
                                            ) {
                                                if is_param {
                                                    //switch is_param
                                                    is_param = false;

                                                    //Delete parameter
                                                    synchronisation_controller.mutate(
                                                        SequencerMutation::RemoveParam(
                                                            0,
                                                            0,
                                                            context.selected_step as usize,
                                                            param_of_idx(i),
                                                        ),
                                                    );
                                                } else {
                                                    //switch is_param
                                                    is_param = true;
                                                    //Add parameter
                                                    let default_param = 0.0;
                                                    context.parameter_vals_float[i] = default_param;
                                                    synchronisation_controller.mutate(
                                                        SequencerMutation::SetParam(
                                                            0,
                                                            0,
                                                            context.selected_step as usize,
                                                            param_of_idx(i),
                                                            (default_param as usize)
                                                                .try_into()
                                                                .unwrap(),
                                                        ),
                                                    );
                                                }
                                            }
                                        },
                                    );

                                    Group::new(hash!("Group Slider", i), Vec2::new(500., 38.)).ui(
                                        ui,
                                        |ui| {
                                            if is_param == true {
                                                ui.slider(
                                                    hash!("param slider", i),
                                                    "",
                                                    0f32..64f32,
                                                    &mut context.parameter_vals_float[i],
                                                );
                                            }
                                        },
                                    );
                                },
                            );
                        }
                    }
                },
            );

            //Some leftover code that i decided to comment if i ever need to quickly look how to make sliders.
            //WILL BE DELETED LATER!!!

            // main_window.draw();
            // context.check_all_bounds();
            /*
                for i in 0..num_of_steps {
                    root_ui().slider(hash!(), "[-10 .. 10]", 0f32..10f32, &mut sample);
                    if root_ui().button(
                        None,
                        if sequencer.tracks[0].steps[i].is_some() {
                            "X"
                        } else {
                            " "
                        },
                    ) {
                        if sequencer.tracks[0].steps[i].is_some() {
                            sequencer.tracks[0].steps[i] = None;
                        } else {
                            sequencer.tracks[0].steps[i] = Some(sequencer::Step::default());
                        }
                    }

            */
        }
        sequencer.apply_mutations();
        next_frame().await;
    }
}
