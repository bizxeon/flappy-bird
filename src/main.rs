use rand::Rng;
use sdl2::event::Event;
use sdl2::image::LoadTexture;
use sdl2::pixels::Color;
use sdl2::rect::{Rect, Point};
use sdl2::render::{TextureCreator, Texture, Canvas};
use sdl2::video::{WindowContext, Window};

const SCREEN_X: u32 = 800;
const SCREEN_Y: u32 = 600;

#[derive(Default, Clone, Copy)]
struct BoxCollision<T: std::cmp::PartialEq + std::cmp::PartialOrd> {
    x1: T,
    y1: T,
    x2: T,
    y2: T,
}

impl<T> BoxCollision<T> where T: std::cmp::PartialEq + std::cmp::PartialOrd {
    fn check(&self, object: BoxCollision<T>) -> bool {
        if self.x1 < object.x2 && self.x2 > object.x1 && self.y1 < object.y2 && self.y2 > object.y1 {
            return true;
        }

        false
    }
}

struct Bird<'a> {
    tick_of_next_sprite: u64,
    sprite: Texture<'a>,
    sprite_index: i32,
    y: f64,
    velocity: f64,
    angle: f64,
    collision_box: BoxCollision<f64>
}

impl<'a> Bird<'a> {
    const TICKS_BETWEEN_SPITES: u64 = 1000;
    const BIRD_INITIAL_X: i32 = 50;
    const BIRD_INITIAL_Y: i32 = (SCREEN_Y / 2) as i32 - 32;
    const THRUST_VELOCITY: f64 = -200.0;
    const FALL_VELOCITY: f64 = 220.0;
    const ROTATE_SPEED: f64 = 25.0;

    fn new(tick_count: u64, texture_creator: &'a TextureCreator<WindowContext>) -> Bird<'a> {
        let sprite = match texture_creator.load_texture("data/bird.png") {
            Ok(sprite) => { sprite },
            Err(err) => {
                eprintln!("failed to load sprite 'data/bird.png', error: {}", err.to_string());
                std::process::abort();
            }
        };

        Bird {
            tick_of_next_sprite: tick_count + Self::TICKS_BETWEEN_SPITES,
            sprite: sprite,
            sprite_index: 0,
            y: Self::BIRD_INITIAL_Y as f64,
            velocity: 0.0,
            angle: 0.0,
            collision_box: BoxCollision {
                x1: Self::BIRD_INITIAL_X as f64 + 32.0,
                y1: Self::BIRD_INITIAL_Y as f64 + 32.0,
                x2: Self::BIRD_INITIAL_X as f64 + 64.0,
                y2: Self::BIRD_INITIAL_Y as f64 + 64.0,
            }
        }
    }

    fn event_update(&mut self, event: Event) {
        match event {
            sdl2::event::Event::KeyDown { keycode: Some(sdl2::keyboard::Keycode::Space), .. } => {
                self.velocity = Self::THRUST_VELOCITY;
            }
            _ => {}
        }
    }

    fn update(&mut self, tick_count: u64, delta_time: f64) {
        if tick_count > self.tick_of_next_sprite {
            self.tick_of_next_sprite = tick_count + Self::TICKS_BETWEEN_SPITES;
            self.sprite_index = self.sprite_index + 1;

            if self.sprite_index > 2 {
                self.sprite_index = 0;
            }
        }

        self.velocity = self.velocity + Self::FALL_VELOCITY * delta_time;
        self.y = self.y + self.velocity * delta_time;

        if self.y > (SCREEN_Y as f64 - 64.0) {
            self.y = SCREEN_Y as f64 - 64.0;
        }

        if self.velocity > 0.0 {
            self.angle += Self::ROTATE_SPEED * delta_time;
        } else {
            self.angle -= Self::ROTATE_SPEED * delta_time;
        }

        if self.angle <= -45.0 {
            self.angle = -45.0;
        }

        if self.angle >= 45.0 {
            self.angle = 45.0;
        }

        self.collision_box = BoxCollision {
            x1: Self::BIRD_INITIAL_X as f64 + 16.0,
            y1: self.y + 16.0,
            x2: Self::BIRD_INITIAL_X as f64 + 48.0,
            y2: self.y + 48.0,
        }
    }

    fn render(&mut self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        match self.sprite_index {
            0 => {
                canvas.copy_ex(&self.sprite, Rect::new(0, 0, 32, 32), Rect::new(Self::BIRD_INITIAL_X, self.y as i32, 64, 64), self.angle, Point::new(32, 32), false, false)?;
            },
            1 => {
                canvas.copy_ex(&self.sprite, Rect::new(32, 0, 32, 32), Rect::new(Self::BIRD_INITIAL_X, self.y as i32, 64, 64), self.angle, Point::new(32, 32), false, false)?;
            },
            2 => {
                canvas.copy_ex(&self.sprite, Rect::new(64, 0, 32, 32), Rect::new(Self::BIRD_INITIAL_X, self.y as i32, 64, 64), self.angle, Point::new(32, 32), false, false)?;
            }
            _ => {
                self.sprite_index = 0;
            }
        }

        Ok(())
    }
}

struct Pipes<'a> {
    x: f64,
    hole_y: i32,
    hole_height: i32,
    sprite: Texture<'a>,
    collision_box_top: BoxCollision<f64>,
    collision_box_bottom: BoxCollision<f64>,
    colission_box: BoxCollision<f64>,
}

impl<'a> Pipes<'a> {
    const X_VELOCITY: f64 = 200.0;

    fn new(x: f64, hole_y: i32, hole_height: i32, texture_creator: &'a TextureCreator<WindowContext>) -> Pipes<'a> {
        let sprite = match texture_creator.load_texture("data/pipe.png") {
            Ok(sprite) => { sprite },
            Err(err) => {
                eprintln!("failed to load sprite 'data/pipe.png', error: {}", err.to_string());
                std::process::abort();
            }
        };

        Pipes {
            x: x,
            hole_y: hole_y,
            hole_height: hole_height,
            sprite: sprite,
            collision_box_top: BoxCollision {
                x1: x,
                y1: 0.0,
                x2: x + 64.0,
                y2: (hole_y - hole_height) as f64,
            },
            collision_box_bottom: BoxCollision {
                x1: x,
                y1: (hole_y + hole_height) as f64,
                x2: x + 64.0,
                y2: (SCREEN_Y as i32 - hole_y + hole_height) as f64,
            },
            colission_box: BoxCollision {
                x1: x,
                y1: 0.0,
                x2: x + 64.0,
                y2: SCREEN_Y as f64,
            }
        }
    }

    fn update(&mut self, delta_time: f64) {
        self.x = self.x - Self::X_VELOCITY * delta_time;

        self.collision_box_top = BoxCollision {
            x1: self.x,
            y1: 0.0,
            x2: self.x + 64.0,
            y2: (self.hole_y - self.hole_height) as f64,
        };

        self.collision_box_bottom = BoxCollision {
            x1: self.x,
            y1: (self.hole_y + self.hole_height) as f64,
            x2: self.x + 64.0,
            y2: (SCREEN_Y) as f64,
        };

        self.colission_box = BoxCollision {
            x1: self.x,
            y1: 0.0,
            x2: self.x + 64.0,
            y2: SCREEN_Y as f64,
        };
    }

    fn render(&mut self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        let pipe_body = Rect::new(32, 0, 16, 1);
        let pipe_top_part = Rect::new(0, 0, 16, 16);
        let pipe_bottom_part = Rect::new(16, 0, 16, 16);

        // top pipe
        canvas.copy(&self.sprite, pipe_body, Rect::new(self.x as i32, 0, 64, (self.hole_y - self.hole_height) as u32))?;
        
        // bottom pipe
        canvas.copy(&self.sprite, pipe_body, Rect::new(self.x as i32, (self.hole_y + self.hole_height) as i32, 64, (SCREEN_Y as i32 - self.hole_y + self.hole_height) as u32))?;

        // top part
        canvas.copy(&self.sprite, pipe_top_part, Rect::new(self.x as i32, (self.hole_y + self.hole_height) as i32, 64, 64))?;

        // bottom part
        canvas.copy(&self.sprite, pipe_bottom_part, Rect::new(self.x as i32, (self.hole_y - self.hole_height - 64) as i32, 64, 64))?;

        Ok(())
    }
}

struct Font<'a> {
    sprite: Texture<'a>
}

impl<'a> Font<'a> {
    const FONT_WIDTH: i32 = 32;
    const FONT_HEIGHT: i32 = 32;

    fn new(texture_creator: &'a TextureCreator<WindowContext>) -> Font<'a> {
        let sprite = match texture_creator.load_texture("data/font.png") {
            Ok(sprite) => { sprite },
            Err(err) => {
                eprintln!("failed to load sprite 'data/font.png', error: {}", err.to_string());
                std::process::abort();
            }
        };

        Font {
            sprite: sprite
        }
    }

    fn render(&self, x: i32, y: i32, text: String, canvas: &mut Canvas<Window>) -> Result<(), String> {
        let text = text.to_uppercase();

        for (text_x, value) in text.as_bytes().to_vec().iter().enumerate() {
            let position = match value {
                b' ' =>  { (0  * 20, 0 * 20) },
                b'!' =>  { (1  * 20, 0 * 20) },
                b'"' =>  { (2  * 20, 0 * 20) },
                b'#' =>  { (3  * 20, 0 * 20) },
                b'$' =>  { (4  * 20, 0 * 20) },
                b'%' =>  { (5  * 20, 0 * 20) },
                b'&' =>  { (6  * 20, 0 * 20) },
                b'\'' => { (7  * 20, 0 * 20) },
                b'(' =>  { (8  * 20, 0 * 20) },
                b')' =>  { (9  * 20, 0 * 20) },
                b'*' =>  { (10 * 20, 0 * 20) },
                b'+' =>  { (11 * 20, 0 * 20) },
                b',' =>  { (12 * 20, 0 * 20) },
                b'-' =>  { (13 * 20, 0 * 20) },
                b'.' =>  { (14 * 20, 0 * 20) },

                b'/' =>  { (0  * 20, 1 * 20) },
                b'0' =>  { (1  * 20, 1 * 20) },
                b'1' =>  { (2  * 20, 1 * 20) },
                b'2' =>  { (3  * 20, 1 * 20) },
                b'3' =>  { (4  * 20, 1 * 20) },
                b'4' =>  { (5  * 20, 1 * 20) },
                b'5' =>  { (6  * 20, 1 * 20) },
                b'6' =>  { (7  * 20, 1 * 20) },
                b'7' =>  { (8  * 20, 1 * 20) },
                b'8' =>  { (9  * 20, 1 * 20) },
                b'9' =>  { (10 * 20, 1 * 20) },
                b':' =>  { (11 * 20, 1 * 20) },
                b';' =>  { (12 * 20, 1 * 20) },
                b'<' =>  { (13 * 20, 1 * 20) },
                b'=' =>  { (14 * 20, 1 * 20) },
 
                b'>' =>  { (0  * 20, 2 * 20) },
                b'?' =>  { (1  * 20, 2 * 20) },
                b'@' =>  { (2  * 20, 2 * 20) },
                b'A' =>  { (3  * 20, 2 * 20) },
                b'B' =>  { (4  * 20, 2 * 20) },
                b'C' =>  { (5  * 20, 2 * 20) },
                b'D' =>  { (6  * 20, 2 * 20) },
                b'E' =>  { (7  * 20, 2 * 20) },
                b'F' =>  { (8  * 20, 2 * 20) },
                b'G' =>  { (9  * 20, 2 * 20) },
                b'H' =>  { (10 * 20, 2 * 20) },
                b'I' =>  { (11 * 20, 2 * 20) },
                b'J' =>  { (12 * 20, 2 * 20) },
                b'K' =>  { (13 * 20, 2 * 20) },
                b'L' =>  { (14 * 20, 2 * 20) },

                b'M' =>  { (0  * 20, 3 * 20) },
                b'N' =>  { (1  * 20, 3 * 20) },
                b'O' =>  { (2  * 20, 3 * 20) },
                b'P' =>  { (3  * 20, 3 * 20) },
                b'Q' =>  { (4  * 20, 3 * 20) },
                b'R' =>  { (5  * 20, 3 * 20) },
                b'S' =>  { (6  * 20, 3 * 20) },
                b'T' =>  { (7  * 20, 3 * 20) },
                b'U' =>  { (8  * 20, 3 * 20) },
                b'V' =>  { (9  * 20, 3 * 20) },
                b'W' =>  { (10 * 20, 3 * 20) },
                b'X' =>  { (11 * 20, 3 * 20) },
                b'Y' =>  { (12 * 20, 3 * 20) },
                b'Z' =>  { (13 * 20, 3 * 20) },
                b'[' =>  { (14 * 20, 3 * 20) },

                b'\\' => { (0  * 20, 4 * 20) },
                b']' =>  { (1  * 20, 4 * 20) },
                b'^' =>  { (2  * 20, 4 * 20) },
                b'_' =>  { (3  * 20, 4 * 20) },

                _ => { (0, 0 ) }
            };

            canvas.copy(&self.sprite, Rect::new(position.0, position.1, 20, 20), Rect::new(x + (text_x * 32) as i32, y, Self::FONT_HEIGHT as u32, Self::FONT_WIDTH as u32))?;
        }

        Ok(())
    }
}

fn main() -> Result<(), String>{
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = match video_subsystem.window("Flappy Birds", SCREEN_X, SCREEN_Y).opengl().build() {
        Ok(window) => { window },
        Err(err) => {
            return Err(err.to_string());
        }
    };

    let mut canvas = match window.into_canvas().build() {
        Ok(canvas) => { canvas },
        Err(err) => {
            return Err(err.to_string());
        }
    };

    let mut tick_count: u64 = 0;
    let mut delta_time: f64 = 0.0;
    let mut event_pump = sdl_context.event_pump()?;
    let texture_creator = canvas.texture_creator();

    let mut bird = Bird::new(tick_count, &texture_creator);
    let mut pipes: Vec<Pipes> = Vec::new();
    let mut pipes_spawn_tick = 0;

    let font = Font::new(&texture_creator);
    let mut points: i32 = 0;
    let mut bird_is_in_pipe = false;
    let mut game_is_over = false;

    'running: loop {
        let time_start: std::time::Instant = std::time::Instant::now();

        // update
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } | sdl2::event::Event::KeyDown { keycode: Some(sdl2::keyboard::Keycode::Escape), .. } => {
                    break 'running;
                }
                _ => {}
            }        

            if game_is_over == false {
                bird.event_update(event);
            } else {
                match event {
                    sdl2::event::Event::KeyDown { keycode: Some(sdl2::keyboard::Keycode::Space), .. } => {
                        points = 0;
                        game_is_over = false;
                        bird_is_in_pipe = false;
                        pipes.clear();
                        bird = Bird::new(tick_count, &texture_creator);
                    }
                    _ => {}
                }    
            }
        }

        if game_is_over == false {
            bird.update(tick_count, delta_time);

            let mut pipes_to_remove: Vec<usize> = Vec::new();

            for (idx, pipe) in pipes.iter().enumerate() {
                if pipe.x < -150.0 {
                    pipes_to_remove.push(idx)
                }
            }

            for idx in pipes_to_remove.iter().rev() {
                pipes.remove(*idx);
            }

            if tick_count >= pipes_spawn_tick {
                pipes.push(Pipes::new(SCREEN_X as f64 + 100.0, rand::thread_rng().gen_range((SCREEN_Y as f64 * 0.3) as i32..(SCREEN_Y as f64 * 0.7) as i32), 50, &texture_creator));
                pipes_spawn_tick = tick_count + 30000;
            }

            for pipe in &mut pipes {
                pipe.update(delta_time);
            }

            for pipe in &mut pipes {
                if bird.collision_box.check(pipe.collision_box_top) {
                    game_is_over = true;
                } else if bird.collision_box.check(pipe.collision_box_bottom) {
                    game_is_over = true;
                }
            }

            if pipes.len() > 0 && bird.collision_box.check(pipes[0].colission_box) {
                bird_is_in_pipe = true;
            } else {
                if bird_is_in_pipe {
                    points = points + 1;
                }

                bird_is_in_pipe = false;
            }

            if points > 999 {
                points = 999;
            }
        }
        
        // render
        canvas.set_draw_color(sdl2::pixels::Color::RGB(135, 206, 250));
        canvas.clear();

        bird.render(&mut canvas)?;

        for pipe in &mut pipes {
            pipe.render(&mut canvas)?;
        }

        if game_is_over == false {
            font.render((SCREEN_X - Font::FONT_WIDTH as u32 * 4) as i32, Font::FONT_HEIGHT, points.to_string(), &mut canvas)?;
        } else {
            let text1 = String::from("GAME OVER!");
            let text2 = format!("SCORE:{}", points.to_string());

            font.render(SCREEN_X as i32 / 2 - text1.len() as i32 / 2 * Font::FONT_WIDTH, (SCREEN_Y / 2) as i32 - Font::FONT_HEIGHT, text1, &mut canvas)?;
            font.render(SCREEN_X as i32 / 2 - text2.len() as i32 / 2 * Font::FONT_WIDTH, (SCREEN_Y / 2) as i32, text2, &mut canvas)?;
        }

        canvas.present();

        delta_time = std::time::Instant::now().duration_since(time_start).as_nanos() as f64 / 1000000000.0 as f64;
        tick_count = tick_count + 1;
    }    

    Ok(())
}
