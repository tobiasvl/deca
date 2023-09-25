#![warn(missing_docs)]

//! This module contains the entire "CPU" part of Deca's CHIP-8 interpreter.
pub use decasm;
pub use decasm::Instruction;
use decasm::{Byte, Register};
use itertools::Either;
use octopt::LoResDxy0Behavior;
pub use octopt::{Options, Quirks};

mod display;
pub use display::Display;

use ux::u4;

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
    #[must_use]
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

    /// Set variable register's value.
    ///
    /// Note that this is just a convenience method you can use if you have a [`Register`]; you can also just use [`self::v`] directly.
    pub fn set_register(&mut self, register: Register, value: u8) {
        self.v[usize::from(register)] = value;
    }

    /// Get variable register's value.
    ///
    /// Note that this is just a convenience method you can use if you have a [`Register`]; you can also just use [`self::v`] directly.
    #[must_use]
    pub fn register(&self, register: Register) -> u8 {
        self.v[usize::from(register)]
    }

    /// Fetch the next opcode from memory and increment the Program Counter.
    ///
    /// Note that this method does not guarantee that the Program Counter will point at the
    /// next opcode afterwards, as the opcode might have an immediate operand which is not
    /// fetched until decoding (in the case of an [`Instruction::SetIndexLong`]).
    pub fn fetch(&mut self) -> u16 {
        let opcode = (u16::from(self.memory[self.pc as usize]) << 8)
            | u16::from(self.memory[self.pc.wrapping_add(1) as usize]);
        self.pc = self.pc.wrapping_add(2);
        opcode
    }

    /// Decode a CHIP-8 opcode into an `[Instruction]`.
    ///
    /// # Errors
    ///
    /// Returns an `Err` with an error message if the opcode doesn't map to an instruction.
    ///
    /// # Examples
    ///
    /// ```
    /// # use deca::{Chip8, Instruction, Options};
    /// let mut chip8 = Chip8::default();
    /// assert_eq!(chip8.decode(0x00E0), Ok(Instruction::Clear));
    /// ```
    pub fn decode(&mut self, opcode: u16) -> Result<Instruction, String> {
        let _foo = Instruction::try_from(opcode);
        match _foo {
            Ok(opcode) => match opcode {
                Instruction::SetIndexLong => Ok(Instruction::SetIndex(self.fetch())),
                instruction => Ok(instruction),
            },
            Err(e) => Err(format!("{e} at PC {}", self.pc)),
        }
        //Ok(match Instruction::try_from(opcode)? {
        //    Instruction::SetIndexLong => Instruction::SetIndex(self.fetch()),
        //    instruction => instruction,
        //})
    }

    /// Execute a CHIP-8 `[Instruction]`.
    ///
    /// # Errors
    ///
    /// Returns an `Err` with an error message if the opcode caused a runtime CHIP-8 error.
    ///
    /// # Panics
    ///
    /// Should only panic if executing an unimplemented instruction.
    // Allow unwrapping; should only be used when casting eg. a u4 into a larger number type like usize
    #[allow(clippy::too_many_lines, clippy::unwrap_used)]
    pub fn execute(&mut self, instruction: Instruction) -> Result<(), String> {
        match instruction {
            #![allow(clippy::match_same_arms, clippy::cast_possible_truncation)]
            Instruction::Exit(Some(n)) => {
                return Err(format!("Interpreter exited with exit code {n}"))
            }
            Instruction::Exit(None) => return Err("Interpreter exited".to_string()),
            Instruction::ScrollUp(n) => self.display.scroll_up(u8::from(n)),
            Instruction::ScrollDown(n) => self.display.scroll_down(u8::from(n)),
            Instruction::Clear => self.display.clear(false),
            Instruction::Return => {
                if self.sp == 0 {
                    return Err(String::from("Attempted pop from empty stack"));
                }
                self.pc = self.stack[self.sp];
                self.sp -= 1;
            }
            Instruction::ToggleLoadStoreQuirk => {
                self.options.quirks.load_store =
                    Some(!self.options.quirks.load_store.unwrap_or(false));
            }
            Instruction::ScrollRight => self.display.scroll_right(4),
            Instruction::ScrollLeft => self.display.scroll_left(4),
            Instruction::LoRes => self
                .display
                .lores(self.options.quirks.res_clear == Some(true)),
            Instruction::HiRes => self
                .display
                .hires(self.options.quirks.res_clear == Some(true)),
            Instruction::CallMachineCode(_) => {
                return Err(String::from("Machine code is not supported"))
            }
            Instruction::Jump(nnn) => self.pc = u16::from(nnn),
            Instruction::Call(nnn) => {
                self.sp += 1;
                if self.sp >= self.stack.len() {
                    return Err(String::from("Stack limit exceeded"));
                }
                self.stack[self.sp] = self.pc;
                self.pc = u16::from(nnn);
            }
            Instruction::SkipIfEqual(Register(x), Byte::Immediate(kk)) => {
                if self.v[usize::try_from(x).unwrap()] == kk {
                    self.skip();
                }
            }
            Instruction::SkipIfNotEqual(Register(x), Byte::Immediate(kk)) => {
                if self.v[usize::try_from(x).unwrap()] != kk {
                    self.skip();
                }
            }
            Instruction::SkipIfEqual(Register(x), Byte::Register(Register(y))) => {
                if self.v[usize::try_from(x).unwrap()] == self.v[usize::try_from(y).unwrap()] {
                    self.skip();
                }
            }
            Instruction::Set(Register(x), Byte::Immediate(kk)) => {
                self.v[usize::try_from(x).unwrap()] = kk;
            }
            Instruction::Add(Register(x), Byte::Immediate(kk)) => {
                self.v[usize::try_from(x).unwrap()] =
                    self.v[usize::try_from(x).unwrap()].wrapping_add(kk);
            }
            Instruction::Set(Register(x), Byte::Register(Register(y))) => {
                self.v[usize::try_from(x).unwrap()] = self.v[usize::try_from(y).unwrap()];
            }
            Instruction::Or(Register(x), Register(y)) => {
                self.v[usize::try_from(x).unwrap()] |= self.v[usize::try_from(y).unwrap()];
            }
            Instruction::And(Register(x), Register(y)) => {
                self.v[usize::try_from(x).unwrap()] &= self.v[usize::try_from(y).unwrap()];
            }
            Instruction::Xor(Register(x), Register(y)) => {
                self.v[usize::try_from(x).unwrap()] ^= self.v[usize::try_from(y).unwrap()];
            }
            Instruction::Add(Register(x), Byte::Register(Register(y))) => {
                self.v[0xF] = if (u16::from(self.v[usize::try_from(x).unwrap()])
                    + u16::from(self.v[usize::try_from(y).unwrap()]))
                    > 0xFF
                {
                    1
                } else {
                    0
                };
                // FIXME
                self.v[usize::try_from(x).unwrap()] = self.v[usize::try_from(x).unwrap()]
                    .wrapping_add(self.v[usize::try_from(y).unwrap()]);
            }
            Instruction::Sub(Register(x), Register(y)) => {
                self.v[0xF] = match self.v[usize::from(u8::from(x))]
                    .overflowing_sub(self.v[usize::try_from(y).unwrap()])
                {
                    (_, true) => 0,
                    _ => 1,
                };
                self.v[usize::try_from(x).unwrap()] = self.v[usize::try_from(x).unwrap()]
                    .wrapping_sub(self.v[usize::try_from(y).unwrap()]);
            }
            Instruction::ShiftLeft(Register(x), Register(y)) => {
                dbg!(self.options.quirks.shift);
                let operand: u8 = if self.options.quirks.shift == Some(true) {
                    self.v[usize::try_from(x).unwrap()]
                } else {
                    self.v[usize::try_from(y).unwrap()]
                };
                self.v[0xF] = operand & 1;
                self.v[usize::try_from(x).unwrap()] = operand >> 1;
            }
            Instruction::SubReverse(Register(x), Register(y)) => {
                self.v[0xF] = match self.v[usize::from(u8::from(y))]
                    .overflowing_sub(self.v[usize::try_from(x).unwrap()])
                {
                    (_, true) => 0,
                    _ => 1,
                };
                self.v[usize::try_from(x).unwrap()] = self.v[usize::from(u8::from(y))]
                    .wrapping_sub(self.v[usize::try_from(x).unwrap()]);
            }
            Instruction::ShiftRight(Register(x), Register(y)) => {
                dbg!(self.options.quirks.shift);
                let operand: u8 = if self.options.quirks.shift == Some(true) {
                    self.v[usize::try_from(x).unwrap()]
                } else {
                    self.v[usize::try_from(y).unwrap()]
                };

                self.v[0xF] = (operand & 0x80) >> 7;
                self.v[usize::try_from(x).unwrap()] = operand << 1;
            }
            Instruction::SkipIfNotEqual(Register(x), Byte::Register(Register(y))) => {
                if self.v[usize::try_from(x).unwrap()] != self.v[usize::try_from(y).unwrap()] {
                    self.skip();
                }
            }
            Instruction::SetIndex(nnn) => self.i = nnn,
            Instruction::JumpRelative(nnn) => {
                let jump_register = u16::from(if self.options.quirks.jump0 == Some(true) {
                    self.v[usize::try_from((nnn & 0x0F00) >> 8).unwrap()]
                } else {
                    self.v[0]
                });
                self.pc = jump_register + nnn;
            }
            Instruction::Random(Register(x), kk) => {
                self.v[usize::try_from(x).unwrap()] = fastrand::u8(..) & kk;
            }
            Instruction::Draw(Register(x), Register(y), n) => {
                let mut width: u8 = 8;
                let mut height: u8 = n.into();
                let mut address = self.i;

                if n == u4::new(0) {
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
                        self.v[0xF] = self.display.draw(
                            sprite,
                            self.v[usize::try_from(x).unwrap()],
                            self.v[usize::try_from(y).unwrap()],
                        );
                    }
                }
                self.display.active_plane = active_plane;
            }
            Instruction::SkipKey(Register(x)) => {
                if self.keyboard[usize::from(self.v[usize::try_from(x).unwrap()])] {
                    self.skip();
                }
            }
            Instruction::SkipNotKey(Register(x)) => {
                if !self.keyboard[usize::from(self.v[usize::try_from(x).unwrap()])] {
                    self.skip();
                }
            }
            Instruction::SoundStuff => todo!(),
            Instruction::LoadDelay(Register(x)) => self.v[usize::try_from(x).unwrap()] = self.delay,
            Instruction::BlockKey(Register(x)) => {
                self.pc = self.pc.wrapping_sub(2);
                for key in 0..self.keyboard.len() {
                    if self.keyboard[key] {
                        self.v[usize::try_from(x).unwrap()] = key as u8;
                        self.skip();
                        self.keyboard[key] = false;
                        break;
                    }
                }
            }
            Instruction::SelectPlane(n) => {
                let n: u8 = n.into();
                if n > 3 {
                    return Err(format!(
                        "XO-CHIP currently only supports 3 planes, attempted to select plane {}",
                        n
                    ));
                }
                self.display.plane(n);
            }
            Instruction::SoundStuffTwo => todo!(),
            Instruction::SetDelay(Register(x)) => self.delay = self.v[usize::try_from(x).unwrap()],
            Instruction::SetSound(Register(x)) => self.sound = self.v[usize::try_from(x).unwrap()],
            Instruction::AddRegisterToIndex(Register(x)) => {
                self.i = self
                    .i
                    .wrapping_add(u16::from(self.v[usize::try_from(x).unwrap()]));
            }
            Instruction::FontCharacter(Register(x)) => {
                self.i = 0x50 + u16::from(self.v[usize::try_from(x).unwrap()] * 5);
            }
            Instruction::BigFontCharacter(Register(x)) => {
                self.i = 0xA0 + u16::from(self.v[usize::try_from(x).unwrap()] * 10);
            }
            Instruction::Bcd(Register(x)) => {
                let vx: u8 = self.v[usize::try_from(x).unwrap()];
                self.memory[self.i as usize] = vx / 100;
                self.memory[self.i as usize + 1] = (vx / 10) % 10;
                self.memory[self.i as usize + 2] = vx % 10;
            }
            Instruction::Store(Register(x)) => {
                let mut i = self.i;
                let x = usize::try_from(x).unwrap();
                for n in 0..=x {
                    self.memory[i as usize] = self.v[n];
                    i = i.wrapping_add(1);
                }
                if self.options.quirks.load_store != Some(true) {
                    self.i = i;
                }
            }
            Instruction::Load(Register(x)) => {
                let mut i = self.i;
                let x = usize::try_from(x).unwrap();

                for n in 0..=x {
                    self.v[n] = self.memory[i as usize];
                    i = i.wrapping_add(1);
                }
                if self.options.quirks.load_store != Some(true) {
                    self.i = i;
                }
            }
            Instruction::StoreRange(Register(x), Register(y)) => {
                let mut i = self.i;
                // FIXME this can't be necessary...
                let (x, y): (usize, usize) = (usize::from(u8::from(x)), usize::from(u8::from(y)));
                for n in if x <= y {
                    Either::Left(x..=y)
                } else {
                    Either::Right((y..=x).rev())
                } {
                    self.memory[i as usize] = self.v[n];
                    i = i.wrapping_add(1);
                }
            }
            Instruction::LoadRange(Register(x), Register(y)) => {
                let mut i = self.i;
                // FIXME this can't be necessary...
                let (x, y): (usize, usize) = (usize::from(u8::from(x)), usize::from(u8::from(y)));
                for n in if x <= y {
                    Either::Left(x..=y)
                } else {
                    Either::Right((y..=x).rev())
                } {
                    self.v[n] = self.memory[i as usize];
                    i = i.wrapping_add(1);
                }
            }
            Instruction::StoreFlags(Register(x)) => {
                let x: usize = u8::from(x).into();
                for n in 0..=x {
                    self.flags[n] = self.v[n];
                }
            }
            Instruction::LoadFlags(Register(x)) => {
                let x: usize = u8::from(x).into();
                for n in 0..=x {
                    self.v[n] = self.flags[n];
                }
            }
            Instruction::SetIndexLong => self.i = self.fetch(),
            _ => panic!("Unknown instruction {:?}", instruction),
        }
        Ok(())
    }

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
            let _addr = self.pc;
            let opcode = self.fetch();
            //dbg!(format!("{:02x}: {:04x}", _addr, opcode));
            let instruction = self.decode(opcode)?;
            self.execute(instruction)?;
            if self.options.quirks.vblank == Some(true) && (0xD000..=0xDFFF).contains(&opcode) {
                break;
            }
        }
        Ok(())
    }

    fn skip(&mut self) {
        let opcode = self.fetch();
        if let Ok(instruction) = self.decode(opcode) {
            if instruction == Instruction::SetIndexLong {
                let _ = self.fetch();
            }
        }
    }
}

impl Default for Chip8 {
    fn default() -> Self {
        Self::new(Options::default())
    }
}
