#[cfg(test)]
mod tests {
    use crate::state::{Cube, Color};
    use crate::reduction::centers::get_center_commutator;
    use crate::reduction::edges::get_slice_flip_unslice;

    fn capture_state(cube: &Cube) -> Vec<Color> {
        let mut state = Vec::new();
        for face_idx in 0..6 {
            state.extend_from_slice(&cube.faces[face_idx]);
        }
        state
    }

    #[test]
    fn test_center_macro_isolation() {
        for size in [4, 5, 6] {
            let mut cube = Cube::new(size);
            // Target: U face (1,1), Source: F face (1,1) -> same column c=1
            let macro_str = get_center_commutator(Color::U, 1, 1, Color::F, 1, 1);
            println!("Testing Center Macro on {}x{}x{}: {}", size, size, size, macro_str);

            let before = capture_state(&cube);
            cube.apply_moves(&macro_str);
            let after = capture_state(&cube);

            // In my current move logic, D move rotates bottom row of F, R, B, L.
            // 2D rotates 2nd row from bottom.
            // (c1+1)R' rotates (c1+1)-th column.

            // Check if piece moved.
            // This specific commutator might move F(1,1) to U(1,1) only if aligned correctly.
            let mut changed_count = 0;
            for i in 0..before.len() {
                if before[i] != after[i] {
                    changed_count += 1;
                }
            }
            println!("Pieces changed: {}", changed_count);
            assert!(changed_count > 0);
            assert!(changed_count <= 20, "Macro is not isolated enough. Changed: {}", changed_count);
        }
    }

    #[test]
    fn test_edge_macro_isolation() {
        for size in [4, 5, 6] {
            let mut cube = Cube::new(size);
            let macro_str = get_slice_flip_unslice(1);
            println!("Testing Edge Macro on {}x{}x{}: {}", size, size, size, macro_str);

            let before = capture_state(&cube);
            cube.apply_moves(&macro_str);
            let after = capture_state(&cube);

            let mut changed_count = 0;
            for i in 0..before.len() {
                if before[i] != after[i] {
                    changed_count += 1;
                }
            }
            println!("Pieces changed: {}", changed_count);
            assert!(changed_count > 0);
        }
    }

    #[test]
    fn test_multi_size_coordinate_mapping() {
        for size in [4, 5, 6] {
            let mut cube = Cube::new(size);
            // Rotate a single slice
            cube.apply_move("2R");
            // In my implementation, layer 1 of face R is the 2nd column from right.
            // But face R is +X, so it's index (size-1) - 1.
            // Wait, let's check state.rs

            // F face is +Z. Rotation around X axis (R face) affects F face columns.
            // If we rotate a slice, the F face should change.
            let mut changed = false;
            for r in 0..size {
                for c in 0..size {
                    if cube.get_color(Color::F, r, c) != Color::F {
                        changed = true;
                        println!("F({},{}) changed to {:?}", r, c, cube.get_color(Color::F, r, c));
                    }
                }
            }
            assert!(changed, "Size {} failed: no pieces changed on F face after 2R", size);
        }
    }
}
