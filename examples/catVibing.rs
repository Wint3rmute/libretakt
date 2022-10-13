use std::string;

use macroquad::prelude::*;

#[macroquad::main("BasicShapes")]
async fn main() {
    let test_texture = Texture2D::empty();

    let mut cat_frames: [Texture2D; 10] = [test_texture; 10];

    for i in 0..10 {
        // let pathName = &("../examples/catVibingFolder/cat_frame-".to_owned()
        //   + &stringify!(i).to_owned()
        // + &".png".to_owned());
        //cat_frames[i] = Texture2D::from_file_with_format(pathName.as_bytes(), None);
        let a = format!("./examples/catVibingFolder/cat_frame-0.png");
        println!("{a}");
        cat_frames[i] = load_texture(a.as_bytes()).await.unwrap();
    }

    cat_frames[0] = Texture2D::from_file_with_format(
        include_bytes!("../examples/catVibingFolder/cat_frame-0.png"),
        None,
    );

    loop {
        clear_background(LIGHTGRAY);

        draw_texture(cat_frames[0], 100.0, 100.0, WHITE);
        draw_text("HELLO", 20.0, 20.0, 30.0, DARKGRAY);

        next_frame().await
    }
}
