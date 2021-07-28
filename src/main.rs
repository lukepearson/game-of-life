extern crate find_folder;
extern crate piston_window;

use piston_window::*;

const WIDTH: usize = 160;
const HEIGHT: usize = 160;

fn main() {
    let help_text = ["\"Space\" to pause", "\"Esc\" to quit", "\"r\" to reset"];
    let mut fps = 10;

    let opengl = OpenGL::V3_2;
    let mut window: PistonWindow = WindowSettings::new("qr", [800; 2])
        .exit_on_esc(true)
        .graphics_api(opengl)
        .build()
        .unwrap();

    let assets = find_folder::Search::ParentsThenKids(3, 3)
        .for_folder("assets")
        .unwrap();
    println!("{:?}", assets);
    let mut glyphs = window
        .load_font(assets.join("FiraSans-Regular.ttf"))
        .unwrap();

    let mut events = Events::new(EventSettings::new());
    events.set_max_fps(fps);

    let size = 5;
    let ww = WIDTH - 1;
    let hh = HEIGHT - 1;

    let mut cells = generate_random();
    let mut prev: [[bool; WIDTH]; HEIGHT] = [[true; WIDTH]; HEIGHT];

    let mut new_cells = cells.clone();
    let rect = math::margin_rectangle([0.0, 0.0, size as f64, size as f64], 1.0);
    let red = [1.0, 0.0, 0.0, 0.4];
    let mut restart = false;
    let mut paused = false;
    let mut frame = 0;
    let mut show_text = true;

    while let Some(e) = events.next(&mut window) {
        if let Some(Button::Keyboard(key)) = e.press_args() {
            if key == Key::R {
                restart = true
            }
            if key == Key::Space {
                paused = !paused;
            }
            if key == Key::Plus {
                fps += 1;
                if fps > 60 {
                    fps = 60;
                }
                events.set_max_fps(fps);
            }
            if key == Key::Minus {
                fps -= 1;
                if fps < 1 {
                    fps = 1;
                }
                events.set_max_fps(fps);
            }
            continue;
        };

        if restart == true {
            cells = generate_random();
            restart = false;
        }

        window.draw_2d(&e, |c, g, device| {
            clear([0.0; 4], g);

            let frame_text_trans = c.transform.trans(5.0, 10.0);
            text::Text::new_color([0.8, 0.8, 0.8, 1.0], 10)
                .draw(
                    &frame.to_string(),
                    &mut glyphs,
                    &c.draw_state,
                    frame_text_trans,
                    g,
                )
                .unwrap();

            if show_text == true {
                let mut text_index = 1.0;
                let starting_y = hh / 2;
                for text in help_text.iter() {
                    let transform = c
                        .transform
                        .trans((ww as f64) / 2.0, (20.0 * text_index) + starting_y as f64);
                    text::Text::new_color([0.8, 0.8, 0.8, 1.0], 18)
                        .draw(text, &mut glyphs, &c.draw_state, transform, g)
                        .unwrap();
                    text_index += 1.0;
                }
                if frame > 100 {
                    show_text = false;
                }
            }

            for h in 0..HEIGHT {
                for w in 0..WIDTH {
                    let c = c.trans(w as f64 * size as f64, h as f64 * size as f64);
                    if cells[h][w] == true {
                        rectangle(red, rect, c.transform, g);
                    }

                    let up = if h > 0 { cells[h - 1][w] } else { cells[hh][w] };
                    let right = if w < ww { cells[h][w + 1] } else { cells[h][0] };
                    let left = if w > 0 { cells[h][w - 1] } else { cells[h][ww] };
                    let down = if h < hh { cells[h + 1][w] } else { cells[0][w] };

                    let up_left = if h > 0 && w > 0 {
                        cells[h - 1][w - 1]
                    } else if w > 0 {
                        cells[hh][w - 1]
                    } else if h > 0 {
                        cells[h - 1][ww]
                    } else {
                        cells[hh][ww]
                    };

                    let up_right = if h > 0 && w < ww {
                        cells[h - 1][w + 1]
                    } else if w < ww {
                        cells[hh][w + 1]
                    } else if h > 0 {
                        cells[h - 1][0]
                    } else {
                        cells[hh][0]
                    };

                    let down_left = if h < hh && w > 0 {
                        cells[h + 1][w - 1]
                    } else if w > 0 {
                        cells[0][w - 1]
                    } else if h < hh {
                        cells[h + 1][ww]
                    } else {
                        cells[0][ww]
                    };

                    let down_right = if h < hh && w < ww {
                        cells[h + 1][w + 1]
                    } else if w < ww {
                        cells[0][w + 1]
                    } else if h < hh {
                        cells[h + 1][0]
                    } else {
                        cells[0][0]
                    };

                    let mut alive = 0;
                    for direction in &[
                        up, right, left, down, up_left, up_right, down_left, down_right,
                    ] {
                        if direction == &true {
                            alive += 1;
                        }
                    }

                    let mut will_live = cells[h][w];
                    if alive == 3 {
                        will_live = true;
                    }
                    if alive > 3 {
                        will_live = false;
                    }
                    if alive < 2 {
                        will_live = false;
                    }

                    new_cells[h][w] = will_live;
                }
            }

            glyphs.factory.encoder.flush(device);

            if prev == new_cells {
                println!("Ended after {} frames", frame);
                restart = true;
                frame = 0;
            }
            if !paused {
                prev = cells.clone();
                cells = new_cells.clone();
                frame += 1;
            }
        });
    }
}

fn generate_random() -> [[bool; WIDTH]; HEIGHT] {
    let mut cells: [[bool; WIDTH]; HEIGHT] = [[true; WIDTH]; HEIGHT];
    for i in 0..HEIGHT {
        for j in 0..WIDTH {
            cells[i][j] = rand::random::<bool>();
        }
    }
    return cells;
}
