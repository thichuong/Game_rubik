// Example to verify Rubik's Cube solving consistency for 4x4 and 5x5:
// 1. Spawns a mock Rubik's cube using Bevy entities (Parent-Child hierarchy) with proper coordinates.
// 2. Shuffles the cube using the random rotation logic from src/ui/interactions.rs.
// 3. Solves the cube:
//    - Call the unified API `solve_cube_for_size` on Bevy Entities,
//      parse the resulting logic string moves into physical moves, and apply
//      them instantly to the Bevy entities.
// 4. Verifies that the Bevy cube successfully returns to its 100% solved state.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::uninlined_format_args,
    clippy::missing_const_for_fn,
    clippy::similar_names,
    clippy::too_many_lines,
    clippy::type_complexity
)]

use bevy::prelude::*;
use game_rubik::rubik::components::{Cubie, CubieFace, GridCoord, RubikCube};
use rubik_solver::core::{Direction, Face, FaceMapping, RotationAxis, RotationMove};

// Simple LCG PRNG for deterministic scrambles without external dependencies
struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u32(&mut self) -> u32 {
        self.state = self
            .state
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223);
        (self.state >> 32) as u32
    }

    fn next_range(&mut self, min: usize, max: usize) -> usize {
        let range = max - min;
        if range == 0 {
            return min;
        }
        min + (self.next_u32() as usize % range)
    }
}

// Spawns a mock Rubik's cube with Bevy entities, mimicking spawn_rubik_cube_internal
fn spawn_mock_rubik_cube(commands: &mut Commands, size: i32) -> Entity {
    let cube_root = commands
        .spawn((
            RubikCube,
            Transform::IDENTITY,
            GlobalTransform::IDENTITY,
            InheritedVisibility::default(),
        ))
        .id();

    let offset = (size as f32 - 1.0) / 2.0;
    let scale = 3.0 / size as f32;
    let current_gap = 1.02 * scale; // GAP = 1.02

    for x in 0..size {
        for y in 0..size {
            for z in 0..size {
                // Skip inner cubies
                if x > 0 && x < size - 1 && y > 0 && y < size - 1 && z > 0 && z < size - 1 {
                    continue;
                }

                let grid_coord = IVec3::new(x, y, z);
                let position = (grid_coord.as_vec3() - Vec3::splat(offset)) * current_gap;

                let cubie_id = commands
                    .spawn((
                        Cubie,
                        GridCoord(grid_coord),
                        Transform::from_translation(position).with_scale(Vec3::splat(scale)),
                        GlobalTransform::default(),
                        InheritedVisibility::default(),
                    ))
                    .id();

                commands.entity(cube_root).add_child(cubie_id);

                let add_mock_face = |cmds: &mut Commands, parent: Entity, face: Face| {
                    let normal = face.normal();
                    let translation = normal * 0.501;
                    let rotation = Quat::from_rotation_arc(Vec3::Z, normal);
                    let face_id = cmds
                        .spawn((
                            Transform::from_translation(translation).with_rotation(rotation),
                            GlobalTransform::default(),
                            InheritedVisibility::default(),
                            CubieFace(face),
                        ))
                        .id();
                    cmds.entity(parent).add_child(face_id);
                };

                if x == size - 1 {
                    add_mock_face(commands, cubie_id, Face::Right);
                } else if x == 0 {
                    add_mock_face(commands, cubie_id, Face::Left);
                }
                if y == size - 1 {
                    add_mock_face(commands, cubie_id, Face::Up);
                } else if y == 0 {
                    add_mock_face(commands, cubie_id, Face::Down);
                }
                if z == size - 1 {
                    add_mock_face(commands, cubie_id, Face::Front);
                } else if z == 0 {
                    add_mock_face(commands, cubie_id, Face::Back);
                }
            }
        }
    }
    cube_root
}

// Applies a single RotationMove instantly to the Bevy cubies
fn apply_move_to_bevy_cube(
    m: RotationMove,
    size: i32,
    cubies: &mut Query<(Entity, &mut Transform, &mut GridCoord), (With<Cubie>, Without<RubikCube>)>,
) {
    let (axis_vec, angle) = m.get_rotation_info();
    let rot_step = Quat::from_axis_angle(axis_vec, angle);
    let offset = (size as f32 - 1.0) / 2.0;
    let scale = 3.0 / size as f32;
    let current_gap = 1.02 * scale;

    for (_, mut transform, mut coord) in cubies.iter_mut() {
        if m.is_cubie_at_slice(coord.0) {
            // Update logical coordinates
            coord.rotate(axis_vec, angle, size);
            // Update 3D position
            transform.translation = (coord.0.as_vec3() - Vec3::splat(offset)) * current_gap;
            // Update 3D rotation
            transform.rotation = (rot_step * transform.rotation).normalize();
        }
    }
}

// Shuffles the Bevy Rubik's cube using interactions.rs logic with a deterministic seed
fn shuffle_bevy_cube(
    size: i32,
    rng: &mut SimpleRng,
    cubies: &mut Query<(Entity, &mut Transform, &mut GridCoord), (With<Cubie>, Without<RubikCube>)>,
) -> Vec<RotationMove> {
    let mut moves = Vec::new();
    for _ in 0..20 {
        let axis = match rng.next_range(0, 3) {
            0 => RotationAxis::X,
            1 => RotationAxis::Y,
            _ => RotationAxis::Z,
        };

        let index = match rng.next_range(0, 2) {
            0 => 0,
            _ => size - 1,
        };

        let direction = match rng.next_range(0, 2) {
            0 => Direction::Clockwise,
            _ => Direction::CounterClockwise,
        };

        let m = RotationMove {
            axis,
            index,
            direction,
            add_to_history: true,
        };

        apply_move_to_bevy_cube(m, size, cubies);
        moves.push(m);
    }
    moves
}

fn run_verification_test(size: i32, seed: u64, table: &kewb::DataTable) {
    println!("\n==================================================");
    println!("     START VERIFICATION TEST FOR {size}x{size} CUBE");
    println!("==================================================");

    // 1. Create a minimal Bevy App
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::transform::TransformPlugin);

    // 2. Spawn Mock Cube
    let cube_entity = spawn_mock_rubik_cube(&mut app.world_mut().commands(), size);

    // Run updates twice to guarantee hierarchy and GlobalTransform propagation are completely initialized
    app.update();
    app.update();

    // 3. Shuffle Cube using interactions.rs logic with deterministic seed
    let mut rng = SimpleRng::new(seed);
    println!("1. Shuffling mock Bevy cube with 20 moves...");

    // Direct mutation
    let mut cell = bevy::ecs::system::SystemState::<(
        Query<(Entity, &mut Transform, &mut GridCoord), (With<Cubie>, Without<RubikCube>)>,
    )>::new(app.world_mut());
    let (mut query,) = cell.get_mut(app.world_mut());
    let _scramble_moves = shuffle_bevy_cube(size, &mut rng, &mut query);
    println!("   Scramble applied successfully!");

    // Run updates to propagate the scrambled transforms
    app.update();

    let mapping = FaceMapping::default();

    println!("\n2. Executing Solver: Solving via Bevy Entity solver integration...");

    // Refresh state queries from Bevy
    let mut system_state =
        bevy::ecs::system::SystemState::<Query<(&CubieFace, &GlobalTransform)>>::new(
            app.world_mut(),
        );
    let bevy_faces = system_state.get(app.world());
    let cube_transform = app
        .world()
        .get::<GlobalTransform>(cube_entity)
        .copied()
        .unwrap_or(GlobalTransform::IDENTITY);

    // Call unified solver API directly on Bevy entities
    let Some(state_str) =
        rubik_solver::helpers::get_cube_state_for_size(size, &bevy_faces, &cube_transform, mapping)
    else {
        eprintln!("Failed to get scrambled cube state!");
        return;
    };
    let Some(solution_moves_strings) =
        rubik_solver::solver::solve_cube_for_size(size, &state_str, table)
    else {
        eprintln!("Unified Bevy solver failed to find a solution!");
        return;
    };

    println!(
        "   Solver returned a sequence of {} moves.",
        solution_moves_strings.len()
    );
    let solution_moves_joined = solution_moves_strings.join(" ");

    // Parse logic moves string into physical moves
    let solution_physical_moves = rubik_solver::helpers::logical_string_to_physical_moves_any(
        &solution_moves_joined,
        size,
        mapping,
    );

    // Apply the solution physical moves instantly to Bevy entities in our mock App
    let mut cell = bevy::ecs::system::SystemState::<(
        Query<(Entity, &mut Transform, &mut GridCoord), (With<Cubie>, Without<RubikCube>)>,
    )>::new(app.world_mut());
    let (mut query,) = cell.get_mut(app.world_mut());

    for m in solution_physical_moves {
        apply_move_to_bevy_cube(m, size, &mut query);
    }

    // Propagate all applied moves to compute the final GlobalTransform positions
    app.update();

    // 4. Verification of Bevy entities solved state
    let mut system_state =
        bevy::ecs::system::SystemState::<Query<(&CubieFace, &GlobalTransform)>>::new(
            app.world_mut(),
        );
    let final_bevy_faces = system_state.get(app.world());
    let final_cube_transform = app
        .world()
        .get::<GlobalTransform>(cube_entity)
        .copied()
        .unwrap_or(GlobalTransform::IDENTITY);
    let Some(final_bevy_state_str) = rubik_solver::helpers::get_cube_state_for_size(
        size,
        &final_bevy_faces,
        &final_cube_transform,
        mapping,
    ) else {
        eprintln!("Failed to scrape final Bevy state!");
        return;
    };

    let solved_target = format!(
        "{}{}{}{}{}{}",
        "U".repeat((size * size) as usize),
        "R".repeat((size * size) as usize),
        "F".repeat((size * size) as usize),
        "D".repeat((size * size) as usize),
        "L".repeat((size * size) as usize),
        "B".repeat((size * size) as usize),
    );

    if final_bevy_state_str == solved_target {
        println!("   [SUCCESS] Bevy entities fully solved 100% using parsed solver moves!");
    } else {
        eprintln!(
            "   [FAILED] Bevy state mismatch!\n   Expected: {}\n   Got:      {}",
            solved_target, final_bevy_state_str
        );
        return;
    }

    println!("\n==================================================");
    println!("    [VERIFICATION SUCCESSFUL] {size}x{size} CUBE IS 100% SOLVED!");
    println!("==================================================\n");
}

fn main() {
    println!("Loading Kociemba 2-phase data table...");
    let table = kewb::DataTable::default();
    println!("Data table successfully loaded!\n");

    // Verify 4x4 with complex deterministic scramble
    run_verification_test(4, 12345, &table);

    // Verify 5x5 with complex deterministic scramble
    run_verification_test(5, 54321, &table);
}
