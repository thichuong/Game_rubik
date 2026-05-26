// Represents the components of the Rubik's cube data structures.
// All comments in source files must be in English.

pub mod moves;

use std::fmt;

/// Custom error type for Rubik's cube operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CubeError {
    InvalidInputLength(usize, usize), // (expected, actual)
    InvalidCharacter(char),
    InvalidSize(usize),
    InvalidMove(String),
    IndexOutOfBounds(usize, usize), // (index, limit)
}

impl fmt::Display for CubeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CubeError::InvalidInputLength(expected, actual) => {
                write!(
                    f,
                    "Invalid input length: expected {}, got {}",
                    expected, actual
                )
            }
            CubeError::InvalidCharacter(c) => write!(f, "Invalid character in input: {}", c),
            CubeError::InvalidSize(size) => write!(f, "Invalid cube size: {}", size),
            CubeError::InvalidMove(m) => write!(f, "Invalid move string: {}", m),
            CubeError::IndexOutOfBounds(idx, limit) => {
                write!(f, "Index out of bounds: index {} with limit {}", idx, limit)
            }
        }
    }
}

impl std::error::Error for CubeError {}

/// Represents the 6 faces of the Rubik's cube.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Face {
    U = 0,
    D = 1,
    F = 2,
    B = 3,
    L = 4,
    R = 5,
}

impl Face {
    /// Returns the character representation of the face.
    pub fn to_char(self) -> char {
        match self {
            Face::U => 'U',
            Face::D => 'D',
            Face::F => 'F',
            Face::B => 'B',
            Face::L => 'L',
            Face::R => 'R',
        }
    }

    /// Tries to parse a character into a Face enum.
    pub fn from_char(c: char) -> Result<Self, CubeError> {
        match c {
            'U' => Ok(Face::U),
            'D' => Ok(Face::D),
            'F' => Ok(Face::F),
            'B' => Ok(Face::B),
            'L' => Ok(Face::L),
            'R' => Ok(Face::R),
            _ => Err(CubeError::InvalidCharacter(c)),
        }
    }
}

/// Represents an nxn Rubik's Cube.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cube {
    size: usize,
    // Flat representation of the cube state. Length is 6 * size^2.
    // Order: U, D, F, B, L, R.
    // Each face is a size x size grid stored row-by-row (top-to-bottom, left-to-right).
    grid: Vec<Face>,
}

impl Cube {
    /// Creates a new Cube of size n in its solved state.
    pub fn new(size: usize) -> Result<Self, CubeError> {
        if size < 2 {
            return Err(CubeError::InvalidSize(size));
        }
        let face_size = size * size;
        let mut grid = Vec::with_capacity(6 * face_size);

        let faces = [Face::U, Face::D, Face::F, Face::B, Face::L, Face::R];
        for &face in &faces {
            for _ in 0..face_size {
                grid.push(face);
            }
        }

        Ok(Self { size, grid })
    }

    /// Creates a Cube from a face-represented string (length must be 6 * size^2).
    /// Characters must be in {'U', 'D', 'F', 'B', 'L', 'R'}.
    pub fn from_string(size: usize, state: &str) -> Result<Self, CubeError> {
        if size < 2 {
            return Err(CubeError::InvalidSize(size));
        }
        let expected_len = 6 * size * size;
        if state.len() != expected_len {
            return Err(CubeError::InvalidInputLength(expected_len, state.len()));
        }

        let mut grid = Vec::with_capacity(expected_len);
        for c in state.chars() {
            grid.push(Face::from_char(c)?);
        }

        Ok(Self { size, grid })
    }

    /// Returns the cube size.
    pub fn size(&self) -> usize {
        self.size
    }

    /// Returns the facelet value at a specific face, row, and column.
    pub fn get(&self, face: Face, row: usize, col: usize) -> Result<Face, CubeError> {
        if row >= self.size || col >= self.size {
            return Err(CubeError::IndexOutOfBounds(row, self.size));
        }
        let idx = self.flat_index(face, row, col);
        self.grid
            .get(idx)
            .copied()
            .ok_or(CubeError::IndexOutOfBounds(idx, self.grid.len()))
    }

    /// Sets the facelet value at a specific face, row, and column.
    pub fn set(&mut self, face: Face, row: usize, col: usize, val: Face) -> Result<(), CubeError> {
        if row >= self.size || col >= self.size {
            return Err(CubeError::IndexOutOfBounds(row, self.size));
        }
        let idx = self.flat_index(face, row, col);
        if let Some(elem) = self.grid.get_mut(idx) {
            *elem = val;
            Ok(())
        } else {
            Err(CubeError::IndexOutOfBounds(idx, self.grid.len()))
        }
    }

    /// Converts the current cube state back to a string representation.
    pub fn to_string_state(&self) -> String {
        let mut s = String::with_capacity(self.grid.len());
        for &face in &self.grid {
            s.push(face.to_char());
        }
        s
    }

    /// Prints a beautiful 2D net representation of the cube state to stdout.
    /// Format:
    ///        [U]
    ///  [L]   [F]   [R]   [B]
    ///        [D]
    pub fn print_net(&self) {
        let size = self.size;
        let indent = " ".repeat(size + 1);

        // 1. Print U face
        for r in 0..size {
            print!("{}", indent);
            for c in 0..size {
                if let Ok(f) = self.get(Face::U, r, c) {
                    print!("{}", f.to_char());
                }
            }
            println!();
        }
        println!();

        // 2. Print L, F, R, B faces side by side
        for r in 0..size {
            // L face
            for c in 0..size {
                if let Ok(f) = self.get(Face::L, r, c) {
                    print!("{}", f.to_char());
                }
            }
            print!(" ");

            // F face
            for c in 0..size {
                if let Ok(f) = self.get(Face::F, r, c) {
                    print!("{}", f.to_char());
                }
            }
            print!(" ");

            // R face
            for c in 0..size {
                if let Ok(f) = self.get(Face::R, r, c) {
                    print!("{}", f.to_char());
                }
            }
            print!(" ");

            // B face
            for c in 0..size {
                if let Ok(f) = self.get(Face::B, r, c) {
                    print!("{}", f.to_char());
                }
            }
            println!();
        }
        println!();

        // 3. Print D face
        for r in 0..size {
            print!("{}", indent);
            for c in 0..size {
                if let Ok(f) = self.get(Face::D, r, c) {
                    print!("{}", f.to_char());
                }
            }
            println!();
        }
    }

    /// Internal helper to calculate flat index in grid vector.
    #[inline]
    fn flat_index(&self, face: Face, row: usize, col: usize) -> usize {
        let face_offset = (face as usize) * self.size * self.size;
        face_offset + row * self.size + col
    }
}
