// Decomposes the center pieces of an nxn Rubik's cube into independent orbits.
// All comments in source files must be in English.

use crate::cube::Face;

/// Represents a single center piece of the Rubik's cube.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CenterPiece {
    pub face: Face,
    pub row: usize,
    pub col: usize,
}

/// Represents an independent orbit of center pieces.
/// Every mobile center orbit in a Rubik's cube has exactly 24 pieces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Orbit {
    pub d_min: usize,
    pub d_max: usize,
    pub sub_orbit: usize, // 0 for diagonal, axis, or first sub-orbit. 1 for second sub-orbit of oblique pieces.
    pub pieces: Vec<CenterPiece>,
}

/// Decomposes all mobile center pieces of an nxn Rubik's cube into independent orbits.
pub fn decompose_orbits(size: usize) -> Vec<Orbit> {
    if size < 4 {
        return Vec::new(); // Cubes smaller than 4x4 do not have mobile center pieces.
    }

    let mut orbits = Vec::new();

    // We scan the unique orbit keys (d_min, d_max) where 1 <= d_min <= d_max <= (size-1)/2.
    let max_d = (size - 1) / 2;

    for d_min in 1..=max_d {
        for d_max in d_min..=max_d {
            // Skip the absolute central piece of odd cubes which is fixed.
            if size % 2 == 1 && d_min == size / 2 && d_max == size / 2 {
                continue;
            }

            if d_min == d_max {
                // Diagonal orbit: exactly 1 orbit of 24 pieces (4 pieces per face * 6 faces)
                let mut pieces = Vec::with_capacity(24);
                let faces = [Face::U, Face::D, Face::F, Face::B, Face::L, Face::R];
                for &face in &faces {
                    let r0 = d_min;
                    let c0 = d_min;
                    let r1 = size - 1 - d_min;
                    let c1 = size - 1 - d_min;

                    pieces.push(CenterPiece {
                        face,
                        row: r0,
                        col: c0,
                    });
                    pieces.push(CenterPiece {
                        face,
                        row: r0,
                        col: c1,
                    });
                    pieces.push(CenterPiece {
                        face,
                        row: r1,
                        col: c0,
                    });
                    pieces.push(CenterPiece {
                        face,
                        row: r1,
                        col: c1,
                    });
                }
                orbits.push(Orbit {
                    d_min,
                    d_max,
                    sub_orbit: 0,
                    pieces,
                });
            } else if size % 2 == 1 && d_max == size / 2 {
                // Central axis orbit for odd cubes (e.g. (1, 3) or (2, 3) on 7x7)
                // This does NOT split because the pieces lie on the main axis of symmetry.
                // There are exactly 4 pieces per face * 6 faces = 24 pieces.
                let mut pieces = Vec::with_capacity(24);
                let faces = [Face::U, Face::D, Face::F, Face::B, Face::L, Face::R];
                let mid = size / 2;
                let n_1 = size - 1;

                for &face in &faces {
                    // 4 positions on the central cross:
                    // 1. (d_min, mid)
                    // 2. (mid, n_1 - d_min)
                    // 3. (n_1 - d_min, mid)
                    // 4. (mid, d_min)
                    pieces.push(CenterPiece {
                        face,
                        row: d_min,
                        col: mid,
                    });
                    pieces.push(CenterPiece {
                        face,
                        row: mid,
                        col: n_1 - d_min,
                    });
                    pieces.push(CenterPiece {
                        face,
                        row: n_1 - d_min,
                        col: mid,
                    });
                    pieces.push(CenterPiece {
                        face,
                        row: mid,
                        col: d_min,
                    });
                }
                orbits.push(Orbit {
                    d_min,
                    d_max,
                    sub_orbit: 0,
                    pieces,
                });
            } else {
                // General oblique orbit (chiral): split into 2 independent sub-orbits of 24 pieces each (Group A and Group B).
                // They are physically and mathematically isolated orbits, having independent parity states.
                let mut pieces_a = Vec::with_capacity(24);
                let mut pieces_b = Vec::with_capacity(24);

                let faces = [Face::U, Face::D, Face::F, Face::B, Face::L, Face::R];
                for &face in &faces {
                    let r0 = d_min;
                    let c0 = d_max;
                    let n_1 = size - 1;

                    // Group A (canonical CW rotation of (r0, c0)):
                    let group_a_pos = [
                        (r0, c0),
                        (c0, n_1 - r0),
                        (n_1 - r0, n_1 - c0),
                        (n_1 - c0, r0),
                    ];

                    // Group B (canonical CW rotation of (c0, r0)):
                    let group_b_pos = [
                        (c0, r0),
                        (r0, n_1 - c0),
                        (n_1 - c0, n_1 - r0),
                        (n_1 - r0, c0),
                    ];

                    for &(r, c) in &group_a_pos {
                        pieces_a.push(CenterPiece {
                            face,
                            row: r,
                            col: c,
                        });
                    }
                    for &(r, c) in &group_b_pos {
                        pieces_b.push(CenterPiece {
                            face,
                            row: r,
                            col: c,
                        });
                    }
                }

                orbits.push(Orbit {
                    d_min,
                    d_max,
                    sub_orbit: 0,
                    pieces: pieces_a,
                });
                orbits.push(Orbit {
                    d_min,
                    d_max,
                    sub_orbit: 1,
                    pieces: pieces_b,
                });
            }
        }
    }

    // Sort to make output order stable and strategically prioritize orbits:
    // 1. Diagonal orbits first (priority 0) solved by isolated slice-slice commutators.
    // 2. Axis orbits second (priority 1) solved by isolated slice-slice/slice-face commutators.
    // 3. Oblique orbits last (priority 2) solved by slice-face commutators (after other orbits are fully solved and protected).
    orbits.sort_by_key(|o| {
        let is_diagonal = o.d_min == o.d_max;
        let is_axis = size % 2 == 1 && o.d_max == size / 2;
        let priority = if is_diagonal {
            0 // Diagonal first (solved by isolated slice-slice commutators)
        } else if is_axis {
            1 // Axis second
        } else {
            2 // Oblique last (solved with maximum commutator freedom)
        };
        (priority, o.d_min, o.d_max)
    });
    orbits
}
