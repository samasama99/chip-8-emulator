extern crate sdl2;

use rand::prelude::*;

use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::{event::Event, keyboard::Keycode};
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{self, Read};
use std::time::{Duration, SystemTime};

use imgui::*;
use imgui_sdl2::*;

const WINDOW_HEIGHT: i32 = 32;
const WINDOW_WIDTH: i32 = 64;
const SCALE: i32 = 10;
const SCALED_WINDOW_HEIGHT: i32 = WINDOW_HEIGHT * SCALE;
const SCALED_WINDOW_WIDTH: i32 = WINDOW_WIDTH * SCALE;

struct CHIP8 {
    now: SystemTime,
    display_buffer: DisplayBuffer,
    memory: [u8; 4096],
    v: [u8; 16],
    i: u16,
    delay_timer: u8,
    sound_timer: u8,
    pc: u16,
    stack: Vec<u16>,
    paused: bool,
    speed: u32,
    keys_pressed: HashMap<u8, bool>,
    on_next_key_press: Option<Box<dyn FnOnce(&mut CHIP8, u8)>>,
}

impl CHIP8 {
    fn new(display_buffer: DisplayBuffer) -> Self {
        Self {
            now: SystemTime::now(),
            display_buffer,
            memory: [0; 4096],
            v: [0; 16],
            i: 0,
            delay_timer: 0,
            sound_timer: 0,
            pc: 0x200,
            stack: vec![],
            paused: false,
            speed: 10,
            keys_pressed: HashMap::from([
                (0x1, false), // 1
                (0x2, false), // 2
                (0x3, false), // 3
                (0xc, false), // 4
                (0x4, false), // Q
                (0x5, false), // W
                (0x6, false), // E
                (0xD, false), // R
                (0x7, false), // A
                (0x8, false), // S
                (0x9, false), // D
                (0xE, false), // F
                (0xA, false), // Z
                (0x0, false), // X
                (0xB, false), // C
                (0xF, false), // V
                (0x7, false), // a
                (0x8, false), // s
                (0x9, false), // d
                (0xE, false), // f
                (0xA, false), // z
                (0x0, false), // x
                (0xB, false), // c
                (0xF, false), // v
            ]),
            on_next_key_press: None,
        }
    }

    fn load_sprites_into_memory(&mut self) {
        // Array of hex values for each sprite. Each sprite is 5 bytes.
        // The technical reference provides us with each one of these values.
        let sprites = [
            0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70, // 1
            0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
            0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
            0x90, 0x90, 0xF0, 0x10, 0x10, // 4
            0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
            0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
            0xF0, 0x10, 0x20, 0x40, 0x40, // 7
            0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
            0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
            0xF0, 0x90, 0xF0, 0x90, 0x90, // A
            0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
            0xF0, 0x80, 0x80, 0x80, 0xF0, // C
            0xE0, 0x90, 0x90, 0x90, 0xE0, // D
            0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
            0xF0, 0x80, 0xF0, 0x80, 0x80, // F
        ];

        // According to the technical reference, sprites are stored in the interpreter section of memory starting at hex 0x000
        self.memory[..sprites.len()].copy_from_slice(&sprites[..]);
    }

    fn load_program_into_memory(&mut self, _program: &[u8]) {
        self.memory[512..(_program.len() + 512)].copy_from_slice(_program);
        // let test = [
        //     0x60, 0x1F, // Set V0 to X coordinate (31)
        //     0x61, 0x0F, // Set V1 to Y coordinate (15)
        //     0x00, 0xE0, // clearing the window
        //     0xA0, 0x05, // Set I to the address of sprite '1' (0x05)
        //     0xD0, 0x15, // Draw the sprite at (V0, V1)
        //     0x12, 0x0A, // Skip the next instructions (0x20A)
        //     // 0x12, 0x00, // Jump to the start of the program
        // ];
        // self.memory[0x200..(test.len() + 0x200)].copy_from_slice(&test);
    }

    fn update_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        };

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        };
    }

    fn execute_instruction(&mut self, opcode: u16) {
        self.pc += 2;

        // We only need the 2nd nibble, so grab the value of the 2nd nibble
        // and shift it right 8 bits to get rid of everything but that 2nd nibble.
        let x: u16 = (opcode & 0x0F00) >> 8;

        // We only need the 3rd nibble, so grab the value of the 3rd nibble
        // and shift it right 4 bits to get rid of everything but that 3rd nibble.
        let y: u16 = (opcode & 0x00F0) >> 4;

        // println!("{} {:02X}", self.now.elapsed().unwrap().as_secs(), opcode);
        // println!(
        //     "{} {:02X} {:02X}",
        //     self.pc,
        //     self.memory[0x349],
        //     self.memory[0x349 + 0x1]
        // );
        match opcode & 0xF000 {
            0x0000 => match opcode {
                0x00E0 => {
                    self.display_buffer.clear();
                    // println!("calling display clear");
                }
                0x00EE => {
                    self.pc = self.stack.pop().unwrap();
                }
                _ => {
                    // Unknown opcode
                    panic!("Unknown opcode {}", opcode);
                }
            },
            0x1000 => {
                // let next_op: u16 = (self.memory[(opcode & 0x0FFF) as usize] as u16) << 8
                //     | self.memory[(opcode & 0x0FFF) as usize + 1] as u16;

                // if next_op == opcode {
                //     self.pc += 2;
                // } else {
                self.pc = opcode & 0x0FFF;
                // }
            }
            0x2000 => {
                self.stack.push(self.pc);
                self.pc = opcode & 0xFFF;
            }
            0x3000 => {
                if self.v[x as usize] == (opcode & 0xFF) as u8 {
                    self.pc += 2;
                }
            }
            0x4000 => {
                if self.v[x as usize] != ((opcode & 0xFF) as u8) {
                    self.pc += 2;
                }
            }
            0x5000 => {
                if self.v[x as usize] == self.v[y as usize] {
                    self.pc += 2;
                }
            }
            0x6000 => {
                self.v[x as usize] = (opcode & 0xFF) as u8;
            }
            0x7000 => {
                self.v[x as usize] = self.v[x as usize].wrapping_add((opcode & 0xFF) as u8);
            }
            0x8000 => match opcode & 0xF {
                0x0 => {
                    self.v[x as usize] = self.v[y as usize];
                }
                0x1 => {
                    self.v[x as usize] |= self.v[y as usize];
                }
                0x2 => {
                    self.v[x as usize] &= self.v[y as usize];
                }
                0x3 => {
                    self.v[x as usize] ^= self.v[y as usize];
                }
                0x4 => {
                    self.v[x as usize] = self.v[x as usize].wrapping_add(self.v[y as usize]);
                    let sum: u16 = self.v[x as usize] as u16 + self.v[y as usize] as u16;
                    self.v[0xF] = if sum < 0xFFu16 { 1 } else { 0 };
                }
                0x5 => {
                    self.v[0xF] = 0;

                    if self.v[x as usize] > self.v[y as usize] {
                        self.v[0xF] = 1;
                    }

                    self.v[x as usize] = self.v[x as usize].wrapping_sub(self.v[y as usize]);
                }
                0x6 => {
                    self.v[0xF] = self.v[x as usize] & 0x1;

                    self.v[x as usize] >>= 1;
                }
                0x7 => {
                    self.v[0xF] = 0;

                    if self.v[y as usize] > self.v[x as usize] {
                        self.v[0xF] = 1;
                    }

                    self.v[x as usize] = self.v[y as usize] - self.v[x as usize];
                }
                0xE => {
                    self.v[0xF] = self.v[x as usize] & 0x80;
                    self.v[x as usize] <<= 1;
                }
                _ => {
                    // Unknown opcode
                    panic!("Unknown opcode {}", opcode);
                }
            },
            0x9000 => {
                if self.v[x as usize] != self.v[y as usize] {
                    self.pc += 2;
                }
            }
            0xA000 => {
                self.i = opcode & 0xFFF;
            }
            0xB000 => {
                self.pc = (opcode & 0xFFF) + self.v[0] as u16;
            }
            0xC000 => {
                let mut rng = rand::thread_rng();
                let rand: u8 = rng.gen_range(0..=0xFF);

                self.v[x as usize] = rand & (opcode & 0xFF) as u8;
            }
            0xD000 => {
                let width = 8;
                let height = opcode & 0xF;

                self.v[0xF] = 0;

                // dbg!(self.i);

                for row in 0..height {
                    let mut sprite = self.memory[self.i as usize + row as usize];

                    for col in 0..width {
                        // If the bit (sprite) is not 0, render/erase the pixel
                        if (sprite & 0x80) > 0 {
                            // If setPixel returns 1, which means a pixel was erased, set VF to 1
                            if self.display_buffer.toggle_pixel(Point::new(
                                (self.v[x as usize] as u16 + col) as i32,
                                (self.v[y as usize] as u16 + row) as i32,
                            )) {
                                self.v[0xF] = 1;
                            }
                        }

                        // Shift the sprite left 1. This will move the next next col/bit of the sprite into the first position.
                        // Ex. 10010000 << 1 will become 0010000
                        sprite <<= 1;
                    }
                }
            }
            0xE000 => match opcode & 0xFF {
                0x9E => {
                    if *self.keys_pressed.get(&self.v[x as usize]).unwrap_or(&false) {
                        self.pc += 2;
                    }
                }
                0xA1 => {
                    if !self.keys_pressed.get(&self.v[x as usize]).unwrap_or(&false) {
                        self.pc += 2;
                    }
                }
                _ => {
                    panic!("Unknown opcode {}", opcode);
                }
            },
            0xF000 => match opcode & 0xFF {
                0x07 => {
                    self.v[x as usize] = self.delay_timer;
                }
                0x0A => {
                    self.paused = true;

                    let closure: Box<dyn FnOnce(&mut CHIP8, u8)> = Box::new(move |chip8, key| {
                        chip8.v[x as usize] = key;
                        chip8.paused = false;
                    });

                    self.on_next_key_press = Some(closure);
                }
                0x15 => {
                    self.delay_timer = self.v[x as usize];
                }
                0x18 => {
                    self.sound_timer = self.v[x as usize];
                }
                0x1E => {
                    self.i += self.v[x as usize] as u16;
                }
                0x29 => {
                    self.i = self.v[x as usize] as u16 * 5;
                }
                0x33 => {
                    self.memory[self.i as usize] = self.v[x as usize] / 100;

                    // Get tens digit and place it in I+1. Gets a value between 0 and 99,
                    // then divides by 10 to give us a value between 0 and 9.
                    self.memory[self.i as usize + 1] = (self.v[x as usize] % 100) / 10;

                    // Get the value of the ones (last) digit and place it in I+2.
                    self.memory[self.i as usize + 2] = self.v[x as usize] % 10;
                }
                0x55 => {
                    for register_index in 0..x {
                        self.memory[self.i as usize + register_index as usize] =
                            self.v[register_index as usize];
                    }
                }
                0x65 => {
                    for register_index in 0..x {
                        self.v[register_index as usize] =
                            self.memory[self.i as usize + register_index as usize];
                    }
                }
                _ => {
                    panic!("Unknown opcode {}", opcode);
                }
            },
            _ => {
                panic!("Unknown opcode {}", opcode);
            }
        }
    }

    fn cycle(&mut self) {
        for _ in 0..self.speed {
            if !self.paused {
                let opcode: u16 = (self.memory[self.pc as usize] as u16) << 8
                    | self.memory[self.pc as usize + 1] as u16;
                self.execute_instruction(opcode);
                // println!("{}", opcode);
            }
        }

        if !self.paused {
            self.update_timers();
        }

        // self.playSound();
    }

    fn set_key_press(&mut self, mapped_value: u8) {
        self.keys_pressed.insert(mapped_value, true);
    }

    fn unset_key_press(&mut self, mapped_value: u8) {
        self.keys_pressed.insert(mapped_value, false);
    }
}

#[derive(Debug)]
struct DisplayBuffer {
    pixels: [[bool; 64]; 32],
}

impl DisplayBuffer {
    fn new() -> Self {
        Self {
            pixels: [[false; 64]; 32],
        }
    }

    fn toggle_pixel(&mut self, point: Point) -> bool {
        let wrapped_point = Point::new(point.x() % WINDOW_WIDTH, point.y() % WINDOW_HEIGHT);
        let pixel = &mut self.pixels[wrapped_point.y as usize][wrapped_point.x as usize];
        *pixel ^= true;
        *pixel
    }

    fn is_on(&self, point: Point) -> bool {
        let wrapped_point = Point::new(point.x() % WINDOW_WIDTH, point.y() % WINDOW_HEIGHT);
        self.pixels[wrapped_point.y as usize][wrapped_point.x as usize]
    }

    fn clear(&mut self) {
        self.pixels = [[false; 64]; 32];
    }
}

fn read_file_to_vec(filename: &str) -> io::Result<Vec<u8>> {
    let mut file = File::open(filename)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    println!(
        "{:#?}",
        buffer
            .iter()
            .map(|&x| format!("0x{:02X}", x))
            .collect::<Vec<_>>()
    );
    Ok(buffer)
}

pub fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    let filename = args.get(1).unwrap();

    let data = match read_file_to_vec(filename) {
        Ok(data) => data,
        Err(error) => {
            panic!("Error: {}", error);
        }
    };

    let display_buffer = DisplayBuffer::new();
    let mut chip8 = CHIP8::new(display_buffer);

    chip8.load_sprites_into_memory();
    chip8.load_program_into_memory(&data);

    let keymap: HashMap<u8, u8> = HashMap::from([
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

    'running: loop {
        chip8.cycle();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => {
                    if let Some(&mapped_value) = keymap.get(&(keycode as u8)) {
                        chip8.set_key_press(mapped_value);

                        if let Some(on_next_key_press) = chip8.on_next_key_press.take() {
                            on_next_key_press(&mut chip8, mapped_value);
                            chip8.on_next_key_press = None;
                        }
                    }
                }
                Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => {
                    if let Some(&mapped_value) = keymap.get(&(keycode as u8)) {
                        chip8.unset_key_press(mapped_value);
                    }
                }
                _ => {}
            }
        }

        canvas.set_draw_color(Color::BLACK);
        canvas.clear();
        for y in 0..32 {
            for x in 0..64 {
                if chip8.display_buffer.is_on(Point::new(x, y)) {
                    canvas.set_draw_color(Color::GREEN);
                    let rectangle = Rect::new(x * 10, y * 10, SCALE as u32, SCALE as u32);
                    canvas.draw_rect(rectangle).unwrap();
                    canvas.fill_rect(rectangle).unwrap();
                }
            }
        }
        canvas.present();

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(())
}

// let mut _advance = false;
// if advance {
// dbg!(&chip8.display_buffer);
// dbg!(&chip8.stack);
// dbg!(&chip8.memory);
// dbg!(&chip8.v);
// dbg!(&chip8.i);
// dbg!(&chip8.delay_timer);
// dbg!(&chip8.sound_timer);
// dbg!(&chip8.pc);
// dbg!(&chip8.paused);
// dbg!(&chip8.keys_pressed);
//
//     advance = false;
// }

// Event::KeyDown {
//     keycode: Some(Keycode::Return),
//     ..
// } => {
//     _advance = true;
// }
