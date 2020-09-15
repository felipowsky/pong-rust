use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::render::{WindowCanvas, Texture};
use sdl2::rect::{Point, Rect};
use sdl2::image::{self, LoadTexture};
use sdl2::mixer::{self, DEFAULT_CHANNELS, AUDIO_S16LSB};
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

struct Sprite<'a> {
    texture: &'a Texture<'a>,
    rect: Rect
}

struct Entity<'a> {
    position: Point,
    size: (u32, u32),
    sprite: Sprite<'a>
}

enum BallUpdateState {
    PaddleCollision,
    Scoring,
    Moving
}

enum BallMoveState {
    WallCollision,
    Moving
}

fn render(canvas: &mut WindowCanvas, background: Color, entities: &Vec<Entity>) -> Result<(), String> {
    canvas.set_draw_color(background);
    canvas.clear();
    for entity in entities {
        render_entity(canvas, entity)?;
    }
    canvas.present();
    Ok(())
}

fn render_entity(canvas: &mut WindowCanvas, entity: &Entity) -> Result<(), String> {
    // Treat the center of the screen as the (0, 0) coordinate
    let center_screen = Point::new(WINDOW_HALF_SIZE.0 as i32, WINDOW_HALF_SIZE.1 as i32);
    let position_in_screen = center_screen + entity.position;
    let rect = Rect::from_center(position_in_screen, entity.size.0, entity.size.1);
    canvas.copy(entity.sprite.texture, entity.sprite.rect, rect)?;
    Ok(())
}

fn move_paddle(paddle: &mut Entity, movement: i32) {
    let position_y = paddle.position.y + movement;
    let position = Point::new(paddle.position.x, position_y);
    let collider_rect = Rect::from_center(position, PADDLE_COLLIDER_SIZE.0, PADDLE_COLLIDER_SIZE.1);
    let window_top = -(WINDOW_HALF_SIZE.1 as i32);
    let window_bottom = WINDOW_HALF_SIZE.1 as i32;
    if collider_rect.top() >= window_top && collider_rect.bottom() <= window_bottom {
        paddle.position.y = position_y;
    }
}

fn move_ball(ball: &mut Entity, movement: &mut Point) -> BallMoveState {
    let position = ball.position + *movement;
    let window_top = -(WINDOW_HALF_SIZE.1 as i32);
    let window_bottom = WINDOW_HALF_SIZE.1 as i32;
    let mut result = BallMoveState::Moving;
    if position.y - (BALL_RADIUS as i32) < window_top || position.y + (BALL_RADIUS as i32) > window_bottom {
        movement.y = -movement.y;
        result = BallMoveState::WallCollision;
    }
    ball.position += *movement;
    result
}

fn update_ball_state(ball: &Entity, ball_movement: Point, paddle: &Entity) -> BallUpdateState {
    if ball_movement.x == 0 {
        return BallUpdateState::Moving
    }
    let paddle_collider_rect = Rect::from_center(paddle.position, PADDLE_COLLIDER_SIZE.0, PADDLE_COLLIDER_SIZE.1);
    if ball_movement.x > 0 {
        if ball.position.x + (BALL_RADIUS as i32) > paddle_collider_rect.left() {
            if ball.position.y >= paddle_collider_rect.top_left().y && ball.position.y <= paddle_collider_rect.bottom_left().y {
                return BallUpdateState::PaddleCollision;
            } else {
                return BallUpdateState::Scoring;
            }
        }
    } else {
        if ball.position.x - (BALL_RADIUS as i32) < paddle_collider_rect.right() {
            if ball.position.y >= paddle_collider_rect.top_right().y && ball.position.y <= paddle_collider_rect.bottom_right().y {
                return BallUpdateState::PaddleCollision;
            } else {
                return BallUpdateState::Scoring;
            }
        }
    }
    return BallUpdateState::Moving
}

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let _sdl_image_context = image::init(image::InitFlag::PNG)?;
    let _sdl_audio = sdl_context.audio()?;
    let frequency = 44_100;
    let format = AUDIO_S16LSB;
    let channels = DEFAULT_CHANNELS;
    let chunk_size = 1_024;
    sdl2::mixer::open_audio(frequency, format, channels, chunk_size)?;
    let _sdl_mixer_context = mixer::init(mixer::InitFlag::OGG)?;
    sdl2::mixer::allocate_channels(4);
    let pop_sound = sdl2::mixer::Music::from_file(POP_SOUND_FILENAME)?;
    let score_sound = sdl2::mixer::Music::from_file(SCORE_SOUND_FILENAME)?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem.window("Pong", WINDOW_SIZE.0, WINDOW_SIZE.1)
        .position_centered()
        .build()
        .expect("Could not initialize window");
    let mut canvas = window.into_canvas().build().expect("Could not create a canvas");
    let texture_creator = canvas.texture_creator();
    let texture = texture_creator.load_texture(SPRITESHEET_FILENAME)?;
    let mut event_pump = sdl_context.event_pump()?;
    let fps = TARGET_FPS as u32;
    let mut entities: Vec<Entity> = Vec::new();
    entities.push(Entity {
        position: Point::new(-(WINDOW_HALF_SIZE.0 as i32) + PADDLE_COLLIDER_SIZE.0 as i32, 0),
        size: PADDLE_SIZE,
        sprite: Sprite {
            texture: &texture,
            rect: Rect::new(0, 0, PADDLE_SIZE.0, PADDLE_SIZE.1)
        }
    });
    let paddle1_index = entities.len() - 1; 
    entities.push(Entity {
        position: Point::new((WINDOW_HALF_SIZE.0 as i32) - PADDLE_COLLIDER_SIZE.0 as i32, 0),
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
    let mut score: (u64, u64) = (0, 0);
    let mut paddle1_movement: i32 = 0;
    let mut paddle2_movement: i32 = 0;
    let mut ball_movement = Point::new(BALL_SPEED as i32, BALL_SPEED as i32);
    'running: loop {
        // Handle events
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::KeyDown { keycode: Some(Keycode::W), .. } => {
                    paddle1_movement = -(PADDLE_SPEED as i32);
                },
                Event::KeyDown { keycode: Some(Keycode::S), .. } => {
                    paddle1_movement = PADDLE_SPEED as i32;
                },
                Event::KeyUp { keycode: Some(Keycode::W), .. } |
                Event::KeyUp { keycode: Some(Keycode::S), .. } => {
                    paddle1_movement = 0;
                },
                Event::KeyDown { keycode: Some(Keycode::Up), .. } => {
                    paddle2_movement = -(PADDLE_SPEED as i32);
                },
                Event::KeyDown { keycode: Some(Keycode::Down), .. } => {
                    paddle2_movement = PADDLE_SPEED as i32;
                },
                Event::KeyUp { keycode: Some(Keycode::Up), .. } |
                Event::KeyUp { keycode: Some(Keycode::Down), .. } => {
                    paddle2_movement = 0;
                },
                _ => {}
            }
        }
        // Update
        match move_ball(&mut entities[ball_index], &mut ball_movement) {
            BallMoveState::WallCollision => { pop_sound.play(1)?; },
            BallMoveState::Moving => (),
        }
        move_paddle(&mut entities[paddle1_index], paddle1_movement);
        move_paddle(&mut entities[paddle2_index], paddle2_movement);
        let paddle_index_collision = if ball_movement.x > 0 { paddle2_index } else { paddle1_index };
        match update_ball_state(&entities[ball_index], ball_movement, &entities[paddle_index_collision]) {
            BallUpdateState::Scoring => {
                if ball_movement.x > 0 {
                    score.0 += 1
                } else {
                    score.1 += 1
                }
                score_sound.play(1)?;
                ball_movement = Point::new(BALL_SPEED as i32, BALL_SPEED as i32);
                entities[ball_index].position = Point::new(0, 0);
            },
            BallUpdateState::PaddleCollision => {
                ball_movement.x = -ball_movement.x;
                ball_movement.x += if ball_movement.x > 0 { 1 } else { -1 };
                ball_movement.x = if ball_movement.x > 0 { min(ball_movement.x, BALL_MAX_SPEED as i32) } else { max(ball_movement.x, -(BALL_MAX_SPEED as i32)) };
                pop_sound.play(1)?;
            },
            BallUpdateState::Moving => (),
        }
        // Render
        render(&mut canvas, BACKGROUND_COLOR, &entities)?;
        // Time management
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / fps));
    }
    Ok(())
}
