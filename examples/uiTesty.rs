use std::f64::consts::PI;

use macroquad::prelude::*;

/*
struct Context {

         stepsInTrack = 16;
         numOfTracks = 4;
        //RACK BOX:
         TRACK_BOX_X = screen_width() * 0.05;
         TRACK_BOX_Y = screen_height() * 0.05;
         TRACK_BOX_W = screen_width() * 0.9;
         TRACK_BOX_H = screen_height() * 0.4;
        //RACKS:
         TRACK_H = TRACK_BOX_H / numOfTracks as f32;
         TRACK_MARGIN = 5.0;
        //RACK STEPS:
         STEP_W = TRACK_BOX_W / stepsInTrack as f32;


}
*/
fn plot_fun(x: f32) -> f32 {
    (((x as f32 - 0.5) * 40.0) as f64 / (2.0 * PI)).sin() as f32
}

fn draw_plot_view(window_x: f32, window_y: f32, window_w: f32, window_h: f32) {
    let window_margine = 5.0;
    let plot_acuracy = 200;
    let plot_w = window_w;
    let plot_h = window_h;
    let plot_x = window_x;
    let plot_y = window_y + plot_h / 2.0;
    let plot_max_h = plot_h / 2.0;
    let plot_color = GREEN;
    let plot_line_thickness = 2.0;

    draw_rectangle_lines(
        window_x - window_margine,
        window_y - window_margine,
        plot_w + window_margine,
        plot_h + window_margine,
        2.0,
        plot_color,
    );
    draw_line(plot_x, plot_y, plot_x + plot_w, plot_y, 1.0, plot_color);

    for step in 0..plot_acuracy - 1 {
        let x1 = (step as f32) / plot_acuracy as f32 * plot_w;
        let x2 = ((step as f32) + 1.0) / plot_acuracy as f32 * plot_w;
        let fx1 = step as f32 / plot_acuracy as f32;
        let fx2 = (step + 1) as f32 / plot_acuracy as f32;
        let fy1 = plot_fun(fx1);
        let fy2 = plot_fun(fx2);
        let y1 = fy1 * plot_max_h;
        let y2 = fy2 * plot_max_h;
        draw_line(
            plot_x + x1,
            plot_y - y1,
            plot_x + x2,
            plot_y - y2,
            plot_line_thickness,
            plot_color,
        );
    }
    /*

    */
}

#[macroquad::main("BasicShapes")]

async fn main() {
    let mut mouseX = 100.0;
    let mut mouseY = 100.0;
    loop {
        /*
        clear_background(LIGHTGRAY);

        //Draw Tracks
        //Draw tracks field:
        draw_rectangle_lines(
            TRACK_BOX_X,
            TRACK_BOX_Y,
            TRACK_BOX_W,
            TRACK_BOX_H,
            15.0,
            BLACK,
        );

        //Populate Track Box with Tracks:
        let tracksYOffset = TRACK_H;
        let stepXOffset = STEP_W;
        for i in 0..numOfTracks {
            let col = if i % 2 == 0 { GREEN } else { RED };

            draw_rectangle(
                TRACK_BOX_X,
                TRACK_BOX_Y + (i as f32) * tracksYOffset,
                TRACK_BOX_W,
                TRACK_H,
                col,
            );
            for j in 0..stepsInTrack {
                let stepCol = if j % 2 == 0 { BLUE } else { YELLOW };

                draw_rectangle(
                    TRACK_BOX_X + j as f32 * STEP_W,
                    TRACK_BOX_Y + (i as f32) * tracksYOffset,
                    STEP_W,
                    TRACK_H,
                    stepCol,
                );
                draw_rectangle_lines(
                    TRACK_BOX_X + j as f32 * STEP_W,
                    TRACK_BOX_Y + (i as f32) * tracksYOffset,
                    STEP_W,
                    TRACK_H,
                    10.0,
                    BLACK,
                );
            }
        }
        //
        */

        //Testowanie nawalania oknami z funkcją sinus;
        //Saghetti code przenoszący okno:
        if is_mouse_button_down(MouseButton::Left) {
            (mouseX, mouseY) = mouse_position();
        }
        draw_plot_view(mouseX, mouseY, 300.0, 200.0);

        next_frame().await
    }
}
