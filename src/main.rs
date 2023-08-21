extern crate sdl2;

use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::{event::Event, keyboard::Keycode};
use std::collections::HashMap;
use std::io::repeat;
use std::time::Duration;

const WINDOW_HEIGHT: i32 = 32;
const WINDOW_WIDTH: i32 = 64;
const SCALE: i32 = 10;
const SCALED_WINDOW_HEIGHT: i32 = WINDOW_HEIGHT * SCALE;
const SCALED_WINDOW_WIDTH: i32 = WINDOW_WIDTH * SCALE;

// TODO: add the wraping behavior
struct DisplayBuffer {
    pixels: [[bool; 64]; 32],
}

impl DisplayBuffer {
    fn new() -> Self {
        Self {
            pixels: [[false; 64]; 32],
        }
    }

    fn switch(self: &mut Self, point: Point) -> () {
        self.pixels[point.y as usize][point.x as usize] =
            !self.pixels[point.y as usize][point.x as usize];
    }

    fn is_on(self: &Self, point: Point) -> bool {
        self.pixels[point.y as usize][point.x as usize]
    }

    fn clear(self: &mut Self) -> () {
        self.pixels = [[false; 64]; 32];
    }
}

pub fn main() -> Result<(), String> {
    let mut display_buffer = DisplayBuffer::new();

    let keymap: HashMap<i32, i32> = HashMap::from([
        (49, 0x1),  // 1
        (50, 0x2),  // 2
        (51, 0x3),  // 3
        (52, 0xc),  // 4
        (81, 0x4),  // Q
        (87, 0x5),  // W
        (69, 0x6),  // E
        (82, 0xD),  // R
        (65, 0x7),  // A
        (83, 0x8),  // S
        (68, 0x9),  // D
        (70, 0xE),  // F
        (90, 0xA),  // Z
        (88, 0x0),  // X
        (67, 0xB),  // C
        (86, 0xF),  // V
        (97, 0x7),  // a
        (115, 0x8), // s
        (100, 0x9), // d
        (102, 0xE), // f
        (122, 0xA), // z
        (120, 0x0), // x
        (99, 0xB),  // c
        (118, 0xF), // v
    ]);

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window(
            "chip-8-emulator",
            SCALED_WINDOW_WIDTH as u32,
            SCALED_WINDOW_HEIGHT as u32,
        )
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let mut event_pump = sdl_context.event_pump()?;

    display_buffer.switch(Point::new(0, 0));
    display_buffer.switch(Point::new(10, 5));

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyUp {
                    timestamp: _,
                    window_id: _,
                    keycode: Some(keycode),
                    scancode: _,
                    keymod: _,
                    repeat: _,
                } => {
                    println!("keycode {}", keycode as i32);
                    if let Some(mapped_value) = keymap.get(&(keycode as i32)) {
                        println!("mapped value {}", mapped_value);
                    }
                }
                _ => {}
            }
        }

        canvas.set_draw_color(Color::BLACK);
        canvas.clear();
        for y in 0..32 {
            for x in 0..64 {
                if display_buffer.is_on(Point::new(x, y)) {
                    canvas.set_draw_color(Color::GREEN);
                    let rectangle =
                        Rect::new((x * 10) as i32, (y * 10) as i32, SCALE as u32, SCALE as u32);
                    canvas.draw_rect(rectangle).unwrap();
                    canvas.fill_rect(rectangle).unwrap();
                }
            }
        }
        canvas.present();

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
        // The rest of the game loop goes here...
    }

    Ok(())
}
