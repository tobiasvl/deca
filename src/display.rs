pub struct Display {
    pub display: [[u8; 128]; 64],
    pub dirty: bool,
    pub clear: bool,
    pub hires: bool,
    pub width: u8,
    pub height: u8,
    pub active_plane: u8,
}

impl Display {
    pub fn new() -> Display {
        Display {
            display: [[0; 128]; 64],
            dirty: false,
            clear: true,
            hires: false,
            width: 64,
            height: 32,
            active_plane: 1,
        }
    }

    pub fn clear(&mut self, all_planes: bool) {
        for y in self.display.iter_mut() {
            for pixel in y.iter_mut() {
                if all_planes {
                    *pixel = 0
                } else {
                    *pixel &= !self.active_plane;
                }
            }
        }

        self.dirty = true;
        self.clear = true;
    }

    pub fn draw(&mut self, sprite: Vec<Vec<u8>>, x: u8, y: u8) -> u8 {
        let x = x % self.width as u8;
        let y = y % self.height as u8;
        let mut collision = 0;
        for row in 0..sprite.len() {
            if row + y as usize >= self.height as usize {
                break;
            }
            for col in 0..sprite[row].len() {
                if col + x as usize >= self.width as usize {
                    break;
                }
                if sprite[row][col] == 1 {
                    let display_pixel = &mut self.display[y as usize + row][x as usize + col];
                    if *display_pixel & self.active_plane == 0 {
                        *display_pixel |= self.active_plane;
                    } else {
                        *display_pixel &= !self.active_plane;
                        collision = 1;
                    };
                }
            }
        }
        self.clear = false;
        self.dirty = true;
        collision
    }

    pub fn scroll_up(&mut self, pixels: u8) {
        if !self.clear && pixels > 0 {
            for y in pixels..self.height {
                for x in 0..self.width {
                    self.display[(y - pixels) as usize][x as usize] |=
                        self.display[y as usize][x as usize] & self.active_plane;
                    self.display[y as usize][x as usize] &= !self.active_plane;
                }
            }
            for y in (self.height - pixels)..self.height {
                for x in 0..=self.width {
                    self.display[y as usize][x as usize] &= !self.active_plane;
                }
            }

            self.dirty = true;
        }
    }

    pub fn scroll_down(&mut self, pixels: u8) {
        if !self.clear && pixels > 0 {
            for y in (0..self.height - pixels).rev() {
                for x in 0..self.width {
                    self.display[(y + pixels) as usize][x as usize] |=
                        self.display[y as usize][x as usize] & self.active_plane;
                    self.display[y as usize][x as usize] &= !self.active_plane;
                }
            }
            for y in 0..pixels {
                for x in 0..self.width {
                    self.display[y as usize][x as usize] &= !self.active_plane;
                }
            }

            self.dirty = true;
        }
    }

    pub fn scroll_left(&mut self, pixels: u8) {
        if !self.clear && pixels > 0 {
            for y in 0..self.height {
                for x in pixels..self.width {
                    self.display[y as usize][(x - pixels) as usize] |=
                        self.display[y as usize][x as usize] & self.active_plane;
                    self.display[y as usize][x as usize] &= !self.active_plane;
                }
            }
            for y in 0..self.height {
                for x in (self.width - pixels)..self.width {
                    self.display[y as usize][x as usize] &= !self.active_plane;
                }
            }

            self.dirty = true;
        }
    }

    pub fn scroll_right(&mut self, pixels: u8) {
        if !self.clear && pixels > 0 {
            for y in 0..self.height {
                for x in (0..self.width - pixels).rev() {
                    self.display[y as usize][(x + pixels) as usize] |=
                        self.display[y as usize][x as usize] & self.active_plane;
                    self.display[y as usize][x as usize] &= !self.active_plane;
                }
            }
            for y in 0..self.height {
                for x in 0..pixels {
                    self.display[y as usize][x as usize] &= !self.active_plane;
                }
            }

            self.dirty = true;
        }
    }

    pub fn plane(&mut self, plane: u8) {
        self.active_plane = plane;
    }

    pub fn hires(&mut self, clear: bool) {
        self.hires = true;
        self.width = 128;
        self.height = 64;
        if clear && !self.clear {
            self.clear(true);
            self.clear = true
        }
    }

    pub fn lores(&mut self, clear: bool) {
        self.hires = false;
        self.width = 64;
        self.height = 32;
        if clear && !self.clear {
            self.clear(true);
            self.clear = true
        }
    }
}

impl Default for Display {
    fn default() -> Self {
        Self::new()
    }
}
