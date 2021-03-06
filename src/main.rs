extern crate find_folder;
extern crate fps_counter;
extern crate image as im;
extern crate piston_window;

use fps_counter::*;
use piston_window::*;
use rand::Rng;
use std::sync::mpsc;
use std::thread;
use std::time::Instant;

const WIDTH: usize = 160;
const HEIGHT: usize = 160;
const SIZE: usize = 5;
const WW: usize = WIDTH - 1;
const HH: usize = HEIGHT - 1;

const STARTING_FPS: u64 = 60;

struct Settings {
    density: u64,
    fps: u64,
}

impl Settings {
    const MAX_FPS: u64 = 240;
    const MIN_FPS: u64 = 1;
    pub fn new(density: u64, fps: u64) -> Self {
        Settings { density, fps }
    }
    pub fn increase_fps(&mut self) {
        self.fps = if self.fps >= Settings::MAX_FPS {
            Settings::MAX_FPS
        } else {
            self.fps + 1
        };
    }
    pub fn decrease_fps(&mut self) {
        self.fps = if self.fps <= Settings::MIN_FPS {
            Settings::MIN_FPS
        } else {
            self.fps - 1
        };
    }
}

struct State {
    x: usize,
    y: usize,
    is_alive: bool,
}

impl State {
    pub fn new(x: usize, y: usize, is_alive: bool) -> Self {
        State { x, y, is_alive }
    }
}

fn main() {
    let help_text = [
        "\"Space\" to pause",
        "\"Esc\" or \"q\" to quit",
        "\"r\" to reset",
        "\"c\" to clear",
        "\"-\" to reduce speed",
        "\"+\" to increase speed",
        "Click on a square to toggle it",
    ];
    let mut settings: Settings = Settings::new(50, STARTING_FPS);
    let opengl = OpenGL::V3_2;
    let mut window: PistonWindow = WindowSettings::new("qr", [800; 2])
        .exit_on_esc(true)
        .graphics_api(opengl)
        .build()
        .unwrap();

    let assets = find_folder::Search::ParentsThenKids(3, 3)
        .for_folder("assets")
        .unwrap();
    let mut glyphs = window
        .load_font(assets.join("FiraSans-Regular.ttf"))
        .unwrap();

    let mut events = Events::new(EventSettings::new());
    events.set_max_fps(settings.fps);

    let mut cells = generate_random(settings.density);
    let mut new_cells = cells.clone();
    let red = im::Rgba([255, 0, 0, 150]);
    let mut darkness =
        im::ImageBuffer::new(WIDTH as u32 * SIZE as u32, HEIGHT as u32 * SIZE as u32);
    darkness.fill(0);
    let mut paused = false;
    let mut frame = 0;
    let mut fps_counter = FPSCounter::default();
    let mut show_text = true;
    let mut is_mouse_down = false;
    let mut canvas = im::ImageBuffer::new(WIDTH as u32 * SIZE as u32, HEIGHT as u32 * SIZE as u32);
    let mut texture_context = TextureContext {
        factory: window.factory.clone(),
        encoder: window.factory.create_command_buffer().into(),
    };
    let mut texture: G2dTexture =
        Texture::from_image(&mut texture_context, &canvas, &TextureSettings::new()).unwrap();

    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.button_args() {
            if args.state == ButtonState::Press {
                println!("Button pressed");
                is_mouse_down = true;
            } else if args.state == ButtonState::Release {
                println!("Button released");
                is_mouse_down = false;
            }
        }

        if let Some(xy) = e.mouse_cursor_args() {
            let cx = (xy[0] / SIZE as f64).floor();
            let cy = (xy[1] / SIZE as f64).floor();
            println!("coords {}, {} | cell {}, {}", xy[0], xy[1], cx, cy);
            if is_mouse_down && cx < WIDTH as f64 && cy < HEIGHT as f64 {
                cells[cy as usize][cx as usize] = true;
            }
        }

        if let Some(Button::Keyboard(key)) = e.press_args() {
            if key == Key::R {
                cells = generate_random(settings.density);
                new_cells = cells.clone();
                frame = 0;
            }
            if key == Key::Space {
                paused = !paused;
            }
            if key == Key::Q {
                return;
            }
            if key == Key::C {
                cells = [[false; WIDTH]; HEIGHT];
            }
            if key == Key::Plus || key == Key::Equals {
                settings.increase_fps();
                events.set_max_fps(settings.fps);
            }
            if key == Key::Minus {
                settings.decrease_fps();
                events.set_max_fps(settings.fps);
            }
        }

        window.draw_2d(&e, |c, g, device| {
            let (tx, rx) = mpsc::channel();
            clear([0.0; 4], g);
            canvas.clone_from(&darkness);

            let frame_text_trans = c.transform.trans(5.0, 10.0);
            let fps: usize = fps_counter.tick();
            let fps_text = format!(
                "target_fps {} | actual_fps {} | frame {}",
                settings.fps, fps, frame
            );
            text::Text::new_color([0.8, 0.8, 0.8, 1.0], 10)
                .draw(
                    &fps_text.to_string(),
                    &mut glyphs,
                    &c.draw_state,
                    frame_text_trans,
                    g,
                )
                .unwrap();

            if show_text == true {
                let mut text_index = 1.0;
                let starting_y = HH / 2;
                for text in help_text.iter() {
                    let transform = c
                        .transform
                        .trans((WW as f64) / 2.0, (20.0 * text_index) + starting_y as f64);
                    text::Text::new_color([0.8, 0.8, 0.8, 1.0], 18)
                        .draw(text, &mut glyphs, &c.draw_state, transform, g)
                        .unwrap();
                    text_index += 1.0;
                }
                if frame > 500 {
                    show_text = false;
                }
            }

            for h in 0..HEIGHT {
                let tx1 = tx.clone();
                thread::spawn(move || {
                    for w in 0..WIDTH {
                        let new_state = State::new(w, h, determine_next_state(cells, w, h));
                        tx1.send(new_state).expect("Unable to send state");
                    }
                });
            }

            let mut received_count = 0;
            for _received in rx {
                received_count += 1;
                if _received.is_alive {
                    let x = _received.x as u32 * SIZE as u32;
                    let y = _received.y as u32 * SIZE as u32;
                    for cell_x in 1..SIZE - 1 {
                        for cell_y in 1..SIZE - 1 {
                            let xx = x + cell_x as u32 + 1;
                            let yy = y + cell_y as u32 + 1;
                            canvas.put_pixel(xx, yy, red);
                        }
                    }
                }
                new_cells[_received.y][_received.x] = _received.is_alive;
                if received_count == WIDTH * HEIGHT {
                    break;
                }
            }

            image(&texture, c.transform, g);
            texture.update(&mut texture_context, &canvas).unwrap();
            texture_context.encoder.flush(device);
            glyphs.factory.encoder.flush(device);
        });

        if !paused {
            cells = new_cells.clone();
            frame += 1;
        }
    }
}

fn generate_random(density: u64) -> [[bool; WIDTH]; HEIGHT] {
    let mut cells: [[bool; WIDTH]; HEIGHT] = [[true; WIDTH]; HEIGHT];
    for i in 0..HEIGHT {
        for j in 0..WIDTH {
            let num = rand::thread_rng().gen_range(0..100);
            if num <= density {
                cells[i][j] = true
            } else {
                cells[i][j] = false
            }
        }
    }
    return cells;
}

fn determine_next_state(cells: [[bool; WIDTH]; HEIGHT], w: usize, h: usize) -> bool {
    let up = if h > 0 { cells[h - 1][w] } else { cells[HH][w] };
    let right = if w < WW { cells[h][w + 1] } else { cells[h][0] };
    let left = if w > 0 { cells[h][w - 1] } else { cells[h][WW] };
    let down = if h < HH { cells[h + 1][w] } else { cells[0][w] };

    let up_left = if h > 0 && w > 0 {
        cells[h - 1][w - 1]
    } else if w > 0 {
        cells[HH][w - 1]
    } else if h > 0 {
        cells[h - 1][WW]
    } else {
        cells[HH][WW]
    };

    let up_right = if h > 0 && w < WW {
        cells[h - 1][w + 1]
    } else if w < WW {
        cells[HH][w + 1]
    } else if h > 0 {
        cells[h - 1][0]
    } else {
        cells[HH][0]
    };

    let down_left = if h < HH && w > 0 {
        cells[h + 1][w - 1]
    } else if w > 0 {
        cells[0][w - 1]
    } else if h < HH {
        cells[h + 1][WW]
    } else {
        cells[0][WW]
    };

    let down_right = if h < HH && w < WW {
        cells[h + 1][w + 1]
    } else if w < WW {
        cells[0][w + 1]
    } else if h < HH {
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

    if alive == 3 {
        return true;
    }
    if alive > 3 || alive < 2 {
        return false;
    }
    cells[h][w]
}
