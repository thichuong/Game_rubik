// English: All comments in source code must be in English.
// Vietnamese: Trao đổi trong phần chat hoàn toàn bằng Tiếng Việt.

use bevy::prelude::*;
use rand::RngExt;
use rubik_solver::core::{Direction, Face, RotationAxis, RotationMove};
use rubik_solver::macro_solver::{
    Macro, SolverPhase, SymmetricMacro, VirtualCube, VirtualCubie, count_misplaced_centers,
    count_unpaired_edges, generate_symmetric_macros, get_center1_moves, get_center2_moves,
    get_center3_moves, get_center4_moves, get_edge_flip_moves, get_edge_pair_moves,
    get_last_two_edges_1_moves, get_last_two_edges_2_moves, get_niklas_8_moves,
    get_oll_parity_moves, get_pll_parity_moves, get_t_perm_moves, solve_phase_beam_search,
};
use std::collections::HashSet;
use std::time::Instant;

fn main() {
    println!("==================================================");
    println!("  🧠 NxN STEP-BY-STEP CUBE SOLVER COMPARER (6x6x6) ");
    println!("==================================================");

    let size = 6;
    let mut original_cube = VirtualCube::new(size);

    // Deep scramble: 50 random moves
    let mut rng = rand::rng();
    let mut scramble_moves = Vec::new();
    let axes = [RotationAxis::X, RotationAxis::Y, RotationAxis::Z];

    println!("Scrambling cube with 50 moves...");
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
        original_cube.apply_move(m);
        scramble_moves.push(m);
    }

    println!(
        "Scramble complete. Misplaced stickers: {}",
        original_cube.count_misplaced_stickers()
    );

    // Reconstruct state_str
    let state_str = to_state_str(&original_cube);
    println!("\nGenerating state_str and parsing back to ensure consistency...");
    let mut parsed_cube = match VirtualCube::from_state_str(size, &state_str) {
        Some(c) => c,
        None => {
            println!("ERROR: from_state_str failed!");
            return;
        }
    };
    println!(
        "Consistency check: Reconstructed cube misplaced stickers = {}",
        parsed_cube.count_misplaced_stickers()
    );

    // Run solve comparison
    println!("\n==================================================");
    println!("     TESTING SOLVING ON ORIGINAL CUBE (CUBE A)    ");
    println!("==================================================");
    let mut cube_a = original_cube.clone();
    let success_a = solve_step_by_step(&mut cube_a);

    println!("\n==================================================");
    println!("   TESTING SOLVING ON RECONSTRUCTED CUBE (CUBE B) ");
    println!("==================================================");
    let mut cube_b = parsed_cube.clone();
    let success_b = solve_step_by_step(&mut cube_b);

    println!("\n==================================================");
    println!("                  COMPARISON RESULTS              ");
    println!("==================================================");
    println!("Cube A (Original)      Solve Success: {}", success_a);
    println!("Cube B (Reconstructed) Solve Success: {}", success_b);
}

fn solve_step_by_step(cube: &mut VirtualCube) -> bool {
    let size = cube.size;
    let rotations = rubik_solver::macro_solver::generate_cube_rotations();

    // 1. Generate Macros (Same as production solver)
    let mut center_bases = Vec::new();
    for i in 1..(size - 1) {
        center_bases.push(Macro {
            name: format!("Inner_Slice_Turn_s{}", i),
            moves: vec![RotationMove {
                axis: RotationAxis::X,
                index: i,
                direction: Direction::Clockwise,
                add_to_history: true,
            }],
            cost: 1,
        });
        for j in 1..(size - 1) {
            center_bases.push(Macro {
                name: format!("Center_F_U_Right_s{}_s{}", i, j),
                moves: get_center1_moves(size, i, j),
                cost: 8,
            });
            center_bases.push(Macro {
                name: format!("Center_F_U_Left_s{}_s{}", i, j),
                moves: get_center2_moves(size, i, j),
                cost: 8,
            });
            center_bases.push(Macro {
                name: format!("Center_R_U_Back_s{}_s{}", i, j),
                moves: get_center3_moves(size, i, j),
                cost: 8,
            });
            center_bases.push(Macro {
                name: format!("Center_R_U_Front_s{}_s{}", i, j),
                moves: get_center4_moves(size, i, j),
                cost: 8,
            });
        }
    }
    let center_macros = generate_symmetric_macros(&center_bases, &rotations, size);

    let mut edge_bases = Vec::new();
    edge_bases.push(Macro {
        name: "Outer_Face_Turn".to_string(),
        moves: vec![RotationMove {
            axis: RotationAxis::X,
            index: size - 1,
            direction: Direction::Clockwise,
            add_to_history: true,
        }],
        cost: 1,
    });
    edge_bases.push(Macro {
        name: "Edge_Flip".to_string(),
        moves: get_edge_flip_moves(size),
        cost: 7,
    });
    for i in 1..(size - 1) {
        edge_bases.push(Macro {
            name: format!("Edge_Pair_R_F_s{}", i),
            moves: get_edge_pair_moves(size, i),
            cost: 9,
        });
        edge_bases.push(Macro {
            name: format!("Last_Two_Edges_1_s{}", i),
            moves: get_last_two_edges_1_moves(size, i),
            cost: 9,
        });
        edge_bases.push(Macro {
            name: format!("Last_Two_Edges_2_s{}", i),
            moves: get_last_two_edges_2_moves(size, i),
            cost: 9,
        });
    }
    let edge_macros = generate_symmetric_macros(&edge_bases, &rotations, size);

    let stage3_bases = vec![
        Macro {
            name: "Outer_Face_Turn".to_string(),
            moves: vec![RotationMove {
                axis: RotationAxis::X,
                index: size - 1,
                direction: Direction::Clockwise,
                add_to_history: true,
            }],
            cost: 1,
        },
        Macro {
            name: "Corner_Cycle_Niklas".to_string(),
            moves: get_niklas_8_moves(size),
            cost: 8,
        },
        Macro {
            name: "Corner_Swap_T_Perm".to_string(),
            moves: get_t_perm_moves(size),
            cost: 15,
        },
        Macro {
            name: "PLL_Parity".to_string(),
            moves: get_pll_parity_moves(size),
            cost: 12,
        },
        Macro {
            name: "OLL_Parity".to_string(),
            moves: get_oll_parity_moves(size),
            cost: 25,
        },
        Macro {
            name: "Edge_Flip_Stage3".to_string(),
            moves: get_edge_flip_moves(size),
            cost: 7,
        },
    ];
    let stage3_macros = generate_symmetric_macros(&stage3_bases, &rotations, size);

    // =========================================================================
    // PHASE 1: Solving Centers
    // =========================================================================
    println!("\n--- [PHASE 1] SOLVING CENTERS ---");
    let mut step = 1;
    let mut visited_centers = HashSet::new();
    visited_centers.insert(cube.clone());

    let total_centers = 6 * (size - 2) * (size - 2);
    let max_center_steps = (total_centers * 2) as usize;

    let mut centers_solved = false;
    loop {
        let misplaced = count_misplaced_centers(cube);
        println!(
            "  Step {}: Misplaced centers = {}/{}",
            step, misplaced, total_centers
        );
        if misplaced == 0 {
            println!("  🎉 [PHASE 1 SUCCESS] All centers placed correctly!");
            centers_solved = true;
            break;
        }

        let start = Instant::now();
        if let Some(best_macros) = solve_phase_beam_search(
            cube,
            SolverPhase::Phase1Centers,
            &center_macros,
            300,
            8,
            &visited_centers,
        ) {
            if best_macros.is_empty() {
                println!("  ❌ [PHASE 1 FAILED] Beam search returned empty list (stuck!).");
                break;
            }
            println!(
                "  -> Found path of {} macros in {:?}",
                best_macros.len(),
                start.elapsed()
            );
            for m in &best_macros {
                println!("     * Apply center macro: {}", m.name);
                cube.apply_moves(&m.moves);
                visited_centers.insert(cube.clone());
            }
        } else {
            println!("  ❌ [PHASE 1 FAILED] Beam search failed to find any path.");
            break;
        }

        step += 1;
        if step > max_center_steps {
            println!("  ❌ [PHASE 1 FAILED] Step limit exceeded!");
            break;
        }
    }

    if !centers_solved {
        return false;
    }

    // =========================================================================
    // PHASE 2: Pairing Edges
    // =========================================================================
    println!("\n--- [PHASE 2] PAIRING EDGES ---");
    step = 1;
    let mut visited_edges = HashSet::new();
    visited_edges.insert(cube.clone());

    let total_edges = 12 * (size - 2);
    let max_edge_steps = (total_edges * 2) as usize;

    let mut edges_solved = false;
    loop {
        let unpaired = count_unpaired_edges(cube);
        println!(
            "  Step {}: Unpaired edges = {}/{}",
            step, unpaired, total_edges
        );
        if unpaired == 0 {
            println!("  🎉 [PHASE 2 SUCCESS] All edges paired!");
            edges_solved = true;
            break;
        }

        let start = Instant::now();
        if let Some(best_macros) = solve_phase_beam_search(
            cube,
            SolverPhase::Phase2Edges,
            &edge_macros,
            300,
            8,
            &visited_edges,
        ) {
            if best_macros.is_empty() {
                println!("  ❌ [PHASE 2 FAILED] Beam search returned empty list (stuck!).");
                break;
            }
            println!(
                "  -> Found path of {} macros in {:?}",
                best_macros.len(),
                start.elapsed()
            );
            for m in &best_macros {
                println!("     * Apply edge macro: {}", m.name);
                cube.apply_moves(&m.moves);
                visited_edges.insert(cube.clone());
            }
        } else {
            println!("  ❌ [PHASE 2 FAILED] Beam search failed to find any path.");
            break;
        }

        step += 1;
        if step > max_edge_steps {
            println!("  ❌ [PHASE 2 FAILED] Step limit exceeded!");
            break;
        }
    }

    if !edges_solved {
        return false;
    }

    // =========================================================================
    // PHASE 3: Corners, Edges and Parities (3x3x3 stage)
    // =========================================================================
    println!("\n--- [PHASE 3] SOLVING 3x3x3 STAGE ---");
    step = 1;
    let mut visited_stage3 = HashSet::new();
    visited_stage3.insert(cube.clone());
    let max_stage3_steps = 50;

    let mut stage3_solved = false;
    loop {
        let misplaced = cube.count_misplaced_stickers();
        println!("  Step {}: Misplaced stickers = {}", step, misplaced);
        if misplaced == 0 {
            println!("  🎉 [ALL PHASES SUCCESS] Rubik is 100% solved!");
            stage3_solved = true;
            break;
        }

        let start = Instant::now();
        if let Some(best_macros) = solve_phase_beam_search(
            cube,
            SolverPhase::Phase3CornersAndParity,
            &stage3_macros,
            50,
            6,
            &visited_stage3,
        ) {
            if best_macros.is_empty() {
                println!("  ❌ [PHASE 3 FAILED] Beam search returned empty list (stuck!).");
                break;
            }
            println!(
                "  -> Found path of {} macros in {:?}",
                best_macros.len(),
                start.elapsed()
            );
            for m in &best_macros {
                println!("     * Apply 3x3 macro: {}", m.name);
                cube.apply_moves(&m.moves);
                visited_stage3.insert(cube.clone());
            }
        } else {
            println!("  ❌ [PHASE 3 FAILED] Beam search failed to find any path.");
            break;
        }

        step += 1;
        if step > max_stage3_steps {
            println!("  ❌ [PHASE 3 FAILED] Step limit exceeded!");
            break;
        }
    }

    stage3_solved
}

fn to_state_str(cube: &VirtualCube) -> String {
    let size = cube.size;
    let size_usize = size as usize;
    let mut state = vec![' '; 6 * size_usize * size_usize];

    let get_cubie =
        |pos: IVec3| -> &VirtualCubie { cube.cubies.iter().find(|c| c.pos == pos).unwrap() };

    let char_for_face = |f: Face| -> char {
        match f {
            Face::Up => 'U',
            Face::Down => 'D',
            Face::Right => 'R',
            Face::Left => 'L',
            Face::Front => 'F',
            Face::Back => 'B',
        }
    };

    for row in 0..size_usize {
        for col in 0..size_usize {
            let r = row as i32;
            let c = col as i32;

            // Face 0: Up (y = size - 1)
            // x = col, z = row
            {
                let cubie = get_cubie(IVec3::new(c, size - 1, r));
                let local_dir = cubie.rotation.inverse() * Vec3::Y;
                let f = Face::from_normal(local_dir).unwrap();
                state[0 * size_usize * size_usize + row * size_usize + col] = char_for_face(f);
            }

            // Face 1: Right (x = size - 1)
            // z = size - 1 - col, y = size - 1 - row
            {
                let cubie = get_cubie(IVec3::new(size - 1, size - 1 - r, size - 1 - c));
                let local_dir = cubie.rotation.inverse() * Vec3::X;
                let f = Face::from_normal(local_dir).unwrap();
                state[1 * size_usize * size_usize + row * size_usize + col] = char_for_face(f);
            }

            // Face 2: Front (z = size - 1)
            // x = col, y = size - 1 - row
            {
                let cubie = get_cubie(IVec3::new(c, size - 1 - r, size - 1));
                let local_dir = cubie.rotation.inverse() * Vec3::Z;
                let f = Face::from_normal(local_dir).unwrap();
                state[2 * size_usize * size_usize + row * size_usize + col] = char_for_face(f);
            }

            // Face 3: Down (y = 0)
            // x = col, z = size - 1 - row
            {
                let cubie = get_cubie(IVec3::new(c, 0, size - 1 - r));
                let local_dir = cubie.rotation.inverse() * Vec3::NEG_Y;
                let f = Face::from_normal(local_dir).unwrap();
                state[3 * size_usize * size_usize + row * size_usize + col] = char_for_face(f);
            }

            // Face 4: Left (x = 0)
            // z = col, y = size - 1 - row
            {
                let cubie = get_cubie(IVec3::new(0, size - 1 - r, c));
                let local_dir = cubie.rotation.inverse() * Vec3::NEG_X;
                let f = Face::from_normal(local_dir).unwrap();
                state[4 * size_usize * size_usize + row * size_usize + col] = char_for_face(f);
            }

            // Face 5: Back (z = 0)
            // x = size - 1 - col, y = size - 1 - row
            {
                let cubie = get_cubie(IVec3::new(size - 1 - c, size - 1 - r, 0));
                let local_dir = cubie.rotation.inverse() * Vec3::NEG_Z;
                let f = Face::from_normal(local_dir).unwrap();
                state[5 * size_usize * size_usize + row * size_usize + col] = char_for_face(f);
            }
        }
    }

    state.into_iter().collect()
}
