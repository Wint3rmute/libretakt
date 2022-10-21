use libretakt::engine::{Engine, Voice};
use libretakt::sample_provider::SampleProvider;
use libretakt::sequencer::{Parameter, Sequencer, SequencerMutation, SynchronisationController};
use macroquad::prelude::*;

use macroquad::ui::{
    hash, root_ui,
    widgets::Group,
    Drag, Ui, Skin
};
use rodio::{OutputStream, Sink};
use std::sync::Arc;

//Most of those traits and structs might be deleted but im too lazy  right now to figure it out
//which might be usefull in the future... 
pub trait Interactive{
    fn checkBounds(&self);
}

pub trait Draw{
    fn draw(&self);
}

pub struct NULL{

}
impl Interactive for NULL{
    fn checkBounds(&self){

    }
}
impl Draw for NULL{
    fn draw(&self){

    }
}

pub struct Context<I: Interactive, D: Draw>{
    //jakieś gówno które kiedyś usune. Teraz jest do flexowania się żę użyłem generics
    pub interactives: Vec<I>,
    pub drawable: Vec<D>,

    //(temporary) variables for UI windows dimensions
    pub trackChoicePanel_w : f32,
    pub userPanel_w : f32,
    pub trackPanel_h : f32,
    pub titleBanner_h : f32,

    //Sampler state variables
    pub currentTrack : i32,
    pub currentStepPlay : i32,
    pub selectedStep: i32,

    //Keyboard varbiales
    pub isEditNotePressed: bool,
}

impl<I, D> Context<I, D>
    where I: Interactive, D: Draw {
        //Tu jest jakieś gówno które kiedyś usunę
        pub fn checkAllBounds(&self){
            for element in self.interactives.iter() {
                element.checkBounds();
            }
        }
        pub fn draw(&self){
            
        }

        //Tu są cenne rzeczy
        //jakieś to zjebane to zrobie metode obok xd.
        
        fn readUserInput(&mut self){
            self.isEditNotePressed = false;
            
            if is_key_down(KeyCode::LeftControl){
                self.isEditNotePressed =true;
            }
        }
        
}

pub struct Window{
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

impl Window{
    pub fn draw(&mut self){
        draw_rectangle_lines(self.x, self.y, self.w, self.h, 
        10.0,
        BLACK);
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

pub struct Button{
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    name: String,
}

impl Button{
    pub fn interact(&mut self) {
        //self.fun.invoke();
    }
}

impl Interactive for Button{
    fn checkBounds(&self) {
        let (x, y) = macroquad::input::mouse_position();
        if x >= self.x && x < self.x + self.w && y > self.y && y < self.y + self.h{
            println!("{}",self.name);
        }
    }
}

impl Draw for Button{
    fn draw(&self){
        draw_rectangle_lines(self.x, self.y, self.w, self.h, 
            10.0,
            BLACK);
    }
}

#[macroquad::main("LibreTakt")]
async fn main() {

    //***UI Skins***
    //There is probably way to edit ui elements properties more efficiently but
    //im too stupid to figure it out from documentation and i found examples
    //of doing it so this way uwu
    let titleSkin = {
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

    let emptyNoteSkin = {
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

    let notePlacedSkin = {
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

    let notePlayingSkin = {
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

    let noteSelectedSkin = {
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
    let default_skin = root_ui().default_skin().clone();
    let mut titleBanner_skin = titleSkin.clone();
    let mut noteEmpty_skin = emptyNoteSkin.clone();
    let mut notePlaced_skin = notePlacedSkin.clone();
    let mut notePlaying_skin = notePlayingSkin.clone();
    let mut noteSelected_skin = noteSelectedSkin.clone();

    //***SAMPLER***
    //To be honest i haven't been looking at this code yet but Bączek wrote it
    //so i guess its something important and i trust him. 
    let provider = Arc::new(sample_provider::SampleProvider::default());

    let voice = Voice::new(&provider);
    let engine = Engine {
        sequencer: Sequencer::new(synchronisation_controller.register_new()),
        voices: vec![voice],
    };

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    sink.append(engine);
    sink.play();

    let sequencer = Sequencer::new(synchronisation_controller.register_new());

    ui_main(sequencer, synchronisation_controller).await;
}

async fn ui_main(
    mut sequencer: Sequencer,
    mut synchronisation_controller: SynchronisationController,
) {
    let mut sample = 0.0;

    //Building Context
    //This struck will change but im too lazy to fix it right now
    let mut context = Context{
        interactives: vec![
            Button{
                x: 100.0,
                y: 200.0,
                w: 200.0,
                h: 100.0,
                name: "BUTTON C".to_string(),
            },
        ],
        drawable: vec![
            NULL{},
        ],
        trackChoicePanel_w: 100.,
        userPanel_w: 100.,
        trackPanel_h: 300.,
        titleBanner_h: 60.,

        currentTrack: -1,
        currentStepPlay: 0,
        selectedStep: -1,

        isEditNotePressed: false,

    };
    context.interactives.pop();
    context.drawable.pop();

    loop {
        clear_background(WHITE);

        {
            //Assigning main variables from sequencer. 
            //Not sure if they should be assinged to some context or exist freely this way
            let mut sequencer = sequencer.write().unwrap();
            let num_of_steps = sequencer.tracks[0].steps.len();
            sequencer.tracks[0].default_parameters.parameters[Parameters::Sample as usize] =
                sample as u8;
            context.currentStepPlay = sequencer.tracks[0].current_step as i32;
            
            //READ USER INPUT
            context.readUserInput();

            //***DRAWING UI PANELS***
            //***TITLE PANEL***
            //This panel is purely for aesthetic reason and shows the title of
            //app in fancy way (hopefully in the future...)
            root_ui().push_skin(&titleBanner_skin);
            root_ui().window(hash!(), vec2(0., 0.), vec2(screen_width(), context.titleBanner_h), |ui| {
                ui.label(Vec2::new(0.,0.), 
                "TURBO SAMPLER"
                );  
            });
            root_ui().pop_skin();
            
            //***MAIN TRACK PANEL***
            //This panel shows the track currently selected by user. 
            //Clicking displayed notes allows user to edit their sound. 
            root_ui().window(hash!(), vec2(context.trackChoicePanel_w, context.titleBanner_h), 
            vec2(screen_width() - context.trackChoicePanel_w - context.userPanel_w, 
            context.trackPanel_h), |ui| {
                Group::new(hash!(), Vec2::new(screen_width() - 210., 40.)).ui(ui, |ui| {
                    
                    if context.currentTrack != -1{
                        ui.label(Vec2::new(0.,0.), 
                        &(("TRACK #".to_owned() + &(context.currentTrack + 1).to_string()))
                        );  
                    }else{
                        ui.label(Vec2::new(0.,0.), 
                        "SELECT TRACK"
                        );  
                    }  
                    ui.label(Vec2::new(100.,0.), 
                        &context.currentStepPlay.to_string()
                        );  
                });

                if context.currentTrack != -1 {
                    for i in 0..num_of_steps {
                        Group::new(hash!(), Vec2::new(70., 60.)).ui(ui, |ui| {
                            if context.selectedStep == i as i32{
                                ui.push_skin(&noteSelected_skin);
                            }else if context.currentStepPlay == i as i32{
                                ui.push_skin(&notePlaying_skin);
                            }else if sequencer.tracks[0].steps[i].is_some(){
                                ui.push_skin(&notePlaced_skin);
                            }else{
                                ui.push_skin(&noteEmpty_skin);
                            }
                            
                            if ui.button(Vec2::new(0., 0.), 
                            "....",) {
                                
                                //im not sure if this kind of if/else chain is valid
                                //i would use some "returns" and tide it up a bit but i think i cant coz its not a method
                                context.selectedStep = -1;
                                if sequencer.tracks[0].steps[i].is_some(){
                                    //EDIT MODE:
                                    if context.isEditNotePressed{
                                        context.selectedStep = i as i32;
                                    }else{
                                        sequencer.tracks[0].steps[i] = None;
                                    }
                                }else{
                                    sequencer.tracks[0].steps[i] = Some(sequencer::Step::default());
                                }
                            }
                            
                            ui.pop_skin();
                        });
                    }
                }
            });

           
            //***TRACK CHOICE PANEL***
            //This panel lists all available tracks. Clicking on one of them shows its content
            //on the main Panel. 
            //Todo: Tracks in use have different colors. They can not be selected by user. 
            root_ui().window(hash!(), vec2(0., context.titleBanner_h), vec2(context.trackChoicePanel_w, context.trackPanel_h),|ui| {
                Group::new(hash!(), Vec2::new(90., 20.)).ui(ui, |ui| {
                    ui.label(Vec2::new(0.,0.), "TRACKS");    
                });
                
                for i in 0..sequencer.tracks.len(){
                        Group::new(hash!(), Vec2::new(90., 30.)).ui(ui, |ui| {
                            if ui.button(Vec2::new(30., 0.), (i+1).to_string()) {
                                //TODO - dodać warunek że track nie jest zalockowany przez innego uzytkownika!!!
                                context.currentTrack = i as i32;
                            }
                        });
                }
            });

            //***USER PANEL***
            //Displays current users in the jam session, their nick with the corresponding colour. 
            root_ui().window(hash!(), vec2(screen_width() - context.userPanel_w, context.titleBanner_h), vec2(context.userPanel_w, context.trackPanel_h), |ui| {
                Group::new(hash!(), Vec2::new(90., 20.)).ui(ui, |ui| {
                    ui.label(Vec2::new(0.,0.), "USERS");    
                });  
            });

            //***SETTINGS/EDIT PANEL***
            //This panel allows user to edit parameters of currently selected note. 
            root_ui().window(hash!(), vec2(0., context.titleBanner_h + context.trackPanel_h), vec2(screen_width(), screen_height() - context.titleBanner_h - context.trackPanel_h), |ui| {
                Group::new(hash!(), Vec2::new(200., 20.)).ui(ui, |ui| {
                    if context.selectedStep != -1 {
                        ui.label(Vec2::new(0.,0.), 
                        &(("SELECTED STEP #".to_owned() + &(context.selectedStep + 1).to_string()))); 
                    }else{
                        ui.label(Vec2::new(0.,0.), 
                    "NO STEP SELECTED"); 
                    }   
                });  
            });
            

            //Some leftover code that i decided to comment if i ever need to quickly look how to make sliders.
            //WILL BE DELETED LATER!!!

            // main_window.draw();
            // context.checkAllBounds();
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
