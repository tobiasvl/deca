#![warn(missing_docs)]

//! This module contains the entire "CPU" part of Deca's CHIP-8 interpreter.
use itertools::Either;
use octopt::{LoResDxy0Behavior, Options, Quirks};

mod display;
pub use display::Display;

/// A struct for holding the state of the CHIP-8 interpreter.
pub struct Chip8 {
    /// The Program Counter, which contains the index in [`memory`] that's currently executed.
    pub pc: u16,
    /// The Stack Pointer, which contains the index in [`stack`] that's currently the top of the stack.
    pub sp: usize,
    /// The call stack.
    pub stack: [u16; 16],
    /// The CHIP-8 memory
    pub memory: [u8; 65536],
    /// CHIP-8's index register.
    pub i: u16,
    /// CHIP-8's variable registers.
    pub v: [u8; 16],
    /// SCHIP's user flags.
    pub flags: [u8; 16],
    /// The delay timer. If non-zero, this should count down at 60 Hz until it reaches zero.
    pub delay: u8,
    /// The sound timer. If non-zero, this should count down at 60 Hz until it reaches zero. While it is
    /// non-zero, an audible sound or visual indication should be present.
    pub sound: u8,
    /// CHIP-8's display buffer.
    pub display: Display,
    /// The configuration options for how this CHIP-8 program should behave.
    pub options: Options,
    /// The current state of the CHIP-8 hexadecimal keypad.
    pub keyboard: [bool; 16],
}

impl Chip8 {
    /// Create a new CHIP-8 interpreter with the given [`octopt::Options`].
    pub fn new(options: Options) -> Chip8 {
        let mut memory = [0; 65536];

        let (font, big_font) = &options.font_style.get_font_data();

        memory[0x50..(0x50 + font.len())].clone_from_slice(&font[..]);

        if let Some(big_font) = big_font {
            memory[(0x50 + font.len())..(0x50 + font.len() + big_font.len())]
                .clone_from_slice(&big_font[..]);
        }

        Chip8 {
            pc: options.start_address.unwrap_or(0x200),
            sp: 0,
            stack: [0; 16],
            memory,
            i: 0,
            v: [0; 16],
            flags: [0; 16],
            delay: 0,
            sound: 0,
            display: Display::new(),
            options,
            keyboard: [false; 16],
        }
    }

    /// Change quirk settings
    pub fn set_quirks(&mut self, quirks: Quirks) {
        self.options.quirks = quirks;
    }

    /// Read CHIP-8 program ("ROM") into memory
    pub fn read_rom(&mut self, rom: &[u8]) {
        self.memory[0x200..][..rom.len()].copy_from_slice(rom);
    }

    /// Fetch the next instruction from memory.
    pub fn fetch(&mut self) -> u16 {
        let opcode = (u16::from(self.memory[self.pc as usize]) << 8)
            | u16::from(self.memory[self.pc.wrapping_add(1) as usize]);
        self.pc = self.pc.wrapping_add(2);
        opcode
    }

    /// Decode and execute a CHIP-8 opcode.
    /// TODO: Change this function's name? It doesn't just decode.
    ///
    /// # Errors
    ///
    /// Returns an `Err` with an error message if the opcode caused a runtime CHIP-8 error.
    ///
    /// # Panics
    ///
    /// Should only panic if executing an unimplemented opcode.
    pub fn decode(&mut self, opcode: u16) -> Result<(), String> {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let vx = self.v[x];
        let vy = self.v[y];
        let nnn = opcode & 0x0FFF;
        let kk = (opcode & 0x00FF) as u8;
        let n = (opcode & 0x000F) as u8;

        let op_1 = (opcode & 0xF000) >> 12;
        let op_2 = (opcode & 0x0F00) >> 8;
        let op_3 = (opcode & 0x00F0) >> 4;
        let op_4 = opcode & 0x000F;

        match (op_1, op_2, op_3, op_4) {
            #![allow(clippy::match_same_arms, clippy::cast_possible_truncation)]
            // from chip8run:
            (0x0, 0x0, 0x1, _) => return Err(format!("Interpreter exited with exit code {}", n)),
            // https://chip-8.github.io/extensions/#super-chip-with-scroll-up
            (0x0, 0x0, 0xB, _) => self.display.scroll_up(n),
            (0x0, 0x0, 0xC, _) => self.display.scroll_down(n),
            (0x0, 0x0, 0xD, _) => self.display.scroll_up(n),
            (0x0, 0x0, 0xE, 0x0) => self.display.clear(false),
            (0x0, 0x0, 0xE, 0xE) => {
                if self.sp == 0 {
                    return Err(String::from("Attempted pop from empty stack"));
                }
                self.pc = self.stack[self.sp];
                self.sp -= 1;
            }
            // from chip8run:
            (0x0, 0x0, 0xF, 0xA) => {
                self.options.quirks.load_store =
                    Some(!self.options.quirks.load_store.unwrap_or(false));
            }
            (0x0, 0x0, 0xF, 0xB) => self.display.scroll_right(4),
            (0x0, 0x0, 0xF, 0xC) => self.display.scroll_left(4),
            (0x0, 0x0, 0xF, 0xD) => return Err(String::from("Interpreter exited")),
            (0x0, 0x0, 0xF, 0xE) => self
                .display
                .lores(self.options.quirks.res_clear == Some(true)),
            (0x0, 0x0, 0xF, 0xF) => self
                .display
                .hires(self.options.quirks.res_clear == Some(true)),
            (0x0, _, _, _) => return Err(String::from("Machine code is not supported")),
            (0x1, _, _, _) => self.pc = nnn,
            (0x2, _, _, _) => {
                self.sp += 1;
                if self.sp >= self.stack.len() {
                    return Err(String::from("Stack limit exceeded"));
                }
                self.stack[self.sp] = self.pc;
                self.pc = nnn;
            }
            (0x3, _, _, _) => {
                if vx == kk {
                    self.skip();
                }
            }
            (0x4, _, _, _) => {
                if vx != kk {
                    self.skip();
                }
            }
            (0x5, _, _, 0x0) => {
                if vx == vy {
                    self.skip();
                }
            }
            (0x5, _, _, 0x2) => {
                let mut i = self.i;
                for n in if x <= y {
                    Either::Left(x..=y)
                } else {
                    Either::Right((y..=x).rev())
                } {
                    self.memory[i as usize] = self.v[n];
                    i = i.wrapping_add(1);
                }
            }
            (0x5, _, _, 0x3) => {
                let mut i = self.i;
                for n in if x <= y {
                    Either::Left(x..=y)
                } else {
                    Either::Right((y..=x).rev())
                } {
                    self.v[n] = self.memory[i as usize];
                    i = i.wrapping_add(1);
                }
            }
            (0x6, _, _, _) => self.v[x] = kk,
            (0x7, _, _, _) => self.v[x] = vx.wrapping_add(kk),
            (0x8, _, _, 0) => self.v[x] = vy,
            (0x8, _, _, 1) => self.v[x] |= vy,
            (0x8, _, _, 2) => self.v[x] &= vy,
            (0x8, _, _, 3) => self.v[x] ^= vy,
            (0x8, _, _, 4) => {
                self.v[0xF] = if (u16::from(vx) + u16::from(vy)) > 0xFF {
                    1
                } else {
                    0
                };
                self.v[x] = vx.wrapping_add(vy);
            }
            (0x8, _, _, 5) => {
                self.v[0xF] = match vx.overflowing_sub(vy) {
                    (_, true) => 0,
                    _ => 1,
                };
                self.v[x] = vx.wrapping_sub(vy);
            }
            (0x8, _, _, 0x6) => {
                let operand = if self.options.quirks.shift == Some(true) {
                    vx
                } else {
                    vy
                };
                self.v[0xF] = operand & 1;
                self.v[x] = operand >> 1;
            }
            (0x8, _, _, 0x7) => {
                self.v[0xF] = match vy.overflowing_sub(vx) {
                    (_, true) => 0,
                    _ => 1,
                };
                self.v[x] = vy.wrapping_sub(vx);
            }
            (0x8, _, _, 0xE) => {
                let operand = if self.options.quirks.shift == Some(true) {
                    vx
                } else {
                    vy
                };

                self.v[0xF] = (operand & 0x80) >> 7;
                self.v[x] = operand << 1;
            }
            (0x9, _, _, 0x0) => {
                if vx != vy {
                    self.skip();
                }
            }
            (0xA, _, _, _) => self.i = nnn,
            (0xB, _, _, _) => {
                let jump_register = u16::from(if self.options.quirks.jump0 == Some(true) {
                    vx
                } else {
                    self.v[0]
                });
                self.pc = jump_register + nnn;
            }
            (0xC, _x, _, _) => {
                self.v[x] = fastrand::u8(..) & kk;
            }
            (0xD, _, _, _) => {
                let mut width = 8;
                let mut height = n;
                let mut address = self.i;

                if n == 0 {
                    if self.display.hires
                        || self.options.quirks.lores_dxy0 == Some(LoResDxy0Behavior::BigSprite)
                    {
                        width = 16;
                        height = 16;
                    } else if self.options.quirks.lores_dxy0 == Some(LoResDxy0Behavior::TallSprite)
                    {
                        height = 16;
                    } else {
                        return Ok(());
                    }
                }

                let active_plane = self.display.active_plane;
                for color in 1..=2 {
                    if active_plane & color != 0 {
                        let mut sprite = Vec::<Vec<u8>>::new();
                        for _y in 0..height {
                            let mut row = Vec::<u8>::new();
                            let mut byte = self.memory[address as usize];
                            for x in 0..width {
                                if x == 8 {
                                    address = address.wrapping_add(1);
                                    byte = self.memory[address as usize];
                                }
                                row.push((byte << (x % 8)) >> 7);
                            }
                            sprite.push(row);
                            address = address.wrapping_add(1);
                        }

                        self.display.active_plane = color;
                        self.v[0xF] = self.display.draw(sprite, vx, vy);
                    }
                }
                self.display.active_plane = active_plane;
            }
            (0xE, _x, 0x9, 0xE) => {
                if self.keyboard[vx as usize] {
                    self.skip();
                }
            }
            (0xE, _x, 0xA, 0x1) => {
                if !self.keyboard[vx as usize] {
                    self.skip();
                }
            }
            (0xF, 0x0, 0x0, 0x0) => self.i = self.fetch(),
            (0xF, 0x0, 0x0, 0x2) => todo!(),
            (0xF, _, 0x0, 0x7) => self.v[x] = self.delay,
            (0xF, _x, 0x0, 0xA) => {
                // todo!();
                self.pc = self.pc.wrapping_sub(2);
                for key in 0..self.keyboard.len() {
                    if self.keyboard[key] {
                        self.v[x] = key as u8;
                        self.skip();
                        self.keyboard[key] = false;
                        break;
                    }
                }
            }
            (0xF, _, 0x0, 0x1) => {
                if x > 3 {
                    return Err(format!(
                        "XO-CHIP currently only supports 3 planes, attempted to select plane {}",
                        x
                    ));
                }
                self.display.plane(x as u8);
            }
            (0xF, _n, 0x3, 0xA) => todo!(),
            (0xF, _, 0x1, 0x5) => self.delay = vx,
            (0xF, _, 0x1, 0x8) => self.sound = vx,
            (0xF, _, 0x1, 0xE) => self.i = self.i.wrapping_add(u16::from(vx)),
            (0xF, _x, 0x2, 0x9) => {
                self.i = 0x50 + u16::from(vx * 5);
            }
            (0xF, _x, 0x3, 0x0) => {
                self.i = 0xA0 + u16::from(vx * 10);
            }
            (0xF, _x, 0x3, 0x3) => {
                self.memory[self.i as usize] = vx / 100;
                self.memory[self.i as usize + 1] = (vx / 10) % 10;
                self.memory[self.i as usize + 2] = vx % 10;
            }
            (0xF, _, 0x5, 0x5) => {
                let mut i = self.i;
                for n in 0..=x {
                    self.memory[i as usize] = self.v[n];
                    i = i.wrapping_add(1);
                }
                if self.options.quirks.load_store != Some(true) {
                    self.i = i;
                }
            }
            (0xF, _, 0x6, 0x5) => {
                let mut i = self.i;
                for n in 0..=x {
                    self.v[n] = self.memory[i as usize];
                    i = i.wrapping_add(1);
                }
                if self.options.quirks.load_store != Some(true) {
                    self.i = i;
                }
            }
            (0xF, _x, 0x7, 0x5) => {
                for n in 0..=x {
                    self.flags[n] = self.v[n];
                }
            }
            (0xF, _x, 0x8, 0x5) => {
                for n in 0..=x {
                    self.v[n] = self.flags[n];
                }
            }
            _ => return Err(format!("Unknown opcode {:#06x}", opcode)),
        }
        Ok(())
    }

    //pub fn execute(&mut self, _: Fn) {

    //}

    /// Run the CHIP-8 CPU for the given number of ticks.
    ///
    /// # Errors
    ///
    /// Returns `Err` if a runtime CHIP-8 error occurs during execution.
    pub fn run(&mut self, tickrate: u16) -> Result<(), String> {
        if self.options.quirks.delay_wrap != Some(true) && self.delay > 0 {
            self.delay = self.delay.wrapping_sub(1);
        }
        if self.sound > 0 {
            self.sound -= 1;
        }
        for _ in 0..tickrate {
            //self.execute(self.decode(self.fetch()));
            //let addr = self.pc;
            let opcode = self.fetch();
            //println!("{:02x}: {:04x}", addr, opcode);
            self.decode(opcode)?;
            if self.options.quirks.vblank == Some(true) && opcode >= 0xD000 && opcode <= 0xDFFF {
                break;
            }
        }
        Ok(())
    }

    fn skip(&mut self) {
        if self.fetch() == 0xF000 {
            let _ = self.fetch();
        }
    }
}

impl Default for Chip8 {
    fn default() -> Self {
        Self::new(Options::default())
    }
}
