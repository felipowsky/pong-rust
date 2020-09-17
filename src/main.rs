use sdl2::event::{Event};
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::render::{WindowCanvas, Texture, TextureCreator, TextureQuery};
use sdl2::rect::{Point, Rect};
use sdl2::image::{self, LoadTexture};
use sdl2::mixer::{self, DEFAULT_CHANNELS, AUDIO_S16LSB, Music};
use sdl2::ttf::{self, Sdl2TtfContext, Font};
use sdl2::{Sdl, EventPump};
use std::time::Duration;
use std::cmp::{max, min};

const TARGET_FPS: u8 = 60;
const BACKGROUND_COLOR: Color = Color::RGB(0, 0, 0);
const WINDOW_SIZE: (u32, u32) = (800, 600);
const WINDOW_HALF_SIZE: (u32, u32) = (WINDOW_SIZE.0 / 2, WINDOW_SIZE.1 / 2);
const PADDLE_SIZE: (u32, u32) = (52, 150);
const PADDLE_COLLIDER_SIZE: (u32, u32) = (PADDLE_SIZE.0 - 42, PADDLE_SIZE.1 - 40);
const PADDLE_SPEED: u8 = 4;
const BALL_SIZE: (u32, u32) = (51, 51);
const BALL_RADIUS: u32 = 11;
const BALL_SPEED: u8 = 2;
const BALL_MAX_SPEED: u8 = 15;
const SPRITESHEET_FILENAME: &str = "assets/spritesheet.png";
const POP_SOUND_FILENAME: &str = "assets/pop.ogg";
const SCORE_SOUND_FILENAME: &str = "assets/score.ogg";
const FONT_FILENAME: &str = "assets/sansation.ttf";

enum BallUpdateState {
    PaddleCollision,
    Scoring,
    Moving
}

enum BallMoveState {
    WallCollision,
    Moving
}

enum HandleEventsState {
    Exit,
    Running
}

struct Sprite<'a> {
    texture: &'a Texture<'a>,
    rect: Rect
}

struct Entity<'a> {
    position: Point,
    size: (u32, u32),
    sprite: Sprite<'a>
}

struct Label<'ttf_module, 'rwops, 'tc, T> {
    text: String,
    font: &'ttf_module Font<'ttf_module, 'rwops>,
    position: Point,
    color: Color,
    width: u32,
    height: u32,
    texture_creator: &'tc TextureCreator<T>,
    cached_texture: Option<Texture<'tc>>
}

impl<'ttf_module, 'rwops, 'tc, T> Label<'ttf_module, 'rwops, 'tc, T> {
    fn new(text: String, 
        font: &'ttf_module Font<'ttf_module, 'rwops>, 
        position: Point, 
        color: Color, 
        texture_creator: &'tc TextureCreator<T>) -> Result<Self, String> {
        let mut label = Label {
            text: text,
            font: &font,
            position: position,
            color: color,
            width: 0,
            height: 0,
            texture_creator: texture_creator,
            cached_texture: None
        };
        label.create_cache()?;
        Ok(label)
    }

    fn set_text(&mut self, text: String) {
        self.text = text;
        self.clear_cache();
    }

    fn clear_cache(&mut self) {
        self.cached_texture = None
    }

    fn texture(&mut self) -> Result<&Texture<'tc>, String> {
        match self.cached_texture {
            Some(ref texture) => Ok(&texture),
            None => Ok(self.create_cache()?)
        }
    }

    fn create_texture(&self, 
        text: &str, 
        font: &Font, 
        color: &Color, 
        texture_creator: &'tc TextureCreator<T>) -> Result<Texture<'tc>, String> {
        let surface = font.render(text)
            .blended(*color).map_err(|e| e.to_string())?;
        let texture = texture_creator.create_texture_from_surface(&surface)
            .map_err(|e| e.to_string())?;
        Ok(texture)
    }

    fn create_cache(&mut self) -> Result<&Texture<'tc>, String> {
        let texture = self.create_texture(
            &self.text, 
            &self.font, 
            &self.color, 
            &self.texture_creator
        )?;
        let TextureQuery { width, height, .. } = texture.query();
        self.width = width;
        self.height = height;
        self.cached_texture = Some(texture);
        Ok(self.cached_texture.as_ref().unwrap())
    }
}

fn init() -> Result<(Sdl, Sdl2TtfContext), String> {
    let sdl_context = sdl2::init()
        .expect("Could not initialize SDL");
    let _sdl_image_context = image::init(image::InitFlag::PNG)
        .expect("Could not initialize SDL_image");
    let sdl_ttf_context = ttf::init()
        .expect("Could not initialize SDL_ttf");
    mixer::open_audio(44_100, AUDIO_S16LSB, DEFAULT_CHANNELS, 1_024)
        .expect("Could not open SDL_mixer");
    let _sdl_mixer_context = mixer::init(mixer::InitFlag::OGG)
        .expect("Could not initialize SDL_mixer");
    mixer::allocate_channels(4);
    mixer::Music::set_volume(32);
    Ok((sdl_context, sdl_ttf_context))
}

fn handle_events(event_pump: &mut EventPump, 
    paddle1_movement: &mut i32, 
    paddle2_movement: &mut i32) -> HandleEventsState {
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit {..} |
            Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                return HandleEventsState::Exit
            },
            Event::KeyDown { keycode: Some(Keycode::W), .. } => {
                *paddle1_movement = -(PADDLE_SPEED as i32);
            },
            Event::KeyDown { keycode: Some(Keycode::S), .. } => {
                *paddle1_movement = PADDLE_SPEED as i32;
            },
            Event::KeyUp { keycode: Some(Keycode::W), .. } |
            Event::KeyUp { keycode: Some(Keycode::S), .. } => {
                *paddle1_movement = 0;
            },
            Event::KeyDown { keycode: Some(Keycode::Up), .. } => {
                *paddle2_movement = -(PADDLE_SPEED as i32);
            },
            Event::KeyDown { keycode: Some(Keycode::Down), .. } => {
                *paddle2_movement = PADDLE_SPEED as i32;
            },
            Event::KeyUp { keycode: Some(Keycode::Up), .. } |
            Event::KeyUp { keycode: Some(Keycode::Down), .. } => {
                *paddle2_movement = 0;
            },
            _ => {}
        }
    }
    HandleEventsState::Running
}

fn update<'a, 'b, 'c, 'd, T>(entities: &mut Vec<Entity>, 
    ball_index: usize, 
    ball_movement: &mut Point,
    paddle1_index: usize,
    paddle1_movement: i32,
    paddle1_score: &mut u64,
    paddle1_score_label: &mut Label<'a, 'b, 'c, T>,
    paddle2_index: usize,
    paddle2_movement: i32,
    paddle2_score: &mut u64,
    paddle2_score_label: &mut Label<'a, 'b, 'c, T>,
    pop_sound: &Music,
    score_sound: &Music) -> Result<(), String> {
    match move_ball(&mut entities[ball_index], ball_movement) {
        BallMoveState::WallCollision => { pop_sound.play(1)?; },
        BallMoveState::Moving => (),
    }
    move_paddle(&mut entities[paddle1_index], paddle1_movement);
    move_paddle(&mut entities[paddle2_index], paddle2_movement);
    let paddle_index_collision = if ball_movement.x > 0 {
        paddle2_index 
    } else { 
        paddle1_index 
    };
    match update_ball_state(&entities[ball_index], 
        *ball_movement, 
        &entities[paddle_index_collision]) {
        BallUpdateState::Scoring => {
            let x_movement: i32;
            let score: &mut u64;
            let label: &mut Label<T>;
            if ball_movement.x > 0 {
                x_movement = -(BALL_SPEED as i32);
                score = paddle1_score;
                label = paddle1_score_label;
            } else {
                x_movement = BALL_SPEED as i32;
                score = paddle2_score;
                label = paddle2_score_label;
            }
            *score += 1;
            label.set_text(format!("{}", *score));
            score_sound.play(1)?;
            *ball_movement = Point::new(x_movement, BALL_SPEED as i32);
            entities[ball_index].position = Point::new(0, 0);
        },
        BallUpdateState::PaddleCollision => {
            ball_movement.x = -ball_movement.x;
            ball_movement.x += if ball_movement.x > 0 { 1 } else { -1 };
            ball_movement.x = if ball_movement.x > 0 { 
                min(ball_movement.x, BALL_MAX_SPEED as i32) 
            } else { 
                max(ball_movement.x, -(BALL_MAX_SPEED as i32)) 
            };
            pop_sound.play(1)?;
        },
        BallUpdateState::Moving => (),
    }
    Ok(())
}

fn move_paddle(paddle: &mut Entity, movement: i32) {
    let position_y = paddle.position.y + movement;
    let position = Point::new(paddle.position.x, position_y);
    let collider_rect = Rect::from_center(
        position, 
        PADDLE_COLLIDER_SIZE.0, 
        PADDLE_COLLIDER_SIZE.1
    );
    let window_top = -(WINDOW_HALF_SIZE.1 as i32);
    let window_bottom = WINDOW_HALF_SIZE.1 as i32;
    if collider_rect.top() >= window_top && 
        collider_rect.bottom() <= window_bottom {
        paddle.position.y = position_y;
    }
}

fn move_ball(ball: &mut Entity, movement: &mut Point) -> BallMoveState {
    let position = ball.position + *movement;
    let window_top = -(WINDOW_HALF_SIZE.1 as i32);
    let window_bottom = WINDOW_HALF_SIZE.1 as i32;
    let result: BallMoveState;
    if position.y - (BALL_RADIUS as i32) < window_top || 
        position.y + (BALL_RADIUS as i32) > window_bottom {
        movement.y = -movement.y;
        result = BallMoveState::WallCollision;
    } else {
        result = BallMoveState::Moving;
    }
    ball.position += *movement;
    result
}

fn update_ball_state(ball: &Entity, ball_movement: Point, paddle: &Entity) -> BallUpdateState {
    if ball_movement.x == 0 {
        return BallUpdateState::Moving
    }
    let paddle_collider_rect = Rect::from_center(
        paddle.position, 
        PADDLE_COLLIDER_SIZE.0, 
        PADDLE_COLLIDER_SIZE.1
    );
    if ball_movement.x > 0 {
        if ball.position.x + (BALL_RADIUS as i32) > paddle_collider_rect.left() {
            if ball.position.y >= paddle_collider_rect.top_left().y && 
                ball.position.y <= paddle_collider_rect.bottom_left().y {
                return BallUpdateState::PaddleCollision;
            } else {
                return BallUpdateState::Scoring;
            }
        }
    } else {
        if ball.position.x - (BALL_RADIUS as i32) < paddle_collider_rect.right() {
            if ball.position.y >= paddle_collider_rect.top_right().y && 
                ball.position.y <= paddle_collider_rect.bottom_right().y {
                return BallUpdateState::PaddleCollision;
            } else {
                return BallUpdateState::Scoring;
            }
        }
    }
    return BallUpdateState::Moving
}

fn render<T>(canvas: &mut WindowCanvas, 
    background: Color, 
    entities: &Vec<Entity>, 
    paddle1_score_label: &mut Label<T>, 
    paddle2_score_label: &mut Label<T>) -> Result<(), String> {
    canvas.set_draw_color(background);
    canvas.clear();
    for entity in entities {
        render_entity(canvas, entity)?;
    }
    render_label(canvas, paddle1_score_label)?;
    render_label(canvas, paddle2_score_label)?;
    canvas.present();
    Ok(())
}

fn render_entity(canvas: &mut WindowCanvas, entity: &Entity) -> Result<(), String> {
    let center_screen = Point::new(WINDOW_HALF_SIZE.0 as i32, WINDOW_HALF_SIZE.1 as i32);
    let position_in_screen = center_screen + entity.position;
    let rect = Rect::from_center(position_in_screen, entity.size.0, entity.size.1);
    canvas.copy(entity.sprite.texture, entity.sprite.rect, rect)?;
    Ok(())
}

fn render_label<T>(canvas: &mut WindowCanvas, label: &mut Label<T>) -> Result<(), String> {
    let center_screen = Point::new(WINDOW_HALF_SIZE.0 as i32, WINDOW_HALF_SIZE.1 as i32);
    let position_in_screen = center_screen + label.position;
    let rect = Rect::from_center(position_in_screen, label.width, label.height);
    let texture = label.texture()?;
    canvas.copy(texture, None, rect)?;
    Ok(())
}

fn main() -> Result<(), String> {
    let (sdl_context, ttf_context) = init()?;
    let _sdl_audio = sdl_context.audio()
        .expect("Could not initialize SDL audio");
    let font = ttf_context.load_font(FONT_FILENAME, 60)
        .expect(&format!("Could not load font: {}", FONT_FILENAME));
    let pop_sound = mixer::Music::from_file(POP_SOUND_FILENAME)
        .expect(&format!("Could not load audio: {}", POP_SOUND_FILENAME));
    let score_sound = mixer::Music::from_file(SCORE_SOUND_FILENAME)
        .expect(&format!("Could not load audio: {}", SCORE_SOUND_FILENAME));
    let video_subsystem = sdl_context.video()
        .expect("Could not initialize video system");
    let window = video_subsystem.window("Pong", WINDOW_SIZE.0, WINDOW_SIZE.1)
        .position_centered()
        .build()
        .expect("Could not initialize window");
    let mut canvas = window.into_canvas()
        .build()
        .expect("Could not create a canvas");
    let texture_creator = canvas.texture_creator();
    let texture = texture_creator.load_texture(SPRITESHEET_FILENAME)
        .expect(&format!("Could not load image: {}", SPRITESHEET_FILENAME));
    let mut event_pump = sdl_context.event_pump()?;
    let fps = TARGET_FPS as u32;
    let mut entities: Vec<Entity> = Vec::new();
    entities.push(Entity {
        position: Point::new(
            -(WINDOW_HALF_SIZE.0 as i32) + PADDLE_COLLIDER_SIZE.0 as i32, 0
        ),
        size: PADDLE_SIZE,
        sprite: Sprite {
            texture: &texture,
            rect: Rect::new(0, 0, PADDLE_SIZE.0, PADDLE_SIZE.1)
        }
    });
    let paddle1_index = entities.len() - 1; 
    entities.push(Entity {
        position: Point::new(
            (WINDOW_HALF_SIZE.0 as i32) - PADDLE_COLLIDER_SIZE.0 as i32, 0
        ),
        size: PADDLE_SIZE,
        sprite: Sprite {
            texture: &texture,
            rect: Rect::new(52, 0, PADDLE_SIZE.0, PADDLE_SIZE.1)
        }
    });
    let paddle2_index = entities.len() - 1;
    entities.push(Entity {
        position: Point::new(0, 0),
        size: BALL_SIZE,
        sprite: Sprite {
            texture: &texture,
            rect: Rect::new(100, 0, BALL_SIZE.0, BALL_SIZE.1)
        }
    });
    let ball_index = entities.len() - 1;
    let mut paddle1_score: u64 = 0;
    let mut paddle1_score_label = Label::new(
        String::from("0"), 
        &font, 
        Point::new(
            -(WINDOW_HALF_SIZE.0 as i32) / 2, 
            -(WINDOW_HALF_SIZE.1 as i32) + 60
        ), 
        Color::RED, 
        &texture_creator
    )?;
    let mut paddle2_score: u64 = 0;
    let mut paddle2_score_label = Label::new(
        String::from("0"), 
        &font, 
        Point::new(
            WINDOW_HALF_SIZE.0 as i32 / 2, 
            -(WINDOW_HALF_SIZE.1 as i32) + 60
        ), 
        Color::BLUE, 
        &texture_creator
    )?;
    let mut paddle1_movement: i32 = 0;
    let mut paddle2_movement: i32 = 0;
    let mut ball_movement = Point::new(BALL_SPEED as i32, BALL_SPEED as i32);
    // Game loop
    'running: loop {
        // Handle events
        match handle_events(&mut event_pump, &mut paddle1_movement, &mut paddle2_movement) {
            HandleEventsState::Exit => break 'running,
            HandleEventsState::Running => ()
        };
        // Update
        update(&mut entities, 
            ball_index, 
            &mut ball_movement,
            paddle1_index,
            paddle1_movement,
            &mut paddle1_score,
            &mut paddle1_score_label,
            paddle2_index,
            paddle2_movement,
            &mut paddle2_score,
            &mut paddle2_score_label,
            &pop_sound,
            &score_sound)?;
        // Render
        render(&mut canvas, 
            BACKGROUND_COLOR, 
            &entities, 
            &mut paddle1_score_label, 
            &mut paddle2_score_label)?;
        // Time management
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / fps));
    }
    Ok(())
}
