extern crate find_folder;
extern crate fps_counter;
extern crate image as im;
extern crate piston_window;

use std::sync::{Arc, Mutex};

use pathfinding::prelude::bfs;
use piston_window::*;
use piston_window::{G2dTexture, TextureContext};
use rand::Rng;
use rayon::prelude::*;

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

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Pos(i32, i32);

impl Pos {
    fn successors(&self) -> Vec<Pos> {
        let &Pos(x, y) = self;
        vec![
            Pos(x + 1, y + 2),
            Pos(x + 1, y - 2),
            Pos(x - 1, y + 2),
            Pos(x - 1, y - 2),
            Pos(x + 2, y + 1),
            Pos(x + 2, y - 1),
            Pos(x - 2, y + 1),
            Pos(x - 2, y - 1),
        ]
    }
}

impl Settings {
    const MAX_FPS: u64 = 240;
    const MIN_FPS: u64 = 1;
    pub fn new(density: u64, fps: u64) -> Self {
        Settings { density, fps }
    }
    pub fn increase_fps(&mut self) {
        self.fps = self.fps.min(Settings::MAX_FPS).saturating_add(1);
    }
    pub fn decrease_fps(&mut self) {
        self.fps = self.fps.max(Settings::MIN_FPS).saturating_sub(1);
    }
}

fn main() {
    let mut settings = Settings::new(50, STARTING_FPS);
    let opengl = OpenGL::V3_2;
    let mut window: PistonWindow = WindowSettings::new("Conway's Game of Life", [800; 2])
        .exit_on_esc(true)
        .graphics_api(opengl)
        .build()
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
    let mut canvas = im::ImageBuffer::new(WIDTH as u32 * SIZE as u32, HEIGHT as u32 * SIZE as u32);

    let mut texture_context = TextureContext {
        factory: window.factory.clone(),
        encoder: window.factory.create_command_buffer().into(),
    };
    let mut is_mouse_down = false;
    let mut texture: G2dTexture =
        Texture::from_image(&mut texture_context, &canvas, &TextureSettings::new()).unwrap();

    let mut last_coords = [0, 0];

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
            let brush_size = 1;
            let cx = (xy[0] / SIZE as f64).floor() as i32;
            let cy = (xy[1] / SIZE as f64).floor() as i32;
            println!("coords {}, {} | cell {}, {}", xy[0], xy[1], cx, cy);

            if is_mouse_down {
                let line_points = draw_path(last_coords[0], last_coords[1], cx, cy);
                println!("{:?}", line_points);
                for (xx, yy) in line_points {
                    for x in xx ..xx + brush_size {
                        for y in yy ..yy + brush_size {
                            if x < WIDTH as i32 && y < HEIGHT as i32 {
                                cells[y as usize][x as usize] = true;
                                new_cells = cells.clone();
                            }
                        }
                    }
                }
                last_coords = [cx, cy];
            } else {
                last_coords = [0, 0]
            }
        }

        if let Some(Button::Keyboard(key)) = e.press_args() {
            if key == Key::R {
                cells = generate_random(settings.density);
                new_cells = cells.clone();
            }
            if key == Key::Space {
                paused = !paused;
            }
            if key == Key::Q {
                return;
            }
            if key == Key::C {
                cells = [[false; WIDTH]; HEIGHT];
                new_cells = cells.clone();
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
            clear([0.0; 4], g);
            canvas.clone_from(&darkness);

            if !paused {
                let cells_arc = Arc::new(cells.clone());
                new_cells.par_iter_mut().enumerate().for_each(|(h, row)| {
                    let current_cells = cells_arc.clone();
                    for (w, cell) in row.iter_mut().enumerate() {
                        *cell = determine_next_state(&current_cells, w, h);
                    }
                });
            } else {
                new_cells = cells.clone();
            }

            let canvas_arc = Arc::new(Mutex::new(canvas.clone()));
            new_cells.par_iter().enumerate().for_each(|(h, row)| {
                for (w, &is_alive) in row.iter().enumerate() {
                    if is_alive {
                        let x = w as u32 * SIZE as u32;
                        let y = h as u32 * SIZE as u32;

                        let mut canvas = canvas_arc.lock().unwrap();
                        for cell_x in 1..SIZE - 1 {
                            for cell_y in 1..SIZE - 1 {
                                let xx = x + cell_x as u32 + 1;
                                let yy = y + cell_y as u32 + 1;
                                canvas.put_pixel(xx, yy, red);
                            }
                        }
                    }
                }
            });
            let canvas = Arc::try_unwrap(canvas_arc).unwrap().into_inner().unwrap();

            image(&texture, c.transform, g);
            texture.update(&mut texture_context, &canvas).unwrap();
            texture_context.encoder.flush(device);
        });

        cells = new_cells.clone();
    }
}

fn draw_path(x1: i32, y1: i32, x2: i32, y2: i32) -> Vec<(i32, i32)> {
    if x1 == 0 && y1 == 0 {
        return vec![(x2, y2)];
    }
    let target: Pos = Pos(x2, y2);
    let result = bfs(&Pos(x1, y1), |p| p.successors(), |p| *p == target).expect("No!");
    let mut points = Vec::new();
    for point in result {
        points.push((point.0, point.1));
    }
    return points;
}

fn generate_random(density: u64) -> [[bool; WIDTH]; HEIGHT] {
    let mut cells: [[bool; WIDTH]; HEIGHT] = [[true; WIDTH]; HEIGHT];
    let mut rng = rand::thread_rng();
    for row in cells.iter_mut() {
        for cell in row.iter_mut() {
            *cell = rng.gen_range(0..100) < density;
        }
    }
    cells
}

fn determine_next_state(cells: &[[bool; WIDTH]; HEIGHT], w: usize, h: usize) -> bool {
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

    let alive = [
        up, right, left, down, up_left, up_right, down_left, down_right,
    ]
    .iter()
    .filter(|&&cell| cell)
    .count();

    alive == 3 || (cells[h][w] && alive == 2)
}
