/// A struct representing a CHIP-8 display.
pub struct Display {
    /// The display buffer.
    pub display: [[u8; 128]; 64],
    /// A dirty flag denoting whether the display buffer has changed or not. This can be used by a frontend
    /// to minimize drawing calls when the display is unchanged. When reading the display buffer, the
    /// frontend should unset this flag.
    pub dirty: bool,
    /// A flag denoting whether the display buffer is cleared or not. This can be used by a frontend to quickly
    /// clear the display rather than drawing the empty display buffer.
    pub clear: bool,
    /// A flag denoting whether the display is currently in high-resolution mode or not.
    pub hires: bool,
    /// The width of the current display in pixels.
    pub width: u8,
    /// The height of the current display in pixels.
    pub height: u8,
    /// The currently active bitplane, for XO-CHIP compatibility.
    pub active_plane: u8,
}

impl Display {
    /// Create a new CHIP-8 display.
    #[must_use]
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

    /// Clear the currently active display plane.
    pub fn clear(&mut self, all_planes: bool) {
        for y in self.display.iter_mut() {
            for pixel in y.iter_mut() {
                if all_planes {
                    *pixel = 0;
                } else {
                    *pixel &= !self.active_plane;
                }
            }
        }

        self.dirty = true;
        self.clear = true;
    }

    /// Draw a sprite at the given coordinates in the currently active display plane.
    // TODO: Observe clip and collision quirks.
    pub fn draw(&mut self, sprite: Vec<Vec<u8>>, x: u8, y: u8) -> u8 {
        let x = x % self.width as u8;
        let y = y % self.height as u8;
        let mut collision = 0;
        for (row, sprite_row) in sprite.into_iter().enumerate() {
            if row + y as usize >= self.height as usize {
                break;
            }
            for (col, pixel) in sprite_row.iter().enumerate() {
                if col + x as usize >= self.width as usize {
                    break;
                }
                if *pixel == 1 {
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

    /// Scroll the currently active display plane up.
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

    /// Scroll the currently active display plane down.
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

    /// Scroll the currently active display plane left.
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

    /// Scroll the currently active display plane right.
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

    /// Change the currently active plane.
    pub fn plane(&mut self, plane: u8) {
        self.active_plane = plane;
    }

    /// Switch to high-resolution mode.
    pub fn hires(&mut self, clear: bool) {
        self.hires = true;
        self.width = 128;
        self.height = 64;
        if clear && !self.clear {
            self.clear(true);
            self.clear = true;
        }
    }

    /// Switch to low-resolution mode.
    pub fn lores(&mut self, clear: bool) {
        self.hires = false;
        self.width = 64;
        self.height = 32;
        if clear && !self.clear {
            self.clear(true);
            self.clear = true;
        }
    }
}

impl Default for Display {
    fn default() -> Self {
        Self::new()
    }
}
