use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::render::{WindowCanvas, Texture};
use sdl2::rect::{Point, Rect};
use sdl2::image::{self, LoadTexture, InitFlag};
use std::time::Duration;

const TARGET_FPS: u8 = 60;
const BACKGROUND_COLOR: Color = Color::RGB(0, 0, 0);
const WINDOW_SIZE: (u32, u32) = (800, 600);
const WINDOW_HALF_SIZE: (u32, u32) = (WINDOW_SIZE.0 / 2, WINDOW_SIZE.1 / 2);
const PADDLE_SIZE: (u32, u32) = (52, 150);
const PADDLE_PADDING: u8 = 2;
const PADDLE_SPEED: u8 = 4;
const BALL_SIZE: (u32, u32) = (51, 51);
const BALL_SPEED: u8 = 3;
const SPRITESHEET_FILENAME: &str = "assets/spritesheet.png";

struct Sprite<'a> {
    texture: &'a Texture<'a>,
    rect: Rect
}

struct Entity<'a> {
    position: Point,
    size: (u32, u32),
    collider_size: (u32, u32),
    sprite: Sprite<'a>
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
    let collider_rect = Rect::from_center(position, paddle.collider_size.0, paddle.collider_size.1);
    let window_top = -(WINDOW_HALF_SIZE.1 as i32);
    let window_bottom = WINDOW_HALF_SIZE.1 as i32;
    if collider_rect.top() >= window_top && collider_rect.bottom() <= window_bottom {
        paddle.position.y = position_y;
    }
}

fn move_ball(ball: &mut Entity, movement: &mut Point) {
    let position = ball.position + *movement;
    let collider_rect = Rect::from_center(position, ball.collider_size.0, ball.collider_size.1);
    let window_right = WINDOW_HALF_SIZE.0 as i32;
    let window_left = -(WINDOW_HALF_SIZE.0 as i32);
    if collider_rect.right() > window_right || collider_rect.left() < window_left {
        movement.x = movement.x * -1;
    }
    let window_top = -(WINDOW_HALF_SIZE.1 as i32);
    let window_bottom = WINDOW_HALF_SIZE.1 as i32;
    if collider_rect.top() < window_top || collider_rect.bottom() > window_bottom {
        movement.y = -movement.y;
    }
    ball.position += *movement;
}

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let _sdl_image_context = image::init(InitFlag::PNG | InitFlag::JPG)?;
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
        position: Point::new(-(WINDOW_HALF_SIZE.0 as i32) + (PADDLE_SIZE.0 as i32 + PADDLE_PADDING as i32), 0),
        size: PADDLE_SIZE,
        collider_size: PADDLE_SIZE,
        sprite: Sprite {
            texture: &texture,
            rect: Rect::new(0, 0, PADDLE_SIZE.0, PADDLE_SIZE.1)
        }
    });
    let paddle1_index = entities.len() - 1; 
    entities.push(Entity {
        position: Point::new((WINDOW_HALF_SIZE.0 as i32) - (PADDLE_SIZE.0 as i32 + PADDLE_PADDING as i32), 0),
        size: PADDLE_SIZE,
        collider_size: PADDLE_SIZE,
        sprite: Sprite {
            texture: &texture,
            rect: Rect::new(52, 0, PADDLE_SIZE.0, PADDLE_SIZE.1)
        }
    });
    let paddle2_index = entities.len() - 1;
    entities.push(Entity {
        position: Point::new(0, 0),
        size: BALL_SIZE,
        collider_size: BALL_SIZE,
        sprite: Sprite {
            texture: &texture,
            rect: Rect::new(100, 0, BALL_SIZE.0, BALL_SIZE.1)
        }
    });
    let ball_index = entities.len() - 1;
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
        move_ball(&mut entities[ball_index], &mut ball_movement);
        move_paddle(&mut entities[paddle1_index], paddle1_movement);
        move_paddle(&mut entities[paddle2_index], paddle2_movement);
        // Render
        render(&mut canvas, BACKGROUND_COLOR, &entities)?;
        // Time management
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / fps));
    }
    Ok(())
}
