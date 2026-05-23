use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    U, D, L, R, F, B,
}

impl Color {
    pub const fn to_char(self) -> char {
        match self {
            Self::U => 'U',
            Self::D => 'D',
            Self::L => 'L',
            Self::R => 'R',
            Self::F => 'F',
            Self::B => 'B',
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Cube {
    pub size: usize,
    pub faces: [Vec<Color>; 6],
}

impl Cube {
    pub fn new(size: usize) -> Self {
        let mut faces = [
            Vec::with_capacity(size * size),
            Vec::with_capacity(size * size),
            Vec::with_capacity(size * size),
            Vec::with_capacity(size * size),
            Vec::with_capacity(size * size),
            Vec::with_capacity(size * size),
        ];

        let colors = [Color::U, Color::D, Color::L, Color::R, Color::F, Color::B];
        for (i, color) in colors.iter().enumerate() {
            faces[i].resize(size * size, *color);
        }

        Self { size, faces }
    }

    pub const fn get_face_index(color: Color) -> usize {
        match color {
            Color::U => 0,
            Color::D => 1,
            Color::L => 2,
            Color::R => 3,
            Color::F => 4,
            Color::B => 5,
        }
    }

    pub fn get_color(&self, face: Color, row: usize, col: usize) -> Color {
        self.faces[Self::get_face_index(face)][row * self.size + col]
    }

    pub fn set_color(&mut self, face: Color, row: usize, col: usize, color: Color) {
        let size = self.size;
        self.faces[Self::get_face_index(face)][row * size + col] = color;
    }

    pub fn rotate_face_cw(&mut self, face: Color) {
        let size = self.size;
        let face_idx = Self::get_face_index(face);
        let mut new_face = self.faces[face_idx].clone();
        for r in 0..size {
            for c in 0..size {
                new_face[c * size + (size - 1 - r)] = self.faces[face_idx][r * size + c];
            }
        }
        self.faces[face_idx] = new_face;

        match face {
            Color::U => {
                let mut temp = vec![Color::U; size];
                for (i, item) in temp.iter_mut().enumerate().take(size) { *item = self.get_color(Color::F, 0, i); }
                for i in 0..size { self.set_color(Color::F, 0, i, self.get_color(Color::R, 0, i)); }
                for i in 0..size { self.set_color(Color::R, 0, i, self.get_color(Color::B, 0, i)); }
                for i in 0..size { self.set_color(Color::B, 0, i, self.get_color(Color::L, 0, i)); }
                for (i, &item) in temp.iter().enumerate().take(size) { self.set_color(Color::L, 0, i, item); }
            }
            Color::D => {
                let mut temp = vec![Color::D; size];
                for (i, item) in temp.iter_mut().enumerate().take(size) { *item = self.get_color(Color::F, size - 1, i); }
                for i in 0..size { self.set_color(Color::F, size - 1, i, self.get_color(Color::L, size - 1, i)); }
                for i in 0..size { self.set_color(Color::L, size - 1, i, self.get_color(Color::B, size - 1, i)); }
                for i in 0..size { self.set_color(Color::B, size - 1, i, self.get_color(Color::R, size - 1, i)); }
                for (i, &item) in temp.iter().enumerate().take(size) { self.set_color(Color::R, size - 1, i, item); }
            }
            Color::L => {
                let mut temp = vec![Color::L; size];
                for (i, item) in temp.iter_mut().enumerate().take(size) { *item = self.get_color(Color::U, i, 0); }
                for i in 0..size { self.set_color(Color::U, i, 0, self.get_color(Color::B, size - 1 - i, size - 1)); }
                for i in 0..size { self.set_color(Color::B, size - 1 - i, size - 1, self.get_color(Color::D, i, 0)); }
                for i in 0..size { self.set_color(Color::D, i, 0, self.get_color(Color::F, i, 0)); }
                for (i, &item) in temp.iter().enumerate().take(size) { self.set_color(Color::F, i, 0, item); }
            }
            Color::R => {
                let mut temp = vec![Color::R; size];
                for (i, item) in temp.iter_mut().enumerate().take(size) { *item = self.get_color(Color::U, i, size - 1); }
                for i in 0..size { self.set_color(Color::U, i, size - 1, self.get_color(Color::F, i, size - 1)); }
                for i in 0..size { self.set_color(Color::F, i, size - 1, self.get_color(Color::D, i, size - 1)); }
                for i in 0..size { self.set_color(Color::D, i, size - 1, self.get_color(Color::B, size - 1 - i, 0)); }
                for (i, &item) in temp.iter().enumerate().take(size) { self.set_color(Color::B, size - 1 - i, 0, item); }
            }
            Color::F => {
                let mut temp = vec![Color::F; size];
                for (i, item) in temp.iter_mut().enumerate().take(size) { *item = self.get_color(Color::U, size - 1, i); }
                for i in 0..size { self.set_color(Color::U, size - 1, i, self.get_color(Color::L, size - 1 - i, size - 1)); }
                for i in 0..size { self.set_color(Color::L, size - 1 - i, size - 1, self.get_color(Color::D, 0, size - 1 - i)); }
                for i in 0..size { self.set_color(Color::D, 0, size - 1 - i, self.get_color(Color::R, i, 0)); }
                for (i, &item) in temp.iter().enumerate().take(size) { self.set_color(Color::R, i, 0, item); }
            }
            Color::B => {
                let mut temp = vec![Color::B; size];
                for (i, item) in temp.iter_mut().enumerate().take(size) { *item = self.get_color(Color::U, 0, i); }
                for i in 0..size { self.set_color(Color::U, 0, i, self.get_color(Color::R, i, size - 1)); }
                for i in 0..size { self.set_color(Color::R, i, size - 1, self.get_color(Color::D, size - 1, size - 1 - i)); }
                for i in 0..size { self.set_color(Color::D, size - 1, size - 1 - i, self.get_color(Color::L, size - 1 - i, 0)); }
                for (i, &item) in temp.iter().enumerate().take(size) { self.set_color(Color::L, size - 1 - i, 0, item); }
            }
        }
    }

    pub fn rotate_slice_cw(&mut self, face: Color, layer: usize) {
        if layer == 0 {
            self.rotate_face_cw(face);
            return;
        }
        let size = self.size;
        match face {
            Color::U => {
                let mut temp = vec![Color::U; size];
                for (i, item) in temp.iter_mut().enumerate().take(size) { *item = self.get_color(Color::F, layer, i); }
                for i in 0..size { self.set_color(Color::F, layer, i, self.get_color(Color::R, layer, i)); }
                for i in 0..size { self.set_color(Color::R, layer, i, self.get_color(Color::B, layer, i)); }
                for i in 0..size { self.set_color(Color::B, layer, i, self.get_color(Color::L, layer, i)); }
                for (i, &item) in temp.iter().enumerate().take(size) { self.set_color(Color::L, layer, i, item); }
            }
            Color::D => {
                let l = size - 1 - layer;
                let mut temp = vec![Color::D; size];
                for (i, item) in temp.iter_mut().enumerate().take(size) { *item = self.get_color(Color::F, l, i); }
                for i in 0..size { self.set_color(Color::F, l, i, self.get_color(Color::L, l, i)); }
                for i in 0..size { self.set_color(Color::L, l, i, self.get_color(Color::B, l, i)); }
                for i in 0..size { self.set_color(Color::B, l, i, self.get_color(Color::R, l, i)); }
                for (i, &item) in temp.iter().enumerate().take(size) { self.set_color(Color::R, l, i, item); }
            }
            Color::L => {
                let mut temp = vec![Color::L; size];
                for (i, item) in temp.iter_mut().enumerate().take(size) { *item = self.get_color(Color::U, i, layer); }
                for i in 0..size { self.set_color(Color::U, i, layer, self.get_color(Color::B, size - 1 - i, size - 1 - layer)); }
                for i in 0..size { self.set_color(Color::B, size - 1 - i, size - 1 - layer, self.get_color(Color::D, i, layer)); }
                for i in 0..size { self.set_color(Color::D, i, layer, self.get_color(Color::F, i, layer)); }
                for (i, &item) in temp.iter().enumerate().take(size) { self.set_color(Color::F, i, layer, item); }
            }
            Color::R => {
                let l = size - 1 - layer;
                let mut temp = vec![Color::R; size];
                for (i, item) in temp.iter_mut().enumerate().take(size) { *item = self.get_color(Color::U, i, l); }
                for i in 0..size { self.set_color(Color::U, i, l, self.get_color(Color::F, i, l)); }
                for i in 0..size { self.set_color(Color::F, i, l, self.get_color(Color::D, i, l)); }
                for i in 0..size { self.set_color(Color::D, i, l, self.get_color(Color::B, size - 1 - i, size - 1 - l)); }
                for (i, &item) in temp.iter().enumerate().take(size) { self.set_color(Color::B, size - 1 - i, size - 1 - l, item); }
            }
            Color::F => {
                let l = size - 1 - layer;
                let mut temp = vec![Color::F; size];
                for (i, item) in temp.iter_mut().enumerate().take(size) { *item = self.get_color(Color::U, l, i); }
                for i in 0..size { self.set_color(Color::U, l, i, self.get_color(Color::L, size - 1 - i, l)); }
                for i in 0..size { self.set_color(Color::L, size - 1 - i, l, self.get_color(Color::D, size - 1 - l, size - 1 - i)); }
                for i in 0..size { self.set_color(Color::D, size - 1 - l, size - 1 - i, self.get_color(Color::R, i, size - 1 - l)); }
                for (i, &item) in temp.iter().enumerate().take(size) { self.set_color(Color::R, i, size - 1 - l, item); }
            }
            Color::B => {
                let mut temp = vec![Color::B; size];
                for (i, item) in temp.iter_mut().enumerate().take(size) { *item = self.get_color(Color::U, layer, i); }
                for i in 0..size { self.set_color(Color::U, layer, i, self.get_color(Color::R, i, size - 1 - layer)); }
                for i in 0..size { self.set_color(Color::R, i, size - 1 - layer, self.get_color(Color::D, size - 1 - layer, size - 1 - i)); }
                for i in 0..size { self.set_color(Color::D, size - 1 - layer, size - 1 - i, self.get_color(Color::L, size - 1 - i, layer)); }
                for (i, &item) in temp.iter().enumerate().take(size) { self.set_color(Color::L, size - 1 - i, layer, item); }
            }
        }
    }

    /// Apply a move to the cube.
    ///
    /// # Panics
    ///
    /// Panics if the move string contains an invalid layer index.
    #[allow(clippy::missing_panics_doc)]
    pub fn apply_move(&mut self, m: &str) {
        if m.is_empty() { return; }
        let mut chars = m.chars().peekable();
        let mut layer = 0;
        if let Some(&c) = chars.peek() {
            if c.is_ascii_digit() {
                layer = c.to_digit(10).expect("Invalid layer digit") as usize - 1;
                chars.next();
            }
        }
        let Some(face_char) = chars.next() else { return };
        let face = match face_char {
            'U' => Color::U, 'D' => Color::D, 'L' => Color::L,
            'R' => Color::R, 'F' => Color::F, 'B' => Color::B,
            _ => return,
        };
        let mut wide = false;
        if chars.peek() == Some(&'w') {
            wide = true;
            chars.next();
            if layer == 0 { layer = 1; }
        }
        let mut count = 1;
        let mut inverse = false;
        if chars.peek() == Some(&'\'') {
            inverse = true;
            chars.next();
        } else if chars.peek() == Some(&'2') {
            count = 2;
            chars.next();
        }

        for _ in 0..count {
            if wide {
                for l in 0..=layer {
                    if inverse { for _ in 0..3 { self.rotate_slice_cw(face, l); } }
                    else { self.rotate_slice_cw(face, l); }
                }
            } else {
                let l = layer;
                if inverse { for _ in 0..3 { self.rotate_slice_cw(face, l); } }
                else { self.rotate_slice_cw(face, l); }
            }
        }
    }
}

impl fmt::Debug for Cube {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Cube size: {}", self.size)?;
        Ok(())
    }
}
