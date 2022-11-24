extern crate num;
#[macro_use]
extern crate num_derive;

use libretakt::constants::NUM_OF_VOICES;
use libretakt::engine::{Engine, Voice};
use libretakt::sample_provider::SampleProvider;
use libretakt::sequencer::{
    CurrentStepData, Parameter, Sequencer, SequencerMutation, SynchronisationController,
    NUM_OF_PARAMETERS,
};
use macroquad::prelude::*;

use flume::{bounded, Receiver};

use macroquad::ui::{hash, root_ui, widgets::Group, Skin};
use rodio::{OutputStream, Sink};
use std::sync::Arc;

use strum::IntoEnumIterator; // 0.17.1

use strum_macros::EnumIter; // 0.17.1

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

    //SliderGroups
    pub current_slider_group: i32,
    pub slider_group_sizes: Vec<i32>,

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
    let mut iterator = 0;
    for param in Parameter::iter() {
        if i == iterator {
            return param;
        }
        iterator += 1;
    }

    Parameter::Sample
}

pub fn is_in_current_slided_group(context: &Context, i: i32) -> bool {
    let mut sliders_before_count = 0;
    let mut current_iter_group = 0;

    let sliders_group_iter = context.slider_group_sizes.iter();

    for val in sliders_group_iter {
        if context.current_slider_group == current_iter_group {
            if i < sliders_before_count {
                return false;
            }
            if i + 1 > sliders_before_count + val {
                return false;
            }
            return true;
        }

        sliders_before_count += val;
        current_iter_group += 1;
    }

    false
}

pub fn assing_context_param(sequencer: &Sequencer, context: &mut Context, param_index: usize) {
    //Pobranie pojedyńczego parametru
    let _temp = sequencer.tracks[context.current_track as usize].patterns[0].steps
        [context.selected_step as usize]
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

    if !is_param {
        context.parameter_vals_float[param_index] = 7_f32;
        return;
    }

    context.parameter_vals_float[param_index] = param_val as f32;
}

pub fn assign_context_track_params(sequencer: &Sequencer, context: &mut Context) {
    let track_params = sequencer.tracks[context.current_track as usize]
        .default_parameters
        .parameters;

    for i in 0..NUM_OF_PARAMETERS {
        context.parameter_vals_float[i as usize] = track_params[i as usize] as f32;
    }
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
        let _temp = sequencer.tracks[context.current_track as usize].patterns[0].steps
            [context.selected_step as usize]
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
                context.current_track as usize,
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

pub fn compare_floats_with_original_track(
    synchronisation_controller: &mut SynchronisationController,
    sequencer: &Sequencer,
    context: &mut Context,
) {
    //Przejdź po wszystkich parametrach tracku/aż nie znajdziemy mutacji
    for i in 0..NUM_OF_PARAMETERS {
        let mut param_val = sequencer.tracks[context.current_track as usize]
            .default_parameters
            .parameters[i as usize];

        let eps = 1.0;
        if (context.parameter_vals_float[i] < param_val as f32 - eps)
            || (context.parameter_vals_float[i] > param_val as f32 + eps)
        {
            synchronisation_controller.mutate(SequencerMutation::UpdateTrackParam(
                context.current_track as usize,
                i as usize,
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

pub fn deselect_step(sequencer: &Sequencer, context: &mut Context) {
    context.selected_step = -1;

    assign_context_track_params(sequencer, context);
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

    let _voice = Voice::new(&provider);
    let engine = Engine {
        sequencer: Sequencer::new(
            synchronisation_controller.register_new(),
            current_step_tx.clone(),
        ),
        voices: (0..NUM_OF_VOICES).map(|_| Voice::new(&provider)).collect(),
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

        current_track: 0,
        current_step_play: 0,

        selected_step: -1,
        parameter_vals: [0u8; NUM_OF_PARAMETERS],
        parameter_vals_float: [0f32; NUM_OF_PARAMETERS],

        current_slider_group: 0,
        slider_group_sizes: vec![8, 8, 8],

        is_edit_note_pressed: false,
    };

    deselect_step(&sequencer, &mut context);

    loop {
        clear_background(WHITE);

        if let Ok(step_data) = step_data_receiver.try_recv() {
            sequencer.tracks[context.current_track as usize].current_step = step_data[0];
        }

        {
            //Assigning main variables from sequencer.
            //Not sure if they should be assinged to some context or exist freely this way
            let sequencer = &sequencer;
            let num_of_steps = sequencer.tracks[context.current_track as usize].patterns[0]
                .steps
                .len();
            // sequencer.tracks[0].default_parameters.parameters[Parameter::Sample as usize] =
            // sample as u8;

            // TODO BACZEK FIX
            context.current_step_play =
                sequencer.tracks[context.current_track as usize].current_step as i32;

            //READ USER INPUT
            context.check_user_input();

            //Jakiś extra space na logike kodu
            if context.selected_step != -1 {
                compare_params_floats_with_original(
                    &mut synchronisation_controller,
                    &sequencer,
                    &mut context,
                );
            } else {
                compare_floats_with_original_track(
                    &mut synchronisation_controller,
                    &sequencer,
                    &mut context,
                );
            }

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
                                } else if sequencer.tracks[context.current_track as usize].patterns
                                    [0]
                                .steps[i]
                                    .is_some()
                                {
                                    ui.push_skin(&note_placed_skin_clone);
                                } else {
                                    ui.push_skin(&note_empty_skin_clone);
                                }

                                if ui.button(Vec2::new(0., 0.), "....") {
                                    //im not sure if this kind of if/else chain is valid
                                    //i would use some "returns" and tide it up a bit but i think i cant coz its not a method
                                    deselect_step(&sequencer, &mut context);
                                    if sequencer.tracks[context.current_track as usize].patterns[0]
                                        .steps[i]
                                        .is_some()
                                    {
                                        //EDIT MODE:
                                        if context.is_edit_note_pressed {
                                            select_step(sequencer, &mut context, i as i32);
                                        } else {
                                            synchronisation_controller.mutate(
                                                SequencerMutation::RemoveStep(
                                                    context.current_track as usize,
                                                    0,
                                                    i,
                                                ),
                                            )
                                            // sequencer.tracks[0].patterns[0].steps[i] = None;
                                        }
                                    } else {
                                        synchronisation_controller.mutate(
                                            SequencerMutation::CreateStep(
                                                context.current_track as usize,
                                                0,
                                                i,
                                            ),
                                        )
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
                                deselect_step(&sequencer, &mut context);
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
                    //Option BOX

                    //Context Slide Group
                    //Is responsible for showing deciding which slider panels to show

                    Group::new(hash!("Slider Group Selector Box"), Vec2::new(700., 45.)).ui(
                        ui,
                        |ui| {
                            //ui.label(Vec2::new(0., 0.), "SLIDER GROUPS SELECTOR");

                            for button_i in 0..3 {
                                Group::new(
                                    hash!("ASGADGXXZXCZSSCBHRAZEEHSEH", button_i),
                                    Vec2::new(40., 40.),
                                )
                                .ui(ui, |ui| {
                                    if ui.button(Vec2::new(0., 0.), button_i.to_string()) {
                                        context.current_slider_group = button_i;
                                    }
                                });
                            }
                        },
                    );

                    //STEPS LOGIC!!!
                    //Utwórz  slidery do edycji parametrów:
                    if (context.selected_step != -1) {
                        for i in 0..sequencer.tracks[context.current_track as usize].patterns[0]
                            .steps[context.selected_step as usize]
                            .as_ref()
                            .unwrap()
                            .parameters
                            .len()
                        {
                            //Sprawdź czy jest to current slider group
                            //if not current slider group => continue
                            if !is_in_current_slided_group(&context, i as i32) {
                                continue;
                            }

                            //Pobranie pojedyńczego parametru
                            let _temp = sequencer.tracks[context.current_track as usize].patterns
                                [0]
                            .steps[context.selected_step as usize]
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
                                            let parameter: Parameter =
                                                num::FromPrimitive::from_usize(i).unwrap();

                                            ui.label(Vec2::new(0., 0.), &parameter.to_string());
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
                                                            context.current_track as usize,
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
                                                            context.current_track as usize,
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
                    } else {
                        let track_params = sequencer.tracks[context.current_track as usize]
                            .default_parameters
                            .parameters;

                        for i in 0..track_params.len() {
                            if !is_in_current_slided_group(&context, i as i32) {
                                continue;
                            }

                            Group::new(hash!("PanelSettings asf", i), Vec2::new(700., 70.)).ui(
                                ui,
                                |ui| {
                                    Group::new(
                                        hash!("Group LAbel asfasf", i),
                                        Vec2::new(680., 20.),
                                    )
                                    .ui(ui, |ui| {
                                        let parameter: Parameter =
                                            num::FromPrimitive::from_usize(i).unwrap();

                                        ui.label(Vec2::new(0., 0.), &parameter.to_string());
                                    });

                                    Group::new(
                                        hash!("Group Slider asdasfa", i),
                                        Vec2::new(500., 38.),
                                    )
                                    .ui(ui, |ui| {
                                        ui.slider(
                                            hash!("param slider ...", i),
                                            "",
                                            0f32..64f32,
                                            &mut context.parameter_vals_float[i],
                                        );
                                    });
                                },
                            );
                        }
                    }
                },
            );
        }
        sequencer.apply_mutations();
        next_frame().await;
    }
}
