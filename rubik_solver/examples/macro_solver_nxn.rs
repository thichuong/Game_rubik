// English: All comments in source code must be in English.
// Vietnamese: Trao đổi trong phần chat hoàn toàn bằng Tiếng Việt.

use rand::RngExt;
use rubik_solver::core::{Direction, Face, RotationAxis, RotationMove};
use rubik_solver::macro_solver::{
    count_misplaced_centers, generate_center_endgame_table, generate_cube_rotations,
    generate_symmetric_macros, get_center1_moves, get_center2_moves, get_center3_moves,
    get_center4_moves, get_misplaced_centers_signature, solve_phase_beam_search, Macro, SolverPhase,
    VirtualCube,
};
use std::collections::HashSet;
use std::time::Instant;

fn main() {
    println!("=====================================================");
    println!("   🧠 NxN CENTERS ONLY DEBUGGER (6x6x6)              ");
    println!("=====================================================");

    let size = 6;
    let mut cube = VirtualCube::new(size);

    // Deep scramble: 50 random moves on inner and outer layers
    let mut rng = rand::rng();
    let mut scramble_moves = Vec::new();
    let axes = [RotationAxis::X, RotationAxis::Y, RotationAxis::Z];

    println!(
        "Scrambling a {}x{}x{} cube with 50 deep random moves...",
        size, size, size
    );
    for _ in 0..50 {
        let axis = axes[rng.random_range(0..3)];
        let index = rng.random_range(0..size);
        let direction = if rng.random_bool(0.5) {
            Direction::Clockwise
        } else {
            Direction::CounterClockwise
        };
        let m = RotationMove {
            axis,
            index,
            direction,
            add_to_history: true,
        };
        cube.apply_move(m);
        scramble_moves.push(m);
    }

    let total_centers = 6 * (size - 2) * (size - 2);
    let initial_misplaced = count_misplaced_centers(&cube);
    println!(
        "Scramble complete. Initial misplaced centers: {}/{}",
        initial_misplaced, total_centers
    );

    println!("\nGenerating center-solving macros...");
    let rotations = generate_cube_rotations();
    let mut center_bases = Vec::new();
    // Clockwise Outer Face Turn
    center_bases.push(Macro {
        name: "Outer_Face_Turn_CW".to_string(),
        setup: Vec::new(),
        macro_seq: vec![RotationMove {
            axis: RotationAxis::X,
            index: size - 1,
            direction: Direction::Clockwise,
            add_to_history: true,
        }],
        undo_setup: Vec::new(),
        cost: 1,
    });
    // CounterClockwise Outer Face Turn
    center_bases.push(Macro {
        name: "Outer_Face_Turn_CCW".to_string(),
        setup: Vec::new(),
        macro_seq: vec![RotationMove {
            axis: RotationAxis::X,
            index: size - 1,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        }],
        undo_setup: Vec::new(),
        cost: 1,
    });
    for i in 1..(size - 1) {
        // Clockwise Inner Slice Turn
        center_bases.push(Macro {
            name: format!("Inner_Slice_Turn_CW_s{}", i),
            setup: Vec::new(),
            macro_seq: vec![RotationMove {
                axis: RotationAxis::X,
                index: i,
                direction: Direction::Clockwise,
                add_to_history: true,
            }],
            undo_setup: Vec::new(),
            cost: 1,
        });
        // CounterClockwise Inner Slice Turn
        center_bases.push(Macro {
            name: format!("Inner_Slice_Turn_CCW_s{}", i),
            setup: Vec::new(),
            macro_seq: vec![RotationMove {
                axis: RotationAxis::X,
                index: i,
                direction: Direction::CounterClockwise,
                add_to_history: true,
            }],
            undo_setup: Vec::new(),
            cost: 1,
        });
        // Commutator base formulas
        for j in 1..(size - 1) {
            let comms = vec![
                (format!("Center_F_U_Right_s{i}_s{j}"), get_center1_moves(size, i, j)),
                (format!("Center_F_U_Left_s{i}_s{j}"), get_center2_moves(size, i, j)),
                (format!("Center_R_U_Back_s{i}_s{j}"), get_center3_moves(size, i, j)),
                (format!("Center_R_U_Front_s{i}_s{j}"), get_center4_moves(size, i, j)),
            ];

            for (name, comm) in comms {
                center_bases.push(Macro {
                    name: name.clone(),
                    setup: Vec::new(),
                    macro_seq: comm.clone(),
                    undo_setup: Vec::new(),
                    cost: 8,
                });

                // Add Setup + Macro + Undo Setup combos
                let outer_turns = vec![
                    (
                        "OuterU_CW",
                        vec![RotationMove {
                            axis: RotationAxis::Y,
                            index: size - 1,
                            direction: Direction::Clockwise,
                            add_to_history: true,
                        }],
                    ),
                    (
                        "OuterU_CCW",
                        vec![RotationMove {
                            axis: RotationAxis::Y,
                            index: size - 1,
                            direction: Direction::CounterClockwise,
                            add_to_history: true,
                        }],
                    ),
                    (
                        "OuterF_CW",
                        vec![RotationMove {
                            axis: RotationAxis::Z,
                            index: size - 1,
                            direction: Direction::Clockwise,
                            add_to_history: true,
                        }],
                    ),
                    (
                        "OuterR_CW",
                        vec![RotationMove {
                            axis: RotationAxis::X,
                            index: size - 1,
                            direction: Direction::Clockwise,
                            add_to_history: true,
                        }],
                    ),
                ];

                for (s_name, s_moves) in outer_turns {
                    let mut undo = s_moves.clone();
                    for m in &mut undo {
                        *m = m.inverse();
                    }
                    undo.reverse();

                    center_bases.push(Macro {
                        name: format!("{s_name}+{name}"),
                        setup: s_moves,
                        macro_seq: comm.clone(),
                        undo_setup: undo,
                        cost: 10,
                    });
                }
            }
        }
    }
    let center_macros = generate_symmetric_macros(&center_bases, &rotations, size);
    println!("Generated {} symmetric center macros.", center_macros.len());

    println!("\n--- Starting Phase 1: Solving Centers (Hybrid Architecture) ---");
    let start_time = Instant::now();
    let mut step = 1;
    let mut visited_centers = HashSet::new();
    visited_centers.insert(cube.clone());

    let max_center_steps = (total_centers * 2) as usize;
    let mut solved = false;

    let center_endgame_table = generate_center_endgame_table(&center_macros, size);
    println!(
        "Generated endgame table with {} signatures.",
        center_endgame_table.len()
    );

    let solving_order = rubik_solver::macro_solver::get_face_solving_order(&cube);
    println!("Solving order: {:?}", solving_order);
    let center_phase = SolverPhase::Phase1Centers {
        order: solving_order,
    };

    loop {
        let misplaced = count_misplaced_centers(&cube);
        println!(
            "  Step {}: Misplaced centers = {}/{}",
            step, misplaced, total_centers
        );
        if misplaced == 0 {
            println!("\n🎉 SUCCESS! All center pieces solved correctly!");
            solved = true;
            break;
        }

        let start_step = Instant::now();

        if misplaced <= 8 {
            println!("     🎯 [Endgame] Attempting lookup table...");
            let sig = get_misplaced_centers_signature(&cube);
            if let Some(mac) = center_endgame_table.get(&sig) {
                println!("     ✅ [Lookup Success] Apply macro: {}", mac.name);
                let moves = mac.all_moves();
                cube.apply_moves(&moves);
                visited_centers.insert(cube.clone());
                step += 1;
                continue;
            } else {
                println!("     ⚠️ [Lookup Fail] Signature not found in table. Falling back to beam search.");
            }
        }

        // Stage 1: Greedy Shallow Search
        let mut best_macros = solve_phase_beam_search(
            &cube,
            &center_phase,
            &center_macros,
            15, // beam_width = 15
            2,  // max_depth = 2
            &visited_centers,
        );

        if let Some(ref bm) = best_macros {
            if bm.is_empty() {
                // If stuck, fall back to deep search (beam width 300, depth 8)
                println!(
                    "     ℹ️ [Step {} stuck] Triggering Adaptive Deep Search fallback...",
                    step
                );
                best_macros = solve_phase_beam_search(
                    &cube,
                    &center_phase,
                    &center_macros,
                    300,
                    8,
                    &visited_centers,
                );
            }
        }

        if let Some(bm) = best_macros {
            if bm.is_empty() {
                println!("  ❌ FAILED: Beam search returned empty list (stuck!).");
                print_misplaced_centers_details(&cube);
                print_cube_faces_grid(&cube);
                break;
            }
            println!(
                "  -> Found path of {} macros in {:?}",
                bm.len(),
                start_step.elapsed()
            );
            for m in &bm {
                println!("     * Apply macro: {}", m.name);
                let moves = m.all_moves();
                cube.apply_moves(&moves);
                visited_centers.insert(cube.clone());
            }
        } else {
            println!("  ❌ FAILED: Beam search failed to find any path.");
            break;
        }

        step += 1;
        if step > max_center_steps {
            println!("  ❌ FAILED: Step limit exceeded!");
            break;
        }
    }

    let elapsed = start_time.elapsed();
    println!("\n=====================================================");
    println!("   TEST RESULTS: Centers Solved: {}", solved);
    println!("   Total Time: {:?}", elapsed);
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
