extern crate num;
#[macro_use]
extern crate num_derive;

use crate::ui_skins::*;
use common::{Parameter, SequencerMutation, NUM_OF_PARAMETERS};
use libretakt::constants::NUM_OF_VOICES;
use libretakt::engine::{Engine, Voice};
use libretakt::mutation_websocket;
use libretakt::persistence::{load_project, save_project};
use libretakt::sample_provider::SampleProvider;
use libretakt::sequencer::{CurrentStepData, Sequencer, SynchronisationController, Track};
mod ui_skins;
use env_logger::Env;

use macroquad::miniquad::Texture;
use macroquad::prelude::*;

use flume::{bounded, Receiver};

use env;
use macroquad::ui::{hash, root_ui, widgets::Group, Skin};
use macroquad::window::Conf;

use rodio::{OutputStream, Sink};
use std::io::Read;
use std::sync::{Arc, Mutex};

use strum::IntoEnumIterator; // 0.17.1

// 0.17.1

pub struct Context {
    //(temporary) variables for UI windows dimensions
    pub track_choice_panel_w: f32,
    pub user_panel_w: f32,
    pub track_panel_h: f32,
    pub title_banner_h: f32,

    //Track Cards
    pub current_track_card: i32,

    //Pattern varibles:
    pub max_patterns: i32,
    pub current_pattern: usize,

    //Sampler state variables:
    pub current_track: i32,
    pub current_step_play: i32,

    //Step selection
    pub current_note_highlighted: i32,
    pub selected_step: i32,
    pub parameter_vals: [u8; NUM_OF_PARAMETERS],
    pub parameter_vals_float: [f32; NUM_OF_PARAMETERS],

    //SliderGroups
    pub current_slider_group: i32,
    pub slider_group_sizes: Vec<i32>,
    pub current_slider: i32,

    //Keyboard varbiales
    pub is_edit_note_pressed: bool,
    pub is_shift_pressed: bool,
    pub is_tab_pressed: bool,
    pub is_escape_pressed: bool,
    pub mapped_note_key_idx: i32,
    pub pressed_number: i32,
    pub is_mute_pressed: bool,

    //Arrow Operations
    pub vertical_move_button: i32,
    pub horizontal_move_button: i32,
    pub super_horizontal_move_button: i32,
    pub slider_group_switch_tab: i32,

    //UI TABS
    pub main_tab: i32,
}

impl Context {
    //Tu są cenne rzeczy
    //jakieś to zjebane to zrobie metode obok xd.

    fn check_user_input(&mut self) {
        //self.is_edit_note_pressed = false;
        //self.is_shift_pressed = false;
        self.is_escape_pressed = false;

        if is_key_pressed(KeyCode::LeftControl) {
            self.is_edit_note_pressed = true;
        }

        if is_key_pressed(KeyCode::LeftShift) {
            self.is_shift_pressed = true;
        }

        if is_key_down(KeyCode::Escape) {
            self.is_escape_pressed = true;
        }

        if is_key_pressed(KeyCode::Tab) {
            self.is_tab_pressed = true;
        }

        //Map keyboard note key
        if is_key_down(KeyCode::Q) {
            self.mapped_note_key_idx = 0i32;
        } else if is_key_down(KeyCode::W) {
            self.mapped_note_key_idx = 1i32;
        } else if is_key_down(KeyCode::E) {
            self.mapped_note_key_idx = 2i32;
        } else if is_key_down(KeyCode::R) {
            self.mapped_note_key_idx = 3i32;
        } else if is_key_down(KeyCode::T) {
            self.mapped_note_key_idx = 4i32;
        } else if is_key_down(KeyCode::Y) {
            self.mapped_note_key_idx = 5i32;
        } else if is_key_down(KeyCode::U) {
            self.mapped_note_key_idx = 6i32;
        } else if is_key_down(KeyCode::I) {
            self.mapped_note_key_idx = 7i32;
        } else if is_key_down(KeyCode::A) {
            self.mapped_note_key_idx = 8i32;
        } else if is_key_down(KeyCode::S) {
            self.mapped_note_key_idx = 9i32;
        } else if is_key_down(KeyCode::D) {
            self.mapped_note_key_idx = 10i32;
        } else if is_key_down(KeyCode::F) {
            self.mapped_note_key_idx = 11i32;
        } else if is_key_down(KeyCode::G) {
            self.mapped_note_key_idx = 12i32;
        } else if is_key_down(KeyCode::H) {
            self.mapped_note_key_idx = 13i32;
        } else if is_key_down(KeyCode::J) {
            self.mapped_note_key_idx = 14i32;
        } else if is_key_down(KeyCode::K) {
            self.mapped_note_key_idx = 15i32;
        }

        if is_key_pressed(KeyCode::Key1) {
            self.pressed_number = 0i32;
        } else if is_key_pressed(KeyCode::Key2) {
            self.pressed_number = 1i32;
        } else if is_key_pressed(KeyCode::Key3) {
            self.pressed_number = 2i32;
        } else if is_key_pressed(KeyCode::Key4) {
            self.pressed_number = 3i32;
        } else if is_key_pressed(KeyCode::Key5) {
            self.pressed_number = 4i32;
        } else if is_key_pressed(KeyCode::Key6) {
            self.pressed_number = 5i32;
        } else if is_key_pressed(KeyCode::Key7) {
            self.pressed_number = 6i32;
        } else if is_key_pressed(KeyCode::Key8) {
            self.pressed_number = 7i32;
        } else if is_key_pressed(KeyCode::Key9) {
            self.pressed_number = 8i32;
        }

        if is_key_released(KeyCode::Key1)
            || is_key_released(KeyCode::Key2)
            || is_key_released(KeyCode::Key3)
            || is_key_released(KeyCode::Key4)
            || is_key_released(KeyCode::Key5)
            || is_key_released(KeyCode::Key6)
            || is_key_released(KeyCode::Key7)
            || is_key_released(KeyCode::Key8)
            || is_key_released(KeyCode::Key9)
        {
            self.pressed_number = -1i32;
        }

        if is_key_down(KeyCode::M) {
            self.is_mute_pressed = true;
        } else {
            self.is_mute_pressed = false;
        }

        //Arrow move
        //Map keyboard note key
        if is_key_pressed(KeyCode::W) {
            self.vertical_move_button = -1i32;
        } else if is_key_pressed(KeyCode::S) {
            self.vertical_move_button = 1i32;
        }

        if is_key_pressed(KeyCode::D) {
            self.horizontal_move_button = 1i32;
        } else if is_key_pressed(KeyCode::A) {
            self.horizontal_move_button = -1i32;
        }

        if is_key_pressed(KeyCode::C) {
            self.super_horizontal_move_button = 1i32;
        } else if is_key_pressed(KeyCode::Z) {
            self.super_horizontal_move_button = -1i32;
        }

        if is_key_pressed(KeyCode::E) {
            self.slider_group_switch_tab = 1i32;
        } else if is_key_pressed(KeyCode::Q) {
            self.slider_group_switch_tab = -1i32;
        }
    }
}

pub fn change_track(context: &mut Context, sequencer: &Sequencer, new_track: i32) {
    context.current_track_card = 0;
    context.current_track = new_track;
    deselect_step(sequencer, context);
}

pub fn add_track_card(
    context: &mut Context,
    synchronisation_controller: &Arc<Mutex<SynchronisationController>>,
    sequencer: &Sequencer,
) {
    let mut steps_in_track = sequencer.tracks[context.current_track as usize].patterns[0]
        .steps
        .len();

    synchronisation_controller
        .lock()
        .unwrap()
        .mutate(SequencerMutation::SetTrackLength(
            context.current_track as usize,
            steps_in_track + 16,
        ));
}

pub fn delete_track_card(
    context: &mut Context,
    synchronisation_controller: &Arc<Mutex<SynchronisationController>>,
    sequencer: &Sequencer,
) {
    let mut steps_in_track = sequencer.tracks[context.current_track as usize].patterns[0]
        .steps
        .len();

    if steps_in_track == 16 {
        return;
    }

    if context.current_track_card + 1
        == (sequencer.tracks[context.current_track as usize].patterns[0]
            .steps
            .len()
            / 16) as i32
    {
        context.current_track_card -= 1;
    }

    synchronisation_controller
        .lock()
        .unwrap()
        .mutate(SequencerMutation::SetTrackLength(
            context.current_track as usize,
            steps_in_track - 16,
        ));
}

pub fn move_track_card(
    context: &mut Context,
    synchronisation_controller: &Arc<Mutex<SynchronisationController>>,
    sequencer: &Sequencer,
    value: i32,
) {
    let mut new_value = context.current_track_card + value;

    //Lower boundry check
    if new_value < 0 {
        return;
    }

    //Upper boundry check
    if new_value
        >= (sequencer.tracks[context.current_track as usize].patterns[0]
            .steps
            .len()
            / 16) as i32
    {
        return;
    }

    context.current_track_card = new_value;
}

pub fn change_pattern(
    context: &mut Context,
    synchronisation_controller: &Arc<Mutex<SynchronisationController>>,
    sequencer: &Sequencer,
    i: usize,
) {
    //Wykonaj mutacje:
    synchronisation_controller
        .lock()
        .unwrap()
        .mutate(SequencerMutation::SelectPattern(
            context.current_track as usize,
            i as usize,
        ));

    deselect_step(&sequencer, context);
}

pub fn create_param(
    context: &mut Context,
    synchronisation_controller: &Arc<Mutex<SynchronisationController>>,
    i: usize,
) {
    let default_param = 0.0;
    context.parameter_vals_float[i] = default_param;
    synchronisation_controller
        .lock()
        .unwrap()
        .mutate(SequencerMutation::SetParam(
            context.current_track as usize,
            context.current_pattern,
            context.selected_step as usize,
            param_of_idx(i),
            (default_param as usize).try_into().unwrap(),
        ));
}

pub fn delete_param(
    context: &mut Context,
    synchronisation_controller: &Arc<Mutex<SynchronisationController>>,
    i: usize,
) {
    //Delete parameter
    synchronisation_controller
        .lock()
        .unwrap()
        .mutate(SequencerMutation::RemoveParam(
            context.current_track as usize,
            context.current_pattern,
            context.selected_step as usize,
            param_of_idx(i),
        ));
}

pub fn param_of_idx(i: usize) -> Parameter {
    for (iterator, param) in Parameter::iter().enumerate() {
        if i == iterator {
            return param;
        }
    }

    Parameter::Sample
}

pub fn is_in_current_slided_group(context: &Context, i: i32) -> bool {
    let mut sliders_before_count = 0;

    let sliders_group_iter = context.slider_group_sizes.iter();

    for (current_iter_group, val) in sliders_group_iter.enumerate() {
        if context.current_slider_group == current_iter_group as i32 {
            if i < sliders_before_count {
                return false;
            }
            if i + 1 > sliders_before_count + val {
                return false;
            }
            return true;
        }

        sliders_before_count += val;
    }

    false
}

pub fn assing_context_param(sequencer: &Sequencer, context: &mut Context, param_index: usize) {
    //Pobranie pojedyńczego parametru
    let _temp = sequencer.tracks[context.current_track as usize].patterns[context.current_pattern]
        .steps[context.selected_step as usize]
        .as_ref()
        .unwrap()
        .parameters[param_index];

    let mut is_param = false;
    let mut param_val = 0;

    if let Some(x) = _temp {
        is_param = true;
        param_val = x;
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
    synchronisation_controller: &Arc<Mutex<SynchronisationController>>,
    sequencer: &Sequencer,
    context: &mut Context,
) {
    if context.selected_step == -1 {
        return;
    }

    //Pobranie pojedyńczego parametru
    for i in 0..NUM_OF_PARAMETERS {
        let _temp = sequencer.tracks[context.current_track as usize].patterns
            [context.current_pattern]
            .steps[context.selected_step as usize]
            .as_ref()
            .unwrap()
            .parameters[i as usize];

        let mut is_param = false;
        let mut param_val = 0;

        if let Some(x) = _temp {
            is_param = true;
            param_val = x;
        }

        if !is_param {
            continue;
        }

        if context.parameter_vals_float[i] as u8 != param_val {
            synchronisation_controller
                .lock()
                .unwrap()
                .mutate(SequencerMutation::SetParam(
                    context.current_track as usize,
                    context.current_pattern,
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
    synchronisation_controller: &Arc<Mutex<SynchronisationController>>,
    sequencer: &Sequencer,
    context: &mut Context,
) {
    //Przejdź po wszystkich parametrach tracku/aż nie znajdziemy mutacji
    for i in 0..NUM_OF_PARAMETERS {
        let param_val = sequencer.tracks[context.current_track as usize]
            .default_parameters
            .parameters[i as usize];

        if context.parameter_vals_float[i] as u8 != param_val {
            synchronisation_controller
                .lock()
                .unwrap()
                .mutate(SequencerMutation::UpdateTrackParam(
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

pub fn silence_track(
    _sequencer: &Sequencer,
    _context: &mut Context,
    synchronisation_controller: &Arc<Mutex<SynchronisationController>>,
    i: usize,
) {
    synchronisation_controller
        .lock()
        .unwrap()
        .mutate(SequencerMutation::SilenceTrack(i));
}

pub fn unsilence_track(
    _sequencer: &Sequencer,
    _context: &mut Context,
    synchronisation_controller: &Arc<Mutex<SynchronisationController>>,
    i: usize,
) {
    synchronisation_controller
        .lock()
        .unwrap()
        .mutate(SequencerMutation::UnSilenceTrack(i));
}

pub fn perform_keyboard_operations(
    sequencer: &Sequencer,
    context: &mut Context,
    synchronisation_controller: &Arc<Mutex<SynchronisationController>>,
) {
    //update cooldown
    //context.current_cooldown-=1.0f32;

    //Check if in notest TAB

    if context.is_escape_pressed {
        context.mapped_note_key_idx = -1;
        context.is_shift_pressed = false;
        context.is_edit_note_pressed = false;
        deselect_step(sequencer, context);
    }

    if context.is_tab_pressed
        && context.pressed_number > -1
        && context.pressed_number < sequencer.tracks.len() as i32
    {
        change_track(context, sequencer, context.pressed_number as i32);
        context.is_tab_pressed = false;
        context.pressed_number = -1;
        return;
    }

    if context.is_mute_pressed
        && context.pressed_number > -1
        && context.pressed_number < sequencer.tracks.len() as i32
    {
        let i = context.pressed_number as usize;
        let is_silenced = sequencer.tracks[i].silenced;
        if is_silenced {
            unsilence_track(sequencer, context, synchronisation_controller, i);
        } else {
            silence_track(sequencer, context, synchronisation_controller, i);
        }

        context.pressed_number = -1;
        return;
    }

    if context.is_tab_pressed {
        context.current_note_highlighted = -1;
        context.mapped_note_key_idx = -1;
        context.slider_group_switch_tab = 0;
        context.horizontal_move_button = 0;
        context.vertical_move_button = 0;
        context.super_horizontal_move_button = 0;
        context.is_shift_pressed = false;
        context.is_edit_note_pressed = false;

        context.main_tab += 1;
        context.main_tab %= 2;
        context.is_tab_pressed = false;
    }

    if context.main_tab == 0 {
        keyboard_operations_notes(sequencer, context, synchronisation_controller);
    } else if context.main_tab == 1 {
        keyboard_operations_sliders(sequencer, context, synchronisation_controller);
    }

    //Check if in Bars TAB
}

pub fn keyboard_operations_notes(
    sequencer: &Sequencer,
    context: &mut Context,
    synchronisation_controller: &Arc<Mutex<SynchronisationController>>,
) {
    context.current_note_highlighted = context.mapped_note_key_idx;

    if context.is_shift_pressed && context.current_note_highlighted != -1 {
        //Check cooldown?
        context.is_shift_pressed = false;
        //If there is not note - add note
        if sequencer.tracks[context.current_track as usize].patterns[context.current_pattern].steps
            [context.current_note_highlighted as usize]
            .is_some()
        {
            if context.selected_step == context.current_note_highlighted {
                deselect_step(sequencer, context);
            }

            synchronisation_controller
                .lock()
                .unwrap()
                .mutate(SequencerMutation::RemoveStep(
                    context.current_track as usize,
                    context.current_pattern,
                    context.current_note_highlighted as usize,
                ))
            // sequencer.tracks[0].patterns[0].steps[i] = None;
        } else {
            synchronisation_controller
                .lock()
                .unwrap()
                .mutate(SequencerMutation::CreateStep(
                    context.current_track as usize,
                    context.current_pattern,
                    context.current_note_highlighted as usize,
                ))
            // sequencer.tracks[0].patterns[0].steps[i] = Some(Step::default());
        }
    }

    if context.is_edit_note_pressed && context.current_note_highlighted != -1 {
        context.is_edit_note_pressed = false;
        if context.selected_step == context.current_note_highlighted {
            deselect_step(sequencer, context);
        } else if sequencer.tracks[context.current_track as usize].patterns[context.current_pattern]
            .steps[context.current_note_highlighted as usize]
            .is_some()
        {
            //EDIT MODE:
            select_step(sequencer, context, context.current_note_highlighted);
        }
    }
}

pub fn keyboard_operations_sliders(
    sequencer: &Sequencer,
    context: &mut Context,
    synchronisation_controller: &Arc<Mutex<SynchronisationController>>,
) {
    if context.slider_group_switch_tab != 0 {
        context.current_slider_group += context.slider_group_switch_tab + 3;
        context.current_slider_group %= 3;
        context.slider_group_switch_tab = 0;
    }

    if context.vertical_move_button != 0 {
        context.current_slider += context.vertical_move_button;
        if context.current_slider < 0 {
            context.current_slider = 0;
        }
        if context.current_slider
            > context.slider_group_sizes[context.current_slider_group as usize] - 1
        {
            context.current_slider =
                context.slider_group_sizes[context.current_slider_group as usize] - 1;
        }

        context.vertical_move_button = 0;
    }

    if context.selected_step != -1 {
        let _temp = sequencer.tracks[context.current_track as usize].patterns
            [context.current_pattern]
            .steps[context.selected_step as usize]
            .as_ref()
            .unwrap()
            .parameters[(context.current_slider + context.current_slider_group * 8) as usize];

        let mut is_param = false;
        //let mut param_val = 0;

        if let Some(_x) = _temp {
            is_param = true;
            //param_val = x;
        }

        if context.is_shift_pressed {
            if is_param {
                delete_param(
                    context,
                    synchronisation_controller,
                    (context.current_slider + context.current_slider_group * 8) as usize,
                );
                is_param = false;
            } else {
                create_param(
                    context,
                    synchronisation_controller,
                    (context.current_slider + context.current_slider_group * 8) as usize,
                );
                is_param = true;
            }

            context.is_shift_pressed = false;
        }
        //if so:
        if !is_param {
            return;
        }
    }

    if context.horizontal_move_button != 0 {
        context.parameter_vals_float
            [(context.current_slider + context.current_slider_group * 8) as usize] +=
            context.horizontal_move_button as f32;
        let float_value = context.parameter_vals_float
            [(context.current_slider + context.current_slider_group * 8) as usize];
        if float_value < 0.0 {
            context.parameter_vals_float
                [(context.current_slider + context.current_slider_group * 8) as usize] = 0.0;
        }
        if float_value > 64.0 {
            context.parameter_vals_float
                [(context.current_slider + context.current_slider_group * 8) as usize] = 64.0;
        }

        context.horizontal_move_button = 0;
    }

    if context.super_horizontal_move_button != 0 {
        context.parameter_vals_float
            [(context.current_slider + context.current_slider_group * 8) as usize] +=
            context.super_horizontal_move_button as f32 * 10.0;
        let float_value = context.parameter_vals_float
            [(context.current_slider + context.current_slider_group * 8) as usize];
        if float_value < 0.0 {
            context.parameter_vals_float
                [(context.current_slider + context.current_slider_group * 8) as usize] = 0.0;
        }
        if float_value > 64.0 {
            context.parameter_vals_float
                [(context.current_slider + context.current_slider_group * 8) as usize] = 64.0;
        }

        context.super_horizontal_move_button = 0;
    }
}

#[macroquad::main("LibreTakt")]
async fn main() {
    //***SAMPLER***
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let tracks = load_project();

    //To be honest i haven't been looking at this code yet but Bączek wrote it
    //so i guess its something important and i trust him 👉👈.
    let provider = Arc::new(SampleProvider::default());
    let mut synchronisation_controller = Arc::new(Mutex::new(SynchronisationController::default()));

    let (current_step_tx, current_step_rx) = bounded::<CurrentStepData>(64);

    let _voice = Voice::new(&provider);
    let engine = Engine {
        sequencer: Sequencer::new(
            synchronisation_controller.lock().unwrap().register_new(),
            current_step_tx.clone(),
            tracks.clone(),
        ),
        voices: (0..NUM_OF_VOICES).map(|_| Voice::new(&provider)).collect(),
    };

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    sink.append(engine);
    sink.play();

    #[cfg(feature = "enable_synchronisation")]
    {
        warn!("Synchronisation enabled, connecting to synchronisation server..");
        let mutation_rx_for_sync_server = synchronisation_controller.lock().unwrap().register_new();

        let sync_controller_clone = synchronisation_controller.clone();
        std::thread::spawn(|| {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async {
                mutation_websocket::send_mutations_to_server(
                    mutation_rx_for_sync_server,
                    sync_controller_clone,
                )
                .await
            })
        });
    }

    let sequencer = Sequencer::new(
        synchronisation_controller.lock().unwrap().register_new(),
        current_step_tx,
        tracks.clone(),
    );
    prevent_quit();
    ui_main(
        sequencer,
        provider,
        synchronisation_controller,
        current_step_rx,
    )
    .await;
}

fn is_step_hit(track_num: usize, sequencer: &Sequencer) -> bool {
    sequencer.tracks[track_num].patterns[sequencer.tracks[track_num].current_pattern]
        .steps
        .len()
        > sequencer.tracks[track_num].current_step as usize
        && !sequencer.tracks[track_num].silenced
        && sequencer.tracks[track_num].patterns[sequencer.tracks[track_num].current_pattern].steps
            [sequencer.tracks[track_num].current_step]
            .is_some()
}

async fn ui_main(
    mut sequencer: Sequencer,
    sample_provider: Arc<SampleProvider>,
    mut synchronisation_controller: Arc<Mutex<SynchronisationController>>,
    step_data_receiver: Receiver<CurrentStepData>,
) {
    let _sample = 0.0;

    // Load the cats
    let cat_up: Texture2D = load_texture("uigraphics/cat_up.png").await.unwrap();
    // let cat_down: Texture2D = load_texture("uigraphics/cat_down.png").await.unwrap();
    let cat_down_left: Texture2D = load_texture("uigraphics/cat_down_left.png").await.unwrap();
    let cat_down_right: Texture2D = load_texture("uigraphics/cat_down_right.png").await.unwrap();

    let cat_cymbal_up: Texture2D = load_texture("uigraphics/cat_cymbal_up.png").await.unwrap();
    let cat_cymbal_down_left: Texture2D = load_texture("uigraphics/cat_cymbal_down_left.png")
        .await
        .unwrap();

    let cat_cymbal_down_right: Texture2D = load_texture("uigraphics/cat_cymbal_down_right.png")
        .await
        .unwrap();

    let cat_piano_left: Texture2D = load_texture("uigraphics/cat_left.png").await.unwrap();
    let cat_piano_right: Texture2D = load_texture("uigraphics/cat_right.png").await.unwrap();
    let cat_piano_none: Texture2D = load_texture("uigraphics/cat_none.png").await.unwrap();

    let cat_piano2_middle: Texture2D = load_texture("uigraphics/cat_middle.png").await.unwrap();
    let cat_piano2_none: Texture2D = load_texture("uigraphics/cat_none_2.png").await.unwrap();

    //Loading UI Skins from ui_skins.rs to not clutter main.rs with code that does not belong in here
    let titlebanner_struct = TitleBannerSkin::new();
    let empty_note_struct = EmptyNoteSkin::new();
    let note_placed_struct = NotePlacedSkin::new();
    let note_playing_struct = NotePlayingSkin::new();
    let note_selected_struct = NoteSelectedSkin::new();
    let empty_note_highlighted_struct = EmptyNoteHighlightedSkin::new();
    let note_placed_highlighted_struct = NotePlacedHighlightedSkin::new();

    //UI Skins Load
    let _default_skin = root_ui().default_skin().clone();
    let titlebanner_skin_clone = titlebanner_struct.titlebanner_skin.clone();
    let note_empty_skin_clone = empty_note_struct.empty_note_skin.clone();
    let note_placed_skin_clone = note_placed_struct.note_placed_skin.clone();
    let note_playing_skin_clone = note_playing_struct.note_playing_skin.clone();
    let note_selected_skin_clone = note_selected_struct.note_selected_skin.clone();
    let empty_note_highlighted_skin_clone = empty_note_highlighted_struct
        .empty_note_highlighted_skin
        .clone();
    let note_placed_highlighted_skin_clone = note_placed_highlighted_struct
        .note_placed_highlighted_skin
        .clone();

    //Building Context
    //This struck will change but im too lazy to fix it right now
    let mut context = Context {
        track_choice_panel_w: 100.,
        user_panel_w: 100.,
        track_panel_h: 300.,
        title_banner_h: 60.,

        current_track_card: 0,

        current_track: 0,
        current_step_play: 0,

        max_patterns: 4i32,
        current_pattern: 0usize,

        current_note_highlighted: -1,
        selected_step: -1,
        parameter_vals: [0u8; NUM_OF_PARAMETERS],
        parameter_vals_float: [0f32; NUM_OF_PARAMETERS],

        current_slider_group: 0,
        slider_group_sizes: vec![8, 8, 8],
        current_slider: 0,

        vertical_move_button: 0i32,
        horizontal_move_button: 0i32,
        super_horizontal_move_button: 0i32,
        slider_group_switch_tab: 0i32,

        is_edit_note_pressed: false,
        is_shift_pressed: false,
        is_tab_pressed: false,
        is_escape_pressed: false,
        mapped_note_key_idx: -1,
        is_mute_pressed: false,
        pressed_number: -1,

        main_tab: 0i32,
    };

    deselect_step(&sequencer, &mut context);

    let mut cat_1_seen = false;
    let mut cat_2_seen = false;
    let mut cat_3_seen = false;
    let mut cat_4_seen = false;

    loop {
        clear_background(WHITE);

        if is_quit_requested() {
            info!("Saving sequencer state...");
            save_project(&sequencer);
            info!("Exiting");
            break;
        }

        if let Ok(step_data) = step_data_receiver.try_recv() {
            for (track, step) in sequencer.tracks.iter_mut().zip(step_data) {
                track.current_step = step;
            }
        }

        {
            //Assigning main variables from sequencer.
            //Not sure if they should be assinged to some context or exist freely this way
            let sequencer = &sequencer;

            // TODO BACZEK FIX
            context.current_step_play =
                sequencer.tracks[context.current_track as usize].current_step as i32;

            //READ USER INPUT
            context.check_user_input();
            perform_keyboard_operations(sequencer, &mut context, &mut synchronisation_controller);

            //To musi być sprawdzane w pętli bo czasami kod wykonuje się szybciej niż mutacja (mutacja jest zlagowana sequencera o jedną iteracje kodu?! xd)
            context.current_pattern =
                sequencer.tracks[context.current_track as usize].current_pattern;

            //Jakiś extra space na logike kodu
            if context.selected_step != -1 {
                compare_params_floats_with_original(
                    &synchronisation_controller,
                    sequencer,
                    &mut context,
                );
            } else {
                compare_floats_with_original_track(
                    &synchronisation_controller,
                    sequencer,
                    &mut context,
                );
            }

            //DRAW EVERYTHING AS GROUPS NOT WINDOWS!!
            //~ Sure thing boss, but I'll also draw a cat

            // let current_cat = if sequencer.tracks[context.current_track as usize].patterns
            //     [context.current_pattern]
            //     .steps
            //     .len()
            //     <= context.current_step_play as usize
            //     || sequencer.tracks[context.current_track as usize].silenced
            // {
            //     if context.current_track == 3 {
            //         cat_piano_none
            //     } else if context.current_track == 1 {
            //         cat_cymbal_up
            //     } else if context.current_track == 2 {
            //         cat_piano2_none
            //     } else {
            //         cat_up
            //     }
            // } else {
            //     if sequencer.tracks[context.current_track as usize].patterns
            //         [context.current_pattern as usize]
            //         .steps[context.current_step_play as usize]
            //         .is_some()
            //     {
            //         if context.current_track == 3 {
            //             if context.current_step_play % 2 == 0 {
            //                 cat_piano_left
            //             } else {
            //                 cat_piano_right
            //             }
            //         } else if context.current_track == 1 {
            //             cat_cymbal_down
            //         } else if context.current_track == 2 {
            //             cat_piano2_middle
            //         } else {
            //             cat_down
            //         }
            //     } else {
            //         if context.current_track == 3 {
            //             cat_piano_none
            //         } else if context.current_track == 1 {
            //             cat_cymbal_up
            //         } else if context.current_track == 2 {
            //             cat_piano2_none
            //         } else {
            //             cat_up
            //         }
            //     }
            // };
            // draw_texture(
            //     current_cat,
            //     750.,
            //     200.,
            //     // screen_width() / 2. - cat_up.width() / 2.,
            //     // screen_height() / 2. - cat_up.height() / 2.,
            //     WHITE,
            // );

            if cat_1_seen || !sequencer.tracks[0].silenced {
                if is_step_hit(0, sequencer) && sequencer.playing {
                    if sequencer.tracks[0].current_step % 2 == 0 {
                        draw_texture(cat_down_right, 750., 200., WHITE);
                    } else {
                        draw_texture(cat_down_left, 750., 200., WHITE);
                    }
                    cat_1_seen = true;
                } else {
                    draw_texture(cat_up, 750., 200., WHITE);
                }
            }

            if cat_2_seen || !sequencer.tracks[1].silenced {
                if is_step_hit(1, sequencer) && sequencer.playing {
                    if sequencer.tracks[1].current_step % 2 == 0 {
                        draw_texture(cat_cymbal_down_left, 750., 400., WHITE);
                    } else {
                        draw_texture(cat_cymbal_down_right, 750., 400., WHITE);
                    }
                    cat_2_seen = true;
                } else {
                    draw_texture(cat_cymbal_up, 750., 400., WHITE);
                }
            }

            if cat_3_seen || !sequencer.tracks[2].silenced {
                if is_step_hit(2, sequencer) && sequencer.playing {
                    draw_texture(cat_piano2_middle, 750., 600., WHITE);
                    cat_3_seen = true;
                } else {
                    draw_texture(cat_piano2_none, 750., 600., WHITE);
                }
            }

            if cat_4_seen || !sequencer.tracks[3].silenced {
                if is_step_hit(3, sequencer) && sequencer.playing {
                    if sequencer.tracks[3].current_step % 2 == 0 {
                        draw_texture(cat_piano_left, 750., 800., WHITE);
                        cat_4_seen = true;
                    } else {
                        draw_texture(cat_piano_right, 750., 800., WHITE);
                    }
                } else {
                    draw_texture(cat_piano_none, 750., 800., WHITE);
                }
            }

            /*
            root_ui().push_skin(&titlebanner_skin_clone);
            root_ui().group(
                hash!(),
                vec2(screen_width() - 10.0, context.title_banner_h),
                |ui| {
                    ui.label(Vec2::new(0., 0.), " libretakt");
                },
            );
            root_ui().pop_skin();
            */

            root_ui().push_skin(&titlebanner_skin_clone);
            root_ui().window(
                hash!("Titlewindow"),
                vec2(0., 0.),
                vec2(screen_width(), context.title_banner_h),
                |ui| {
                    ui.label(Vec2::new(0., 0.), " libretakt");
                },
            );
            root_ui().pop_skin();

            //MAIN TRACK PANEL
            //This panel shows the track currently selected by user.
            //Clicking displayed notes allows user to edit their sound.

            root_ui().window(
                hash!("MainWindow"),
                vec2(context.track_choice_panel_w, context.title_banner_h),
                vec2(610., 150.),
                |ui| {
                    //Group wypisujący nazwę aktualnego tracka
                    Group::new(hash!("GRP1"), Vec2::new(580., 40.)).ui(ui, |ui| {
                        if context.current_track != -1 {
                            ui.label(
                                Vec2::new(0., 0.),
                                &("TRACK #".to_owned() + &(context.current_track + 1).to_string()),
                            );
                        } else {
                            ui.label(Vec2::new(0., 0.), "SELECT TRACK");
                        }
                        ui.label(Vec2::new(100., 0.), &context.current_step_play.to_string());

                        if ui.button(Vec2::new(200., 0.), "Play/Pause")
                            || is_key_pressed(KeyCode::Space)
                        {
                            if sequencer.playing {
                                synchronisation_controller
                                    .lock()
                                    .unwrap()
                                    .mutate(SequencerMutation::StopPlayback);
                            } else {
                                synchronisation_controller
                                    .lock()
                                    .unwrap()
                                    .mutate(SequencerMutation::StartPlayback);
                            }
                        }
                    });

                    //Group związany z przechodzeniem między patternami
                    Group::new(hash!("Przechodzenie miedzy panelami"), Vec2::new(580., 40.)).ui(
                        ui,
                        |ui| {
                            //Utwórz guziki związane z przełączaniem między patternami:
                            for i in 0..context.max_patterns {
                                Group::new(
                                    hash!("Pattern group".to_owned() + &i.to_string()),
                                    Vec2::new(40., 38.),
                                )
                                .ui(ui, |ui| {
                                    if ui.button(
                                        Vec2::new(0., 0.),
                                        if (sequencer.tracks[context.current_track as usize]
                                            .current_pattern
                                            == i as usize)
                                        {
                                            "X"
                                        } else {
                                            "O"
                                        },
                                    ) {
                                        change_pattern(
                                            &mut context,
                                            &synchronisation_controller,
                                            &sequencer,
                                            i as usize,
                                        );
                                    }
                                });
                            }
                        },
                    );

                    Group::new(hash!("CARD TRACK gGROUP"), Vec2::new(580., 40.)).ui(ui, |ui| {
                        Group::new(hash!("Card LEft"), Vec2::new(40., 38.)).ui(ui, |ui| {
                            if ui.button(Vec2::new(0., 0.), "<") {
                                move_track_card(
                                    &mut context,
                                    &synchronisation_controller,
                                    &sequencer,
                                    -1,
                                );
                            }
                        });

                        Group::new(hash!("CARD RIGHT"), Vec2::new(40., 38.)).ui(ui, |ui| {
                            if ui.button(Vec2::new(0., 0.), ">") {
                                move_track_card(
                                    &mut context,
                                    &synchronisation_controller,
                                    &sequencer,
                                    1,
                                );
                            }
                        });

                        Group::new(hash!("CARD DELETE"), Vec2::new(40., 38.)).ui(ui, |ui| {
                            if ui.button(Vec2::new(0., 0.), "-") {
                                delete_track_card(
                                    &mut context,
                                    &synchronisation_controller,
                                    &sequencer,
                                );
                            }
                        });

                        Group::new(hash!("CARD ADD"), Vec2::new(40., 38.)).ui(ui, |ui| {
                            if ui.button(Vec2::new(0., 0.), "+") {
                                add_track_card(
                                    &mut context,
                                    &synchronisation_controller,
                                    &sequencer,
                                );
                            }
                        });

                        Group::new(hash!("CARD MAX SIZE"), Vec2::new(60., 38.)).ui(ui, |ui| {
                            ui.label(
                                Vec2::new(0., 0.),
                                &("MAX: ".to_owned()
                                    + &(sequencer.tracks[context.current_track as usize].patterns
                                        [0]
                                    .steps
                                    .len()
                                        / 16)
                                        .to_string()),
                            );
                        });

                        Group::new(hash!("CARD CURENT ITER"), Vec2::new(60., 38.)).ui(ui, |ui| {
                            ui.label(
                                Vec2::new(0., 0.),
                                &("CUR: ".to_owned()
                                    + &(context.current_track_card + 1).to_string()),
                            );
                        });
                    });
                },
            );

            root_ui().window(
                hash!("MainWindow2"),
                vec2(context.track_choice_panel_w, context.title_banner_h + 150.),
                vec2(610., 150.),
                |ui| {
                    if context.current_track != -1 {
                        Group::new(hash!("Panel guziorów"), Vec2::new(590., 130.)).ui(ui, |ui| {
                            //przejdz przez 16 ziomali w current
                            for j in 0..16 {
                                let i = j + (16 * context.current_track_card as usize);
                                Group::new(hash!("Tracks", i), Vec2::new(70., 60.)).ui(ui, |ui| {
                                    if context.selected_step == i as i32 {
                                        //Check for note select (yellow)
                                        ui.push_skin(&note_selected_skin_clone);
                                    } else if context.current_step_play == i as i32 {
                                        //check for note playing (green)
                                        ui.push_skin(&note_playing_skin_clone);
                                    } else if context.current_note_highlighted == i as i32 {
                                        //Check for note highlights (lighter colours)
                                        if sequencer.tracks[context.current_track as usize].patterns
                                            [context.current_pattern]
                                            .steps[i]
                                            .is_some()
                                        {
                                            ui.push_skin(&note_placed_highlighted_skin_clone);
                                        } else {
                                            ui.push_skin(&empty_note_highlighted_skin_clone);
                                        }
                                    } else if sequencer.tracks[context.current_track as usize]
                                        .patterns[context.current_pattern]
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
                                        deselect_step(sequencer, &mut context);
                                        if sequencer.tracks[context.current_track as usize].patterns
                                            [context.current_pattern]
                                            .steps[i]
                                            .is_some()
                                        {
                                            //EDIT MODE:
                                            if context.is_edit_note_pressed {
                                                select_step(sequencer, &mut context, i as i32);
                                            } else {
                                                synchronisation_controller.lock().unwrap().mutate(
                                                    SequencerMutation::RemoveStep(
                                                        context.current_track as usize,
                                                        context.current_pattern,
                                                        i,
                                                    ),
                                                )
                                                // sequencer.tracks[0].patterns[0].steps[i] = None;
                                            }
                                        } else {
                                            synchronisation_controller.lock().unwrap().mutate(
                                                SequencerMutation::CreateStep(
                                                    context.current_track as usize,
                                                    context.current_pattern,
                                                    i,
                                                ),
                                            )
                                            // sequencer.tracks[0].patterns[0].steps[i] = Some(Step::default());
                                        }
                                    }

                                    ui.pop_skin();
                                });
                            }
                        });
                    }
                },
            );

            //TRACK CHOICE PANEL
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
                            if ui.button(Vec2::new(10., 0.), (i + 1).to_string()) {
                                //TODO - dodać warunek że track nie jest zalockowany przez innego uzytkownika!!!
                                change_track(&mut context, sequencer, i as i32);
                                //SKOKTU
                            }

                            let is_silenced = sequencer.tracks[i as usize].silenced;

                            if is_silenced {
                                if ui.button(Vec2::new(30., 0.), "Unmute") {
                                    unsilence_track(
                                        sequencer,
                                        &mut context,
                                        &mut synchronisation_controller,
                                        i as usize,
                                    );
                                }
                            } else if ui.button(Vec2::new(30., 0.), "Mute") {
                                silence_track(
                                    sequencer,
                                    &mut context,
                                    &mut synchronisation_controller,
                                    i as usize,
                                );
                            }
                        });
                    }
                },
            );

            /*
            //USER PANEL
            //Displays current users in the jam session, their nick with the corresponding colour.
            root_ui().window(
                hash!((screen_width() * 10f32) as i32),
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
            */

            //SETTINGS/EDIT PANEL
            //This panel allows user to edit parameters of currently selected note.
            root_ui().window(
                hash!("Settings"),
                vec2(0., context.title_banner_h + context.track_panel_h),
                vec2(
                    600. + context.track_choice_panel_w + 10.0,
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

                            //Current sample name:
                            let sample_num = if context.selected_step != -1 {
                                let _temp = sequencer.tracks[context.current_track as usize]
                                    .patterns[context.current_pattern]
                                    .steps[context.selected_step as usize]
                                    .as_ref()
                                    .unwrap()
                                    .parameters[Parameter::Sample as usize];

                                let mut param_val = 0;

                                if let Some(x) = _temp {
                                    param_val = x;
                                }

                                param_val as usize
                            } else {
                                let sample_num = sequencer.tracks[context.current_track as usize]
                                    .default_parameters
                                    .parameters[Parameter::Sample as usize];

                                sample_num as usize
                            };

                            let sample_name = if sample_num >= sample_provider.samples.len() {
                                "Invalid!"
                            } else {
                                sample_provider.samples[sample_num].name.as_str()
                            };

                            Group::new(hash!("ASGADGXXZXCZSSCBHRAZEE"), Vec2::new(200., 40.)).ui(
                                ui,
                                |ui| {
                                    ui.label(Vec2::new(0., 0.), "CURRENT SAMPLE NAME:");
                                    ui.label(Vec2::new(0., 20.), sample_name);
                                },
                            );
                        },
                    );

                    //STEPS LOGIC!!!
                    //Utwórz  slidery do edycji parametrów:
                    if context.selected_step != -1 {
                        for i in 0..sequencer.tracks[context.current_track as usize].patterns
                            [context.current_pattern]
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
                                [context.current_pattern]
                                .steps[context.selected_step as usize]
                                .as_ref()
                                .unwrap()
                                .parameters[i];

                            let mut is_param = false;
                            //let mut param_val = 0;

                            if let Some(_x) = _temp {
                                is_param = true;
                                //param_val = x;
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
                                                if i as i32 % 8 == context.current_slider {
                                                    "O"
                                                } else if is_param {
                                                    "X"
                                                } else {
                                                    "."
                                                },
                                            ) {
                                                if is_param {
                                                    //switch is_param
                                                    is_param = false;

                                                    //Delete parameter
                                                    delete_param(
                                                        &mut context,
                                                        &mut synchronisation_controller,
                                                        i as usize,
                                                    );
                                                } else {
                                                    //switch is_param
                                                    is_param = true;

                                                    create_param(
                                                        &mut context,
                                                        &mut synchronisation_controller,
                                                        i as usize,
                                                    );
                                                    //Add parameter
                                                }
                                            }
                                        },
                                    );

                                    Group::new(hash!("Group Slider", i), Vec2::new(500., 38.)).ui(
                                        ui,
                                        |ui| {
                                            if is_param {
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
                                        let string_1 =
                                            &("<O>  ".to_owned() + &parameter.to_string());
                                        let string_2 = &parameter.to_string();

                                        ui.label(
                                            Vec2::new(0., 0.),
                                            if context.current_slider == i as i32 % 8 {
                                                string_1
                                            } else {
                                                string_2
                                            },
                                        );
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
