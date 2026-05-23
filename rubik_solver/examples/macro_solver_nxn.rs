// English: All comments in source code must be in English.
// Vietnamese: Trao đổi trong phần chat hoàn toàn bằng Tiếng Việt.

use rand::RngExt;
use rubik_solver::core::{Direction, Face, RotationAxis, RotationMove};
use rubik_solver::macro_solver::{
    count_misplaced_centers, solve_cube_macro_hybrid, L2CTable, VirtualCube,
};
use std::time::Instant;

fn main() {
    println!("=====================================================");
    println!("   🧠 NxN HYBRID SOLVER DEBUGGER (6x6x6)             ");
    println!("=====================================================");

    let size = 6;
    let table_path = "l2c_table_6x6.json";

    println!("Loading L2C Lookup Table...");
    let start_init = Instant::now();
    let l2c_table = if let Ok(table) = L2CTable::load(table_path) {
        println!("Loaded table from {}.", table_path);
        table
    } else {
        println!("Table not found. Generating (this may take a while)...");
        // Use a smaller depth for testing to avoid timeout/long waits in debug environment
        let table = L2CTable::generate(size, 2);
        let _ = table.save(table_path);
        println!("Table generated (depth 2) and saved.");
        table
    };
    println!("Initialization took {:?}", start_init.elapsed());

    let mut solve_times = Vec::new();
    let mut success_count = 0;
    let num_tests = 10; // Change to 100 for full benchmark

    for i in 1..=num_tests {
        let mut cube = VirtualCube::new(size);
        let mut rng = rand::rng();
        let axes = [RotationAxis::X, RotationAxis::Y, RotationAxis::Z];

        for _ in 0..100 {
            let axis = axes[rng.random_range(0..3)];
            let index = rng.random_range(0..size);
            let direction = if rng.random_bool(0.5) {
                Direction::Clockwise
            } else {
                Direction::CounterClockwise
            };
            cube.apply_move(RotationMove {
                axis,
                index,
                direction,
                add_to_history: true,
            });
        }

        println!("\nTest #{}: Scrambled. Misplaced: {}", i, count_misplaced_centers(&cube));
        let start_solve = Instant::now();
        let result = solve_cube_macro_hybrid(&mut cube, Some(&l2c_table));
        let elapsed = start_solve.elapsed();

        if result.is_some() && count_misplaced_centers(&cube) == 0 {
            println!("  ✅ Solved in {:?}", elapsed);
            success_count += 1;
            solve_times.push(elapsed);
        } else {
            println!("  ❌ FAILED to solve centers.");
        }
    }

    println!("\n=====================================================");
    println!("   BENCHMARK RESULTS ({} tests)", num_tests);
    println!("   Success Rate: {}%", (success_count as f32 / num_tests as f32) * 100.0);
    if !solve_times.is_empty() {
        let avg_time: std::time::Duration = solve_times.iter().sum::<std::time::Duration>() / solve_times.len() as u32;
        println!("   Average Time: {:?}", avg_time);
    }
    println!("=====================================================");
}

fn count_boundary_components(pos: bevy::prelude::IVec3, size: i32) -> i32 {
    let mut count = 0;
    if pos.x == 0 || pos.x == size - 1 {
        count += 1;
    }
    if pos.y == 0 || pos.y == size - 1 {
        count += 1;
    }
    if pos.z == 0 || pos.z == size - 1 {
        count += 1;
    }
    count
}

fn print_misplaced_centers_details(cube: &VirtualCube) {
    let size = cube.size;
    println!("\nMisplaced Center Pieces Details:");
    let mut count = 0;
    for cubie in &cube.cubies {
        if count_boundary_components(cubie.pos, size) == 1 {
            // It's a center cubie
            let mut expected_face = None;
            if cubie.pos.x == size - 1 {
                expected_face = Some(Face::Right);
            } else if cubie.pos.x == 0 {
                expected_face = Some(Face::Left);
            } else if cubie.pos.y == size - 1 {
                expected_face = Some(Face::Up);
            } else if cubie.pos.y == 0 {
                expected_face = Some(Face::Down);
            } else if cubie.pos.z == size - 1 {
                expected_face = Some(Face::Front);
            } else if cubie.pos.z == 0 {
                expected_face = Some(Face::Back);
            }

            if let Some(exp_f) = expected_face {
                // Find actual sticker color facing the outward normal of this face
                let normal = match exp_f {
                    Face::Right => bevy::prelude::Vec3::X,
                    Face::Left => bevy::prelude::Vec3::NEG_X,
                    Face::Up => bevy::prelude::Vec3::Y,
                    Face::Down => bevy::prelude::Vec3::NEG_Y,
                    Face::Front => bevy::prelude::Vec3::Z,
                    Face::Back => bevy::prelude::Vec3::NEG_Z,
                };
                let local_dir = cubie.rotation.inverse() * normal;
                if let Some(actual_f) = Face::from_normal(local_dir) {
                    if actual_f != exp_f {
                        count += 1;
                        println!(
                            "  #{}: Pos: {:?}, Expected Face Color: {:?}, Actual Sticker Color: {:?}",
                            count, cubie.pos, exp_f, actual_f
                        );
                    }
                }
            }
        }
    }
}

fn print_cube_faces_grid(cube: &VirtualCube) {
    let size = cube.size;
    let faces = [
        (Face::Up, "UP (U)"),
        (Face::Left, "LEFT (L)"),
        (Face::Front, "FRONT (F)"),
        (Face::Right, "RIGHT (R)"),
        (Face::Back, "BACK (B)"),
        (Face::Down, "DOWN (D)"),
    ];

    println!("\n=====================================================");
    println!("             VISUAL 2D CUBE REPRESENTATION           ");
    println!("=====================================================");

    let get_char = |f: Face| -> &str {
        match f {
            Face::Up => "U",
            Face::Left => "L",
            Face::Front => "F",
            Face::Right => "R",
            Face::Back => "B",
            Face::Down => "D",
        }
    };

    let find_sticker_color = |x: i32, y: i32, z: i32, face: Face| -> &str {
        if let Some(cubie) = cube
            .cubies
            .iter()
            .find(|c| c.pos == bevy::prelude::IVec3::new(x, y, z))
        {
            let normal = match face {
                Face::Right => bevy::prelude::Vec3::X,
                Face::Left => bevy::prelude::Vec3::NEG_X,
                Face::Up => bevy::prelude::Vec3::Y,
                Face::Down => bevy::prelude::Vec3::NEG_Y,
                Face::Front => bevy::prelude::Vec3::Z,
                Face::Back => bevy::prelude::Vec3::NEG_Z,
            };
            let local_dir = cubie.rotation.inverse() * normal;
            if let Some(actual_f) = Face::from_normal(local_dir) {
                return get_char(actual_f);
            }
        }
        "."
    };

    for &(face, label) in &faces {
        println!("\n--- {} ---", label);
        for row in 0..size {
            let mut row_str = String::new();
            for col in 0..size {
                let (x, y, z) = match face {
                    Face::Up => (col, size - 1, row),
                    Face::Down => (col, 0, size - 1 - row),
                    Face::Left => (0, size - 1 - row, col),
                    Face::Right => (size - 1, size - 1 - row, size - 1 - col),
                    Face::Front => (col, size - 1 - row, size - 1),
                    Face::Back => (size - 1 - col, size - 1 - row, 0),
                };
                let sticker = find_sticker_color(x, y, z, face);
                row_str.push_str(sticker);
                row_str.push(' ');
            }
            println!("  {}", row_str);
        }
    }
}
