pub struct Display {
    pub display: [[u8; 128]; 64],
    pub dirty: bool,
    pub clear: bool,
    pub hires: bool,
    pub width: u16,
    pub height: u16,
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

    pub fn clear(&mut self) {
        for y in self.display.iter_mut() {
            for pixel in y.iter_mut() {
                *pixel = 0;
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
                collision = self.display[y as usize + row][x as usize + col] & sprite[row][col];
                self.display[y as usize + row][x as usize + col] ^= sprite[row][col];
            }
        }
        self.clear = false;
        self.dirty = true;
        collision
    }

    pub fn scroll_up(&mut self, _pixels: u8) {
        if !self.clear {
            self.dirty = true;
        }
    }

    pub fn scroll_down(&mut self, _pixels: u8) {
        if !self.clear {
            self.dirty = true;
        }
    }

    pub fn scroll_left(&mut self, _pixels: u8) {
        if !self.clear {
            self.dirty = true;
        }
    }

    pub fn scroll_right(&mut self, _pixels: u8) {
        if !self.clear {
            self.dirty = true;
        }
    }

    pub fn plane(&mut self, _plane: u8) {}

    pub fn hires(&mut self, clear: bool) {
        self.hires = true;
        self.width = 128;
        self.height = 64;
        if clear && !self.clear {
            self.clear();
            self.clear = true
        }
    }

    pub fn lores(&mut self, clear: bool) {
        self.hires = false;
        self.width = 64;
        self.height = 32;
        if clear && !self.clear {
            self.clear();
            self.clear = true
        }
    }
}

impl Default for Display {
    fn default() -> Self {
        Self::new()
    }
}
