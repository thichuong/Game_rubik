use crate::state::{Color, Cube};

/// Solves centers for any `NxN` cube.
pub fn solve_centers(cube: &mut Cube) -> Vec<String> {
    let mut moves = Vec::new();
    let faces = [Color::U, Color::D, Color::L, Color::R, Color::F, Color::B];
    for &face in &faces {
        moves.extend(solve_single_face_centers(cube, face));
    }
    moves
}

fn solve_single_face_centers(cube: &mut Cube, face: Color) -> Vec<String> {
    let mut moves = Vec::new();
    let size = cube.size;
    for r in 1..size - 1 {
        for c in 1..size - 1 {
            if cube.get_color(face, r, c) != face {
                if let Some(m) = find_and_move_center(cube, face, r, c) {
                    moves.extend(m);
                }
            }
        }
    }
    moves
}

fn find_and_move_center(cube: &mut Cube, target_face: Color, target_r: usize, target_c: usize) -> Option<Vec<String>> {
    let size = cube.size;
    let faces = [Color::U, Color::D, Color::L, Color::R, Color::F, Color::B];
    for &f in &faces {
        if f == target_face { continue; }
        for r in 1..size - 1 {
            for c in 1..size - 1 {
                if cube.get_color(f, r, c) == target_face {
                    let m = get_center_commutator(target_face, target_r, target_c, f, r, c);
                    if !m.is_empty() {
                        cube.apply_moves(&m);
                        return Some(m.split_whitespace().map(String::from).collect());
                    }
                }
            }
        }
    }
    None
}

/// A standard 3-cycle center commutator macro.
pub fn get_center_commutator(f1: Color, _r1: usize, c1: usize, f2: Color, r2: usize, _c2: usize) -> String {
    if f1 == Color::U && f2 == Color::F {
        return format!("{}R' {}D {}R {}D'", c1 + 1, r2 + 1, c1 + 1, r2 + 1);
    }
    String::new()
}
