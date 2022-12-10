#![allow(dead_code, non_camel_case_types, non_snake_case, unused_parens)]
extern crate rand;
extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
// use std::time::Duration;

use std::fs;

#[derive(Debug)]
pub struct Memory {
    ram: [u8; 4096],
    registers: [u8; 16],
    ram_index: u16,
    registers_index: u16,
    i_reg: u16,
    stack: Vec<u16>,
    screen: [bool; 64 * 32],
    dt: u8,
    st: u8,
    buttons: [bool; 16],
}


impl Memory {
    fn new() -> Self {
        const FONTSET: [u8; 80 as usize] = [
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

        let mut mem = Memory {
            ram: [0u8; 4096],
            registers: [0u8; 16],
            ram_index: 0x200,
            registers_index: 0,
            i_reg: 0,
            stack: Vec::new(),
            screen: [false; 64 * 32],
            dt: 0,
            st: 0,
            buttons: [false; 16],
        };
        mem.ram[..80].copy_from_slice(&FONTSET);
        mem
    }
    fn set_ram(&mut self, path: String) {
        let bytes_ref: Result<Vec<u8>, std::io::Error> = fs::read(path);
        let bytes = bytes_ref.unwrap();
        let mut i = 0;
        while i < bytes.len() {
            self.ram[i + 0x200 as usize] = bytes[i];
            i += 1;
        }
    }
    fn get_instruction(&self) -> u16 {
        let a = self.ram[self.ram_index as usize] as u16;
        let b = self.ram[(self.ram_index + 1) as usize] as u16;
        let out: u16 = (a << 8) | b;
        out
    }

    fn execute(&mut self) {
        let command = self.get_instruction();
        // println!("{:4x}", command);

        let A = (command & 0xf000) >> 12;
        let B = (command & 0xf00) >> 8;
        let C = (command & 0xf0) >> 4;
        let D = (command & 0xf);
        match (A, B, C, D) {
            (0, 0, 0, 0) => {
                self.ram_index += 2;
                return;
            }

            (0, 0, 0xe, 0) => {
                self.screen = [false; 64 * 32];
                self.ram_index += 2;
                return;
            }

            (0, 0, 0xe, 0xe) => {
                self.ram_index = self.stack.pop().unwrap() + 2;
                return;
            }

            (0x1, _, _, _) => {
                self.ram_index = command & 0xfff;
                // println!("0x1NNN");
                return;
            }

            (0x2, _, _, _) => {
                let NNN = command & 0xfff;
                self.stack.push(self.ram_index);
                self.ram_index = NNN;
                return;
            }

            (0x3, _, _, _) => {
                let NN: u8 = (command & 0xff) as u8;
                if (self.registers[B as usize] == NN) {
                    self.ram_index += 2;
                }
                self.ram_index += 2;
                return;
            }

            (0x4, _, _, _) => {
                let NN: u8 = (command & 0xff) as u8;
                if (self.registers[B as usize] != NN) {
                    self.ram_index += 2;
                }
                self.ram_index += 2;
                return;
            }

            (0x5, _, _, 0) => {
                if (self.registers[B as usize] == self.registers[C as usize]) {
                    self.ram_index += 2;
                }
                self.ram_index += 2;
                return;
            }

            (0x6, _, _, _) => {
                let val = (command & 0xff) as u8;
                self.registers[B as usize] = val;
                self.ram_index += 2;
                return;
            }

            (0x7, _, _, _) => {
                let NN = (command & 0xff) as u16;
                let sum: u16 = ((self.registers[B as usize] as u16 + NN) & 0xff) as u16;
                self.registers[B as usize] = sum as u8;
                self.ram_index += 2;
                return;
            }

            (0x8, _, _, 0) => {
                self.registers[B as usize] = self.registers[C as usize];
                self.ram_index += 2;
                return;
            }

            (0x8, _, _, 1) => {
                self.registers[B as usize] |= self.registers[C as usize];
                self.ram_index += 2;
                return;
            }

            (0x8, _, _, 2) => {
                self.registers[B as usize] &= self.registers[C as usize];
                self.ram_index += 2;
                return;
            }

            (0x8, _, _, 3) => {
                self.registers[B as usize] ^= self.registers[C as usize];
                self.ram_index += 2;
                return;
            }

            (0x8, _, _, 4) => {
                let (temp, carry) =
                    self.registers[B as usize].overflowing_add(self.registers[C as usize]);
                self.registers[B as usize] = temp;
                if carry {
                    self.registers[0xf] = 1;
                } else {
                    self.registers[0xf] = 0;
                }
                self.ram_index += 2;
                return;
            }

            (0x8, _, _, 5) => {
                let (temp, carry) =
                    self.registers[B as usize].overflowing_sub(self.registers[C as usize]);
                self.registers[B as usize] = temp;
                if carry {
                    self.registers[0xf] = 0;
                } else {
                    self.registers[0xf] = 1;
                }
                self.ram_index += 2;
                return;
            }

            (0x8, _, _, 6) => {
                self.registers[0xf] = self.registers[C as usize] & 1;
                let temp = self.registers[C as usize] >> 1;
                self.registers[B as usize] = temp;
                self.ram_index += 2;
                return;
            }

            (0x8, _, _, 7) => {
                let (temp, carry) =
                    self.registers[C as usize].overflowing_sub(self.registers[B as usize]);
                self.registers[B as usize] = temp;
                self.registers[0xf] = (!carry) as u8;
                self.ram_index += 2;
                return;
            }

            (0x8, _, _, 0xe) => {
                self.registers[0xf] = self.registers[C as usize] & 128;
                let temp = self.registers[C as usize] << 1;
                self.registers[B as usize] = temp;
                self.ram_index += 2;
                return;
            }

            (0x9, _, _, 0) => {
                if (self.registers[B as usize] != self.registers[C as usize]) {
                    self.ram_index += 2;
                }
                self.ram_index += 2;
                return;
            }

            (0xa, _, _, _) => {
                self.i_reg = command & 0xfff;
                self.ram_index += 2;
                return;
            }

            (0xb, _, _, _) => {
                self.ram_index = (command & 0xfff) + self.registers[0] as u16;
                return;
            }

            (0xc, _, _, _) => {
                let NN = (command & 0xff) as u8;
                self.registers[B as usize] = NN & rand::random::<u8>();
                self.ram_index += 2;
                return;
            }

            (0xd, _, _, _) => {
                let x_coord = (self.registers[(B) as usize] % 64) as u16;
                let y_coord = (self.registers[(C) as usize] % 32) as u16;
                let N = D;
                let mut changed = false;
                for i in 0..N {
                    let addr = self.i_reg + i as u16;
                    let pixels = self.ram[addr as usize];
                    for x_line in 0..8 {
                        if (pixels & (0b10000000 >> x_line)) != 0 {
                            let x = (x_coord + x_line) as usize % 64;
                            let y = (y_coord + i) as usize % 32;
                            let idx = x + 64 * y;
                            changed |= self.screen[idx];
                            self.screen[idx] ^= true;
                        }
                    }
                }
                if changed {
                    self.registers[0xf] = 1;
                } else {
                    self.registers[0xf] = 0;
                }
                self.ram_index += 2;
                return;
            }

            (0xe, _, 9, 0xe) => {
                if self.buttons[self.registers[B as usize] as usize] {
                    self.ram_index += 2;
                }
                self.ram_index += 2;
                return;
            }

            (0xe, _, 0xa, 1) => {
                if !self.buttons[self.registers[B as usize] as usize] {
                    self.ram_index += 2;
                }
                self.ram_index += 2;
                return;
            }

            (0xf, _, 0, 7) => {
                self.registers[B as usize] = self.dt;
                self.ram_index += 2;
                return;
            }

            (0xf, _, 0, 0xa) => {
                for i in 0..16 {
                    if self.buttons[i] {
                        self.registers[B as usize] = i as u8;
                        self.ram_index += 2;
                        return;
                    }
                }
                return;
            }

            (0xf, _, 1, 5) => {
                self.dt = self.registers[B as usize];
                self.ram_index += 2;
                return;
            }

            (0xf, _, 1, 8) => {
                self.st = self.registers[B as usize];
                self.ram_index += 2;
                return;
            }

            (0xf, _, 1, 0xe) => {
                self.i_reg =
                    ((self.i_reg as u32 + self.registers[B as usize] as u32) & 0xffff) as u16;
                self.ram_index += 2;
                return;
            }

            (0xf, _, 2, 9) => {
                self.i_reg = (self.registers[B as usize] * 5) as u16;
                self.ram_index += 2;
                return;
            }

            (0xf, _, 3, 3) => {
                let h = (self.registers[B as usize] / 100) as u8;
                let t = ((self.registers[B as usize] / 10) % 10) as u8;
                let o = (self.registers[B as usize] % 10) as u8;
                self.ram[(self.i_reg) as usize] = h;
                self.ram[(self.i_reg + 1) as usize] = t;
                self.ram[(self.i_reg + 2) as usize] = o;

                self.ram_index += 2;
                return;
            }

            (0xf, _, 5, 5) => {
                for i in 0..=B {
                    self.ram[(self.i_reg + i) as usize] = self.registers[i as usize];
                }

                self.ram_index += 2;
                return;
            }

            (0xf, _, 6, 5) => {
                for i in 0..=B {
                    self.registers[i as usize] = self.ram[(i + self.i_reg) as usize];
                }
                self.ram_index += 2;
                return;
            }

            (_, _, _, _) => println!("unimplemented"),
        }
    }
}

struct display_control {
    canvas: sdl2::render::Canvas<sdl2::video::Window>,
    SDL: sdl2::Sdl,
}

impl display_control {
    fn new() -> display_control {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window("rust-sdl2 demo", 640, 320)
            .position_centered()
            .build()
            .unwrap();

        let mut canvas_temp = window.into_canvas().build().unwrap();
        canvas_temp.set_draw_color(Color::RGB(255, 255, 255));
        canvas_temp.clear();
        display_control {
            canvas: canvas_temp,
            SDL: sdl_context,
        }
    }

    fn present(&mut self) {
        self.canvas.present();
    }
    fn draw_square(&mut self, x: i32, y: i32) {
        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        match self
            .canvas
            .fill_rect(sdl2::rect::Rect::new(x * 10, y * 10, 10, 10))
        {
            Result::Ok(_) => {}
            Result::Err(e) => {
                println!("{}", e)
            }
        }
        self.canvas.set_draw_color(Color::RGB(255, 255, 255));
    }

    fn draw_frame(&mut self, arr: &[bool; 64 * 32]) {
        let mut index = 0;
        self.canvas.clear();
        loop {
            if index >= 64 * 32 {
                return;
            }
            if arr[index as usize] {
                self.draw_square((index % 64) as i32, (index / 64) as i32);
            }
            index += 1;
        }
    }
}

fn main() {
    let mut mem = Memory::new();
    mem.set_ram("chip8-test-suite.ch8".to_string());
    let mut my_canvas = display_control::new();
    my_canvas.present();
    let mut event_pump = my_canvas.SDL.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(n), ..
                } => {
                    let index: i8 = match n {
                        Keycode::Num1 => 0,
                        Keycode::Num2 => 1,
                        Keycode::Num3 => 2,
                        Keycode::Num4 => 3,
                        Keycode::Q => 4,
                        Keycode::W => 5,
                        Keycode::E => 6,
                        Keycode::R => 7,
                        Keycode::A => 8,
                        Keycode::S => 9,
                        Keycode::D => 10,
                        Keycode::F => 11,
                        Keycode::Z => 12,
                        Keycode::X => 13,
                        Keycode::C => 14,
                        Keycode::V => 15,
                        _ => -1,
                    };
                    // println!("{:?}: {}", n, index);
                    if index == -1 {
                        continue;
                    }

                    mem.buttons[index as usize] = true;
                }
                Event::KeyUp {
                    keycode: Some(n), ..
                } => {
                    let index: i8 = match n {
                        Keycode::Num1 => 0,
                        Keycode::Num2 => 1,
                        Keycode::Num3 => 2,
                        Keycode::Num4 => 3,
                        Keycode::Q => 4,
                        Keycode::W => 5,
                        Keycode::E => 6,
                        Keycode::R => 7,
                        Keycode::A => 8,
                        Keycode::S => 9,
                        Keycode::D => 10,
                        Keycode::F => 11,
                        Keycode::Z => 12,
                        Keycode::X => 13,
                        Keycode::C => 14,
                        Keycode::V => 15,
                        _ => -1,
                    };
                    // println!("{:?}: {}", n, index);
                    if index == -1 {
                        continue;
                    }

                    mem.buttons[index as usize] = false;
                }
                _ => {}
            }
        }
        mem.execute();
        if (mem.dt > 0) {
            mem.dt -= 1;
        }
        if (mem.st > 0) {
            mem.st -= 1;
        }
        my_canvas.draw_frame(&mem.screen);
        my_canvas.present();
        // std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
