use itertools::Either;

mod display;
use display::Display;

pub struct Chip8 {
    pub pc: u16,
    pub sp: usize,
    pub stack: [u16; 16],
    pub memory: [u8; 65536],
    pub i: u16,
    pub v: [u8; 16],
    pub flags: [u8; 16],
    pub delay: u8,
    pub sound: u8,
    pub display: Display,
    pub quirks: Quirks,
    pub keyboard: [bool; 16],
}

#[derive(Default, Debug, PartialEq)]
pub struct Quirks {
    // shift VX instead of VY
    pub shift: bool,
    // don't increment I after load/store
    pub loadstore: bool,
    // jump to VM instead of V0
    pub jump0: bool,
    // scratch VF after logic ops
    pub logic: bool,
    // Clip sprites instead of wrapping
    pub clip: bool,
    pub vblank: bool,
    pub resclear: bool,
    pub delaywrap: bool,
    pub multicollision: bool,
    pub loresbigsprite: bool,
    pub lorestallsprite: bool,
    pub max_rom: u16,
}

impl Chip8 {
    pub fn new() -> Chip8 {
        let mut memory = [0; 65536];

        let font = [
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

        memory[0x50..(0x50 + font.len())].clone_from_slice(&font[..]);

        let big_font = [
            0xFF, 0xFF, 0xC3, 0xC3, 0xC3, 0xC3, 0xC3, 0xC3, 0xFF, 0xFF, // 0
            0x18, 0x78, 0x78, 0x18, 0x18, 0x18, 0x18, 0x18, 0xFF, 0xFF, // 1
            0xFF, 0xFF, 0x03, 0x03, 0xFF, 0xFF, 0xC0, 0xC0, 0xFF, 0xFF, // 2
            0xFF, 0xFF, 0x03, 0x03, 0xFF, 0xFF, 0x03, 0x03, 0xFF, 0xFF, // 3
            0xC3, 0xC3, 0xC3, 0xC3, 0xFF, 0xFF, 0x03, 0x03, 0x03, 0x03, // 4
            0xFF, 0xFF, 0xC0, 0xC0, 0xFF, 0xFF, 0x03, 0x03, 0xFF, 0xFF, // 5
            0xFF, 0xFF, 0xC0, 0xC0, 0xFF, 0xFF, 0xC3, 0xC3, 0xFF, 0xFF, // 6
            0xFF, 0xFF, 0x03, 0x03, 0x06, 0x0C, 0x18, 0x18, 0x18, 0x18, // 7
            0xFF, 0xFF, 0xC3, 0xC3, 0xFF, 0xFF, 0xC3, 0xC3, 0xFF, 0xFF, // 8
            0xFF, 0xFF, 0xC3, 0xC3, 0xFF, 0xFF, 0x03, 0x03, 0xFF, 0xFF, // 9
            0x7E, 0xFF, 0xC3, 0xC3, 0xC3, 0xFF, 0xFF, 0xC3, 0xC3, 0xC3, // A
            0xFC, 0xFC, 0xC3, 0xC3, 0xFC, 0xFC, 0xC3, 0xC3, 0xFC, 0xFC, // B
            0x3C, 0xFF, 0xC3, 0xC0, 0xC0, 0xC0, 0xC0, 0xC3, 0xFF, 0x3C, // C
            0xFC, 0xFE, 0xC3, 0xC3, 0xC3, 0xC3, 0xC3, 0xC3, 0xFE, 0xFC, // D
            0xFF, 0xFF, 0xC0, 0xC0, 0xFF, 0xFF, 0xC0, 0xC0, 0xFF, 0xFF, // E
            0xFF, 0xFF, 0xC0, 0xC0, 0xFF, 0xFF, 0xC0, 0xC0, 0xC0, 0xC0, // F
        ];

        memory[(0x50 + font.len())..(0x50 + font.len() + big_font.len())]
            .clone_from_slice(&big_font[..]);

        // TODO this is very ugly, implement Default for Quirks with correct max_rom and
        // loresbigsprite
        let quirks = Quirks {
            max_rom: 65024,
            loresbigsprite: true,
            resclear: true,
            ..Default::default()
        };

        Chip8 {
            pc: 0x200,
            sp: 0,
            stack: [0; 16],
            memory,
            i: 0,
            v: [0; 16],
            flags: [0; 16],
            delay: 0,
            sound: 0,
            display: Display::new(),
            quirks,
            keyboard: [false; 16],
        }
    }

    pub fn set_quirks(&mut self, quirks: Quirks) {
        self.quirks = quirks;
    }

    pub fn read_rom(&mut self, rom: &[u8]) {
        self.memory[0x200..][..rom.len()].copy_from_slice(rom);
    }

    pub fn fetch(&mut self) -> u16 {
        let opcode = ((self.memory[self.pc as usize] as u16) << 8)
            | self.memory[self.pc.wrapping_add(1) as usize] as u16;
        self.pc = self.pc.wrapping_add(2);
        opcode
    }

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
            (0x0, 0x0, 0xC, _) => self.display.scroll_down(n),
            (0x0, 0x0, 0xD, _) => self.display.scroll_up(n),
            (0x0, 0x0, 0xE, 0x0) => self.display.clear(),
            (0x0, 0x0, 0xE, 0xE) => {
                self.pc = self.stack[self.sp];
                self.sp = self.sp.wrapping_sub(1);
            }
            (0x0, 0x0, 0xF, 0xB) => self.display.scroll_right(4),
            (0x0, 0x0, 0xF, 0xC) => self.display.scroll_left(4),
            (0x0, 0x0, 0xF, 0xD) => std::process::exit(0),
            (0x0, 0x0, 0xF, 0xE) => self.display.lores(self.quirks.resclear),
            (0x0, 0x0, 0xF, 0xF) => self.display.hires(self.quirks.resclear),
            (0x0, _, _, _) => return Err(String::from("Machine code is not supported")),
            (0x1, _, _, _) => self.pc = nnn,
            (0x2, _, _, _) => {
                self.sp = self.sp.wrapping_add(1);
                self.stack[self.sp] = self.pc;
                self.pc = nnn;
            }
            (0x3, _, _, _) => {
                if vx == kk {
                    self.skip()
                }
            }
            (0x4, _, _, _) => {
                if vx != kk {
                    self.skip()
                }
            }
            (0x5, _, _, 0x0) => {
                if vx == vy {
                    self.skip()
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
                self.v[0xF] = if (vx as u16 + vy as u16) > 0xFF { 1 } else { 0 };
                self.v[x] = vx.wrapping_add(vy)
            }
            (0x8, _, _, 5) => {
                self.v[0xF] = match vx.overflowing_sub(vy) {
                    (_, true) => 0,
                    _ => 1,
                };
                self.v[x] = vx.wrapping_sub(vy)
            }
            (0x8, _, _, 0x6) => {
                let operand = if self.quirks.shift { vx } else { vy };
                self.v[0xF] = operand & 1;
                self.v[x] = operand >> 1;
            }
            (0x8, _, _, 0x7) => {
                self.v[0xF] = match vy.overflowing_sub(vx) {
                    (_, true) => 0,
                    _ => 1,
                };
                self.v[x] = vy.wrapping_sub(vx)
            }
            (0x8, _, _, 0xE) => {
                let operand = if self.quirks.shift { vx } else { vy };

                self.v[0xF] = (operand & 0x80) >> 7;
                self.v[x] = operand << 1
            }
            (0x9, _, _, 0x0) => {
                if vx != vy {
                    self.skip()
                }
            }
            (0xA, _, _, _) => self.i = nnn,
            (0xB, _, _, _) => {
                let jump_register = if self.quirks.jump0 { vx } else { self.v[0] } as u16;
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
                    if self.display.hires || self.quirks.loresbigsprite {
                        width = 16;
                        height = 16;
                    } else if self.quirks.lorestallsprite {
                        height = 16;
                    } else {
                        return Ok(());
                    }
                }

                let mut sprite = Vec::<Vec<u8>>::new();
                for _y in 0..height {
                    let mut row = Vec::<u8>::new();
                    let mut byte = self.memory[address as usize];
                    for x in 0..width {
                        if x == 8 {
                            address = address.wrapping_add(1);
                            byte = self.memory[address as usize];
                        }
                        //byte <<= (x % 8);
                        row.push((byte << (x % 8)) >> 7);
                    }
                    sprite.push(row);
                    address = address.wrapping_add(1);
                }

                self.v[0xF] = self.display.draw(sprite, vx, vy)
            }
            (0xE, _x, 0x9, 0xE) => {
                if self.keyboard[vx as usize] {
                    self.skip()
                }
            }
            (0xE, _x, 0xA, 0x1) => {
                if !self.keyboard[vx as usize] {
                    self.skip()
                }
            }
            (0xF, 0x0, 0x0, 0x0) => self.i = self.fetch(),
            (0xF, 0x0, 0x0, 0x2) => (), // TODO
            (0xF, _, 0x0, 0x7) => self.v[x] = self.delay,
            (0xF, _x, 0x0, 0xA) => {
                // TODO
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
            (0xF, _, 0x0, 0x1) => self.display.plane(n),
            (0xF, _n, 0x3, 0xA) => (), // TODO
            (0xF, _, 0x1, 0x5) => self.delay = vx,
            (0xF, _, 0x1, 0x8) => self.sound = vx,
            (0xF, _, 0x1, 0xE) => self.i = self.i.wrapping_add(vx as u16),
            (0xF, _x, 0x2, 0x9) => {
                self.i = 0x50 + (vx * 5) as u16;
            }
            (0xF, _x, 0x3, 0x0) => {
                self.i = 0xA0 + (vx * 10) as u16;
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
                if !self.quirks.loadstore {
                    self.i = i
                }
            }
            (0xF, _, 0x6, 0x5) => {
                let mut i = self.i;
                for n in 0..=x {
                    self.v[n] = self.memory[i as usize];
                    i = i.wrapping_add(1);
                }
                if !self.quirks.loadstore {
                    self.i = i
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

    pub fn run(&mut self, tickrate: u16) -> Result<(), String> {
        //self.display.dirty = false;
        if !self.quirks.delaywrap && self.delay > 0 {
            self.delay = self.delay.wrapping_sub(1)
        }
        if self.sound > 0 {
            self.sound -= 1
        }
        for _ in 0..tickrate {
            //self.execute(self.decode(self.fetch()));
            //let addr = self.pc;
            let opcode = self.fetch();
            //println!("{:02x}: {:04x}", addr, opcode);
            self.decode(opcode)?;
            if self.quirks.vblank && opcode >= 0xD000 && opcode <= 0xDFFF {
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
        Self::new()
    }
}
