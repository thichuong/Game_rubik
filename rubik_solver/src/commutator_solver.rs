#![allow(clippy::needless_range_loop)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::used_underscore_binding)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::many_single_char_names)]
#![allow(clippy::if_not_else)]
#![allow(clippy::similar_names)]
#![allow(clippy::redundant_else)]
#![allow(clippy::branches_sharing_code)]

use crate::core::{Direction, Face, RotationAxis, RotationMove};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CubeState {
    pub size: usize,
    pub faces: std::collections::HashMap<Face, Vec<Vec<Face>>>,
}

impl CubeState {
    pub fn new(size: usize, state_str: &str) -> Option<Self> {
        let expected_len = size * size * 6;
        if state_str.len() != expected_len {
            return None;
        }

        let mut faces = std::collections::HashMap::new();
        let face_order = [Face::Up, Face::Right, Face::Front, Face::Down, Face::Left, Face::Back];

        let chars: Vec<char> = state_str.chars().collect();
        let mut idx = 0;

        for &face in &face_order {
            let mut grid = vec![vec![Face::Up; size]; size];
            for row in 0..size {
                for col in 0..size {
                    grid[row][col] = Self::char_to_face(chars[idx])?;
                    idx += 1;
                }
            }
            faces.insert(face, grid);
        }

        Some(Self { size, faces })
    }

    fn char_to_face(c: char) -> Option<Face> {
        match c {
            'U' | 'W' => Some(Face::Up),
            'R' => Some(Face::Right),
            'F' | 'G' => Some(Face::Front),
            'D' | 'Y' => Some(Face::Down),
            'L' | 'O' => Some(Face::Left),
            'B' => Some(Face::Back),
            _ => None,
        }
    }

    pub fn to_state_str(&self) -> String {
        let face_order = [Face::Up, Face::Right, Face::Front, Face::Down, Face::Left, Face::Back];
        let mut s = String::new();
        for &face in &face_order {
            if let Some(grid) = self.faces.get(&face) {
                for row in grid {
                    for &piece in row {
                        s.push(Self::face_to_char(piece));
                    }
                }
            }
        }
        s
    }

    fn face_to_char(f: Face) -> char {
        match f {
            Face::Up => 'U',
            Face::Right => 'R',
            Face::Front => 'F',
            Face::Down => 'D',
            Face::Left => 'L',
            Face::Back => 'B',
        }
    }

    pub fn is_solved(&self) -> bool {
        for (&expected_face, grid) in &self.faces {
            for row in grid {
                for &piece in row {
                    if piece != expected_face {
                        return false;
                    }
                }
            }
        }
        true
    }

    pub fn apply_move(&mut self, m: RotationMove) {
        let size = self.size;
        let index = m.index as usize;
        let rev = m.direction == Direction::CounterClockwise;

        let mut u = self.faces.get(&Face::Up).unwrap().clone();
        let mut d = self.faces.get(&Face::Down).unwrap().clone();
        let mut l = self.faces.get(&Face::Left).unwrap().clone();
        let mut r = self.faces.get(&Face::Right).unwrap().clone();
        let mut f = self.faces.get(&Face::Front).unwrap().clone();
        let mut b = self.faces.get(&Face::Back).unwrap().clone();

        match m.axis {
            RotationAxis::X => {
                for i in 0..size {
                    let temp = u[i][index];
                    if !rev {
                        u[i][index] = f[i][index];
                        f[i][index] = d[i][index];
                        d[i][index] = b[size - 1 - i][size - 1 - index];
                        b[size - 1 - i][size - 1 - index] = temp;
                    } else {
                        u[i][index] = b[size - 1 - i][size - 1 - index];
                        b[size - 1 - i][size - 1 - index] = d[i][index];
                        d[i][index] = f[i][index];
                        f[i][index] = temp;
                    }
                }
                if index == size - 1 {
                    rotate_face(&mut r, !rev);
                } else if index == 0 {
                    rotate_face(&mut l, rev);
                }
            }
            RotationAxis::Y => {
                for i in 0..size {
                    let temp = f[index][i];
                    if !rev {
                        f[index][i] = r[index][i];
                        r[index][i] = b[index][i];
                        b[index][i] = l[index][i];
                        l[index][i] = temp;
                    } else {
                        f[index][i] = l[index][i];
                        l[index][i] = b[index][i];
                        b[index][i] = r[index][i];
                        r[index][i] = temp;
                    }
                }
                if index == size - 1 {
                    rotate_face(&mut u, !rev);
                } else if index == 0 {
                    rotate_face(&mut d, rev);
                }
            }
            RotationAxis::Z => {
                for i in 0..size {
                    let temp = u[size - 1 - index][i];
                    if !rev {
                        u[size - 1 - index][i] = l[size - 1 - i][size - 1 - index];
                        l[size - 1 - i][size - 1 - index] = d[index][size - 1 - i];
                        d[index][size - 1 - i] = r[i][index];
                        r[i][index] = temp;
                    } else {
                        u[size - 1 - index][i] = r[i][index];
                        r[i][index] = d[index][size - 1 - i];
                        d[index][size - 1 - i] = l[size - 1 - i][size - 1 - index];
                        l[size - 1 - i][size - 1 - index] = temp;
                    }
                }
                if index == size - 1 {
                    rotate_face(&mut f, !rev);
                } else if index == 0 {
                    rotate_face(&mut b, rev);
                }
            }
        }

        self.faces.insert(Face::Up, u);
        self.faces.insert(Face::Down, d);
        self.faces.insert(Face::Left, l);
        self.faces.insert(Face::Right, r);
        self.faces.insert(Face::Front, f);
        self.faces.insert(Face::Back, b);
    }
}

fn rotate_face(face: &mut Vec<Vec<Face>>, clockwise: bool) {
    let size = face.len();
    let mut new_face = face.clone();
    for i in 0..size {
        for j in 0..size {
            if clockwise {
                new_face[j][size - 1 - i] = face[i][j];
            } else {
                new_face[size - 1 - j][i] = face[i][j];
            }
        }
    }
    *face = new_face;
}

pub fn generate_commutator(
    _size: usize,
    _piece_a: (Face, usize, usize),
    _piece_b: (Face, usize, usize),
    _piece_c: (Face, usize, usize),
) -> Vec<RotationMove> {
    vec![]
}

pub fn solve(size: usize, state_str: &str) -> Option<Vec<String>> {
    let mut state = CubeState::new(size, state_str)?;
    let mut solution = Vec::new();

    let mut limit = 0;
    while !state.is_solved() && limit < 100 {
        let mut target_found = false;
        let mut unsolved_a = (Face::Up, 0, 0);
        let mut unsolved_b = (Face::Up, 0, 0);
        let mut unsolved_c = (Face::Up, 0, 0);

        for (&face, grid) in &state.faces {
            for row in 0..size {
                for col in 0..size {
                    if grid[row][col] != face {
                        unsolved_a = (face, row, col);
                        unsolved_b = (grid[row][col], row, col);
                        unsolved_c = (Face::Right, 0, 0);
                        target_found = true;
                        break;
                    }
                }
                if target_found { break; }
            }
            if target_found { break; }
        }

        if target_found {
            let comm_moves = generate_commutator(size, unsolved_a, unsolved_b, unsolved_c);

            for &m in &comm_moves {
                state.apply_move(m);
            }

            for m in comm_moves {
                solution.push(crate::helpers::physical_move_to_logical_string_any(
                    m,
                    size as i32,
                    crate::core::FaceMapping::default(),
                ));
            }
        }

        if solution.is_empty() {
            if let Some(daemon_moves) = crate::solver::solve_nxn_state_only(&state.to_state_str()) {
                solution.extend(daemon_moves);

                let physical = crate::helpers::logical_string_to_physical_moves_any(
                    &solution.join(" "),
                    size as i32,
                    crate::core::FaceMapping::default(),
                );
                for m in physical {
                    state.apply_move(m);
                }
                break;
            } else {
                break;
            }
        }

        limit += 1;
    }

    if !solution.is_empty() {
        let physical = crate::helpers::logical_string_to_physical_moves_any(
            &solution.join(" "),
            size as i32,
            crate::core::FaceMapping::default(),
        );
        let optimized = crate::helpers::optimize_moves(&physical);
        solution.clear();
        for m in optimized {
            solution.push(crate::helpers::physical_move_to_logical_string_any(
                m,
                size as i32,
                crate::core::FaceMapping::default(),
            ));
        }
    }

    Some(solution)
}
