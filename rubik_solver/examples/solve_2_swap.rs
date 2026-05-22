// English: All comments in source code must be in English.
// Vietnamese: Trao đổi trong phần chat hoàn toàn bằng Tiếng Việt.

use rubik_solver::core::{Direction, Face, RotationAxis, RotationMove};
use rubik_solver::macro_solver::{
    Macro, VirtualCube, count_misplaced_centers, generate_cube_rotations,
    generate_symmetric_macros, get_center1_moves, get_center2_moves, get_center3_moves,
    get_center4_moves, count_misplaced_centers_on_face,
};
use std::collections::{HashSet, VecDeque};

#[derive(Clone)]
struct SearchNode {
    cube: VirtualCube,
    macro_indices: Vec<usize>,
}

fn main() {
    let size = 6;
    let mut cube = VirtualCube::new(size);

    // Rotate the two center cubies by 180 degrees around Y axis to swap their colors (Parity Trap)
    let pos1 = bevy::prelude::IVec3::new(5, 2, 3);
    let pos2 = bevy::prelude::IVec3::new(0, 2, 3);

    let rot_180 = bevy::prelude::Quat::from_rotation_y(std::f32::consts::PI);

    for cubie in &mut cube.cubies {
        if cubie.pos == pos1 || cubie.pos == pos2 {
            cubie.rotation = rot_180;
        }
    }

    let total_misplaced = count_misplaced_centers(&cube);
    println!("Successfully constructed the stuck state!");
    println!("Initial misplaced centers: {}", total_misplaced);
    print_face_misplaced(&cube);

    println!("\nGenerating center-solving macros (WITHOUT inner slice turns)...");
    let rotations = generate_cube_rotations();
    let mut center_bases = Vec::new();
    // Clockwise Outer Face Turn
    center_bases.push(Macro {
        name: "Outer_Face_Turn_CW".to_string(),
        moves: vec![RotationMove {
            axis: RotationAxis::X,
            index: size - 1,
            direction: Direction::Clockwise,
            add_to_history: true,
        }],
        cost: 1,
    });
    // CounterClockwise Outer Face Turn
    center_bases.push(Macro {
        name: "Outer_Face_Turn_CCW".to_string(),
        moves: vec![RotationMove {
            axis: RotationAxis::X,
            index: size - 1,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        }],
        cost: 1,
    });
    for i in 1..(size - 1) {
        // Commutator base formulas ONLY (NO slice turns here!)
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
    println!("Generated {} symmetric commutator macros.", center_macros.len());

    println!("\nSolving Centers with Parity Breakout Algorithm...");
    let mut step = 1;
    let mut global_visited = HashSet::new();
    global_visited.insert(cube.clone());

    let mut solution = Vec::new();

    loop {
        let misplaced = count_misplaced_centers(&cube);
        if misplaced == 0 {
            break;
        }

        println!("  Step {}: Misplaced centers = {}", step, misplaced);

        // Try standard beam search (width 50, depth 5)
        let mut best_macros = solve_phase_beam_search_local(
            &cube,
            &center_macros,
            50,
            5,
            &global_visited,
        );

        if let Some(ref bm) = best_macros {
            if bm.is_empty() {
                println!("    [Step {} stuck] Triggering deep search...", step);
                best_macros = solve_phase_beam_search_local(
                    &cube,
                    &center_macros,
                    300,
                    8,
                    &global_visited,
                );
            }
        }

        if let Some(ref bm) = best_macros {
            if bm.is_empty() {
                println!("    [Step {} STILL stuck] Triggering Parity Breakout...", step);
                let mut parity_fixed = false;
                for i in 1..(size - 1) {
                    let test_moves = vec![RotationMove {
                        axis: RotationAxis::X,
                        index: i,
                        direction: Direction::Clockwise,
                        add_to_history: true,
                    }];
                    let mut temp_cube = cube.clone();
                    temp_cube.apply_moves(&test_moves);

                    // Shallow search to see if this turn unlocks progress
                    let test_search = solve_phase_beam_search_local(
                        &temp_cube,
                        &center_macros,
                        50,
                        3,
                        &global_visited,
                    );
                    if let Some(test_bm) = test_search {
                        if !test_bm.is_empty() {
                            println!("    🎉 Parity breakout succeeded by turning inner slice s{}!", i);
                            cube.apply_moves(&test_moves);
                            global_visited.insert(cube.clone());
                            for &mv in &test_moves {
                                solution.push(mv);
                            }
                            parity_fixed = true;
                            break;
                        }
                    }
                }
                if parity_fixed {
                    step += 1;
                    continue;
                } else {
                    println!("    ❌ Parity breakout FAILED!");
                    break;
                }
            }
        }

        if let Some(bm) = best_macros {
            if bm.is_empty() {
                break;
            }
            for m in &bm {
                cube.apply_moves(&m.moves);
                global_visited.insert(cube.clone());
                println!("    * Apply macro: {}", m.name);
                for &mv in &m.moves {
                    solution.push(mv);
                }
            }
        } else {
            println!("    Search returned None");
            break;
        }

        step += 1;
        if step > 50 {
            break;
        }
    }

    if count_misplaced_centers(&cube) == 0 {
        println!("\n🎉 CUBE SOLVED SUCCESSFULLY IN {} STEPS!", step - 1);
        println!("Total solution moves: {}", solution.len());
    } else {
        println!("\n❌ FAILED to solve the cube.");
    }
}

// Local beam search wrapper for demonstration
fn solve_phase_beam_search_local(
    cube: &VirtualCube,
    macros: &[rubik_solver::macro_solver::SymmetricMacro],
    beam_width: usize,
    max_depth: usize,
    global_visited: &HashSet<VirtualCube>,
) -> Option<Vec<rubik_solver::macro_solver::SymmetricMacro>> {
    let initial_heuristic = count_misplaced_centers(cube);
    if initial_heuristic == 0 {
        return Some(Vec::new());
    }

    let mut current_beam = VecDeque::new();
    current_beam.push_back(SearchNode {
        cube: cube.clone(),
        macro_indices: Vec::new(),
    });

    let mut best_improvement_node: Option<SearchNode> = None;
    let mut best_heuristic = initial_heuristic;
    let mut visited_states = HashSet::new();
    visited_states.insert(cube.clone());

    for _depth in 1..=max_depth {
        let mut next_candidates = Vec::new();

        while let Some(node) = current_beam.pop_front() {
            for (mac_idx, mac) in macros.iter().enumerate() {
                let mut next_cube = node.cube.clone();
                next_cube.apply_moves(&mac.moves);

                if global_visited.contains(&next_cube) || visited_states.contains(&next_cube) {
                    continue;
                }
                visited_states.insert(next_cube.clone());

                let h = count_misplaced_centers(&next_cube);
                let next_cost = node.macro_indices.len() + 1; // using number of macros as simple cost for local testing

                let mut next_indices = node.macro_indices.clone();
                next_indices.push(mac_idx);

                let candidate = SearchNode {
                    cube: next_cube,
                    macro_indices: next_indices,
                };

                if h < best_heuristic {
                    best_heuristic = h;
                    best_improvement_node = Some(candidate.clone());
                } else if h == best_heuristic {
                    if let Some(ref best) = best_improvement_node {
                        if next_cost < best.macro_indices.len() {
                            best_improvement_node = Some(candidate.clone());
                        }
                    } else {
                        best_improvement_node = Some(candidate.clone());
                    }
                }

                next_candidates.push((h, next_cost, candidate));
            }
        }

        if next_candidates.is_empty() {
            break;
        }

        next_candidates.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));

        current_beam.clear();
        for (_, _, node) in next_candidates.iter().take(beam_width) {
            current_beam.push_back(node.clone());
        }

        if best_heuristic == 0 {
            break;
        }
    }

    if let Some(best_node) = best_improvement_node {
        if best_heuristic >= initial_heuristic {
            return Some(Vec::new());
        }
        let applied_macros = best_node
            .macro_indices
            .iter()
            .map(|&idx| macros[idx].clone())
            .collect();
        Some(applied_macros)
    } else {
        None
    }
}

fn print_face_misplaced(cube: &VirtualCube) {
    let faces = [
        Face::Right,
        Face::Left,
        Face::Up,
        Face::Down,
        Face::Front,
        Face::Back,
    ];
    for &face in &faces {
        println!("    {:?}: {}", face, count_misplaced_centers_on_face(cube, face));
    }
}
