use macroquad::prelude::*;

use flume::{bounded, Receiver};

use macroquad::ui::{hash, root_ui, widgets::Group, Skin};

pub struct TitleBannerSkin{
    pub titlebanner_skin: Skin,
}

pub struct EmptyNoteSkin{
    pub empty_note_skin: Skin,
}

pub struct NotePlacedSkin{
    pub note_placed_skin: Skin,
}

pub struct NotePlayingSkin{
    pub note_playing_skin: Skin,
}

pub struct NoteSelectedSkin{
    pub note_selected_skin: Skin,
}

impl TitleBannerSkin{

    pub fn new() -> TitleBannerSkin {
        TitleBannerSkin {
            titlebanner_skin: {
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
            }
        }
    }
}

impl EmptyNoteSkin{

    pub fn new() -> EmptyNoteSkin {
        EmptyNoteSkin {
            empty_note_skin: {
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
            }
        }
    }
}

impl NotePlacedSkin{

    pub fn new() -> NotePlacedSkin {
        NotePlacedSkin {
            note_placed_skin: {
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
            }
        }
    }
}

impl NotePlayingSkin{

    pub fn new() -> NotePlayingSkin {
        NotePlayingSkin {
            note_playing_skin: {
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
            }
        }
    }
}

impl NoteSelectedSkin{

    pub fn new() -> NoteSelectedSkin {
        NoteSelectedSkin {
            note_selected_skin: {
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
            }
        }
    }
}