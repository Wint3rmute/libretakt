use libretakt::engine::{Engine, Voice};
use libretakt::sample_provider::SampleProvider;
use libretakt::sequencer::{
    CurrentStepData, Sequencer, SequencerMutation, SynchronisationController,
};
use macroquad::prelude::*;

use flume::{bounded, Receiver};
use macroquad::ui::{hash, root_ui, widgets::Group, Skin};
use rodio::{OutputStream, Sink};
use std::sync::Arc;

//Most of those traits and structs might be deleted but im too lazy  right now to figure it out
//which might be usefull in the future...
pub trait Interactive {
    fn check_bounds(&self);
}

pub trait Draw {
    fn draw(&self);
}

pub struct NULL {}
impl Interactive for NULL {
    fn check_bounds(&self) {}
}
impl Draw for NULL {
    fn draw(&self) {}
}

pub struct Context<I: Interactive, D: Draw> {
    //jakieś gówno które kiedyś usune. Teraz jest do flexowania się żę użyłem generics
    pub interactives: Vec<I>,
    pub drawable: Vec<D>,

    //(temporary) variables for UI windows dimensions
    pub track_choice_panel_w: f32,
    pub user_panel_w: f32,
    pub track_panel_h: f32,
    pub title_banner_h: f32,

    //Sampler state variables
    pub current_track: i32,
    pub current_step_play: i32,
    pub selected_step: i32,

    //Keyboard varbiales
    pub is_edit_note_pressed: bool,
}

impl<I, D> Context<I, D>
where
    I: Interactive,
    D: Draw,
{
    //Tu jest jakieś gówno które kiedyś usunę
    pub fn check_all_bounds(&self) {
        for element in self.interactives.iter() {
            element.check_bounds();
        }
    }
    pub fn draw(&self) {}

    //Tu są cenne rzeczy
    //jakieś to zjebane to zrobie metode obok xd.

    fn check_user_input(&mut self) {
        self.is_edit_note_pressed = false;

        if is_key_down(KeyCode::LeftControl) {
            self.is_edit_note_pressed = true;
        }
    }
}

pub struct Window {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

impl Window {
    pub fn draw(&mut self) {
        draw_rectangle_lines(self.x, self.y, self.w, self.h, 10.0, BLACK);
    }
}

/*
pub trait Function{
    fn invoke(&self);
}

pub struct FunA{

}
impl Function for FunA{
    fn invoke(&self){
        println!("AAAAA");
    }
}

pub struct FunB{

}
impl Function for FunB{
    fn invoke(&self){
        println!("BBBBB");
    }
}
*/

pub struct Button {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    name: String,
}

impl Button {
    pub fn interact(&mut self) {
        //self.fun.invoke();
    }
}

impl Interactive for Button {
    fn check_bounds(&self) {
        let (x, y) = macroquad::input::mouse_position();
        if x >= self.x && x < self.x + self.w && y > self.y && y < self.y + self.h {
            println!("{}", self.name);
        }
    }
}

impl Draw for Button {
    fn draw(&self) {
        draw_rectangle_lines(self.x, self.y, self.w, self.h, 10.0, BLACK);
    }
}

#[macroquad::main("LibreTakt")]
async fn main() {
    //***SAMPLER***
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
        interactives: vec![Button {
            x: 100.0,
            y: 200.0,
            w: 200.0,
            h: 100.0,
            name: "BUTTON C".to_string(),
        }],
        drawable: vec![NULL {}],
        track_choice_panel_w: 100.,
        user_panel_w: 100.,
        track_panel_h: 300.,
        title_banner_h: 60.,

        current_track: -1,
        current_step_play: 0,
        selected_step: -1,

        is_edit_note_pressed: false,
    };
    context.interactives.pop();
    context.drawable.pop();

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

            //***DRAWING UI PANELS***
            //***TITLE PANEL***
            //This panel is purely for aesthetic reason and shows the title of
            //app in fancy way (hopefully in the future...)
            root_ui().push_skin(&titlebanner_skin_clone);
            root_ui().window(
                hash!(),
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
                hash!(),
                vec2(context.track_choice_panel_w, context.title_banner_h),
                vec2(
                    screen_width() - context.track_choice_panel_w - context.user_panel_w,
                    context.track_panel_h,
                ),
                |ui| {
                    Group::new(hash!(), Vec2::new(screen_width() - 210., 40.)).ui(ui, |ui| {
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
                            Group::new(hash!(), Vec2::new(70., 60.)).ui(ui, |ui| {
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
                                            context.selected_step = i as i32;
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
                hash!(),
                vec2(0., context.title_banner_h),
                vec2(context.track_choice_panel_w, context.track_panel_h),
                |ui| {
                    Group::new(hash!(), Vec2::new(90., 20.)).ui(ui, |ui| {
                        ui.label(Vec2::new(0., 0.), "TRACKS");
                    });

                    for i in 0..sequencer.tracks.len() {
                        Group::new(hash!(), Vec2::new(90., 30.)).ui(ui, |ui| {
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
                hash!(),
                vec2(screen_width() - context.user_panel_w, context.title_banner_h),
                vec2(context.user_panel_w, context.track_panel_h),
                |ui| {
                    Group::new(hash!(), Vec2::new(90., 20.)).ui(ui, |ui| {
                        ui.label(Vec2::new(0., 0.), "USERS");
                    });
                },
            );

            //***SETTINGS/EDIT PANEL***
            //This panel allows user to edit parameters of currently selected note.
            root_ui().window(
                hash!(),
                vec2(0., context.title_banner_h + context.track_panel_h),
                vec2(
                    screen_width(),
                    screen_height() - context.title_banner_h - context.track_panel_h,
                ),
                |ui| {
                    Group::new(hash!(), Vec2::new(200., 20.)).ui(ui, |ui| {
                        if context.selected_step != -1 {
                            ui.label(
                                Vec2::new(0., 0.),
                                &("SELECTED STEP #".to_owned()
                                    + &(context.selected_step + 1).to_string()),
                            );
                        } else {
                            ui.label(Vec2::new(0., 0.), "NO STEP SELECTED");
                        }
                    });
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
