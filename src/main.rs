use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::render::{WindowCanvas, Texture};
use sdl2::rect::{Point, Rect};
use sdl2::sys::{SDL_GetPerformanceCounter, SDL_GetPerformanceFrequency};
use sdl2::image::{self, InitFlag};
use std::time::Duration;

const TARGET_FPS: u8 = 60;
const BACKGROUND_COLOR: Color = Color::RGB(0, 0, 0);
const WINDOW_SIZE: (u32, u32) = (800, 600);

fn render(canvas: &mut WindowCanvas, background: Color) -> Result<(), String> {
    canvas.set_draw_color(background);
    canvas.clear();
    let window_size = canvas.output_size()?;
    // TODO: Render entities
    canvas.present();
    Ok(())
}

fn render_entity(canvas: &mut WindowCanvas, window_size: (u32, u32), entity_position: Point, entity_size: (u32, u32), entity_rect: Rect, entity_texture: &Texture) -> Result<(), String> {
    // Treat the center of the screen as the (0, 0) coordinate
    let center_screen = Point::new(window_size.0 as i32 / 2, window_size.1 as i32 / 2);
    let position_in_screen = center_screen + entity_position;
    let rect = Rect::from_center(position_in_screen, entity_size.0, entity_size.1);
    canvas.copy(entity_texture, entity_rect, rect)?;
    Ok(())
}

fn update() {
    // TODO: Update entities
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
    // TODO: Create textures
    let mut event_pump = sdl_context.event_pump()?;
    let fps = TARGET_FPS as u32;
    let mut performance_counter: u64;
    unsafe { performance_counter = SDL_GetPerformanceCounter(); }
    let mut last_performance_counter: u64 = 0;
    let mut performance_frequency: u64 = 0;
    let mut delta_time: f32 = 0.0;
    let mut movement: Point = Point::new(0, 0);
    'running: loop {
        movement.x = 0;
        movement.y = 0;
        last_performance_counter = performance_counter;
        unsafe { 
            performance_counter = SDL_GetPerformanceCounter();
            performance_frequency = SDL_GetPerformanceFrequency();
        }
        delta_time = ((performance_counter - last_performance_counter) * 1000) as f32 / performance_frequency as f32;
        // Handle events
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::KeyDown { keycode: Some(Keycode::Up), .. } => {
                },
                Event::KeyDown { keycode: Some(Keycode::Down), .. } => {
                },
                Event::KeyDown { keycode: Some(Keycode::Right), .. } => {
                },
                Event::KeyDown { keycode: Some(Keycode::Left), .. } => {
                },
                _ => {}
            }
        }
        // Update
        update();
        // Render
        render(&mut canvas, BACKGROUND_COLOR)?;
        // Time management
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / fps));
    }
    Ok(())
}
