// English: All comments in source code must be in English.
// Vietnamese: Trao đổi trong phần chat hoàn toàn bằng Tiếng Việt.

use bevy::prelude::*;
use rubik_solver::core::{Direction, Face, RotationAxis, RotationMove};

#[derive(Clone, Debug)]
struct VirtualCubie {
    pos: IVec3,
    rotation: Quat,
}

#[derive(Clone, Debug)]
pub struct VirtualCube {
    size: i32,
    cubies: Vec<VirtualCubie>,
}

impl VirtualCube {
    pub fn new(size: i32) -> Self {
        let mut cubies = Vec::new();
        for x in 0..size {
            for y in 0..size {
                for z in 0..size {
                    if x > 0 && x < size - 1 && y > 0 && y < size - 1 && z > 0 && z < size - 1 {
                        continue;
                    }
                    cubies.push(VirtualCubie {
                        pos: IVec3::new(x, y, z),
                        rotation: Quat::IDENTITY,
                    });
                }
            }
        }
        Self { size, cubies }
    }

    pub fn apply_move(&mut self, m: RotationMove) {
        let size = self.size;
        let (axis_vec, angle) = m.get_rotation_info();
        let rot_step = Quat::from_axis_angle(axis_vec, angle);
        let offset = (size as f32 - 1.0) / 2.0;

        for cubie in &mut self.cubies {
            if m.is_cubie_at_slice(cubie.pos) {
                // Rotate logical coordinate
                let centered = cubie.pos.as_vec3() - Vec3::splat(offset);
                let rotated = rot_step * centered;
                cubie.pos = (rotated + Vec3::splat(offset)).round().as_ivec3();

                // Rotate orientation
                cubie.rotation = (rot_step * cubie.rotation).normalize();
            }
        }
    }
}

fn get_niklas_8_moves() -> Vec<RotationMove> {
    // U R U' L' U R' U' L
    vec![
        RotationMove {
            axis: RotationAxis::Y,
            index: 3,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::X,
            index: 3,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Y,
            index: 3,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::X,
            index: 0,
            direction: Direction::Clockwise,
            add_to_history: true,
        }, // L'
        RotationMove {
            axis: RotationAxis::Y,
            index: 3,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::X,
            index: 3,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Y,
            index: 3,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::X,
            index: 0,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        }, // L
    ]
}

fn get_t_perm_moves() -> Vec<RotationMove> {
    // R U R' U' R' F R2 U' R' U' R U R' F'
    vec![
        RotationMove {
            axis: RotationAxis::X,
            index: 3,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Y,
            index: 3,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::X,
            index: 3,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Y,
            index: 3,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::X,
            index: 3,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Z,
            index: 3,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::X,
            index: 3,
            direction: Direction::Clockwise,
            add_to_history: true,
        }, // R2 part 1
        RotationMove {
            axis: RotationAxis::X,
            index: 3,
            direction: Direction::Clockwise,
            add_to_history: true,
        }, // R2 part 2
        RotationMove {
            axis: RotationAxis::Y,
            index: 3,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::X,
            index: 3,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Y,
            index: 3,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::X,
            index: 3,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Y,
            index: 3,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::X,
            index: 3,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Z,
            index: 3,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
    ]
}

fn main() {
    let size = 4;
    let original = VirtualCube::new(size);

    // Test Niklas 8-moves
    {
        let mut cube = original.clone();
        let moves = get_niklas_8_moves();
        for m in moves {
            cube.apply_move(m);
        }
        let mut moved_count = 0;
        for (i, c) in cube.cubies.iter().enumerate() {
            let orig = &original.cubies[i];
            let pos_diff = c.pos != orig.pos;
            let rot_diff = c.rotation.dot(orig.rotation).abs() < 0.99;
            if pos_diff || rot_diff {
                moved_count += 1;
            }
        }
        println!("Niklas 8-moves moved cubies: {}", moved_count);
    }

    // Test T-Perm
    {
        let mut cube = original.clone();
        let moves = get_t_perm_moves();
        for m in moves {
            cube.apply_move(m);
        }
        let mut moved_count = 0;
        for (i, c) in cube.cubies.iter().enumerate() {
            let orig = &original.cubies[i];
            let pos_diff = c.pos != orig.pos;
            let rot_diff = c.rotation.dot(orig.rotation).abs() < 0.99;
            if pos_diff || rot_diff {
                moved_count += 1;
            }
        }
        println!("T-Perm moved cubies: {}", moved_count);
    }
}
