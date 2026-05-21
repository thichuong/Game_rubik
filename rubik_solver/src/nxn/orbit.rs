use crate::nxn::state::{FACES_ORDER, NxNState};
use crate::core::Face;
use bevy::prelude::IVec3;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Orbit {
    pub positions: Vec<IVec3>,
}

pub fn get_orbits(size: usize) -> Vec<Orbit> {
    let mut orbits = Vec::new();
    let mut visited = HashSet::new();

    for &face in &FACES_ORDER {
        for row in 1..(size - 1) {
            for col in 1..(size - 1) {
                if size % 2 == 1 && row == size / 2 && col == size / 2 {
                    continue;
                }

                let coord = NxNState::get_logical_coord(face, row, col, size).unwrap();
                if visited.contains(&coord) {
                    continue;
                }

                // Identify all positions in this orbit
                let mut orbit_positions = Vec::new();

                // An orbit is defined by the set of stickers that can reach each other
                // through any number of face rotations and slice moves.
                // For centers, these are pieces that have the same "radial distance" from the center of the face.
                // Formally, a piece at (face, row, col) is in the same orbit as others
                // if their coordinates (r, c) relative to the face center are permutations
                // and sign changes of each other.

                // Simplified: (r, c) on any face such that {r, N-1-r, c, N-1-c} is the same set.
                let r = row;
                let c = col;
                let s = size - 1;

                let vals = [r, c, s - r, s - c];

                for &f in &FACES_ORDER {
                    for r_i in 1..(size - 1) {
                        for c_i in 1..(size - 1) {
                            let r2 = r_i;
                            let c2 = c_i;
                            let vals2 = [r2, c2, s - r2, s - c2];

                            // Check if the set of values is the same (multi-set)
                            let mut v1 = vals.to_vec();
                            let mut v2 = vals2.to_vec();
                            v1.sort();
                            v2.sort();

                            if v1 == v2 {
                                let coord2 = NxNState::get_logical_coord(f, r_i, c_i, size).unwrap();
                                orbit_positions.push(coord2);
                                visited.insert(coord2);
                            }
                        }
                    }
                }
                orbits.push(Orbit { positions: orbit_positions });
            }
        }
    }
    orbits
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_orbits_4x4() {
        let orbits = get_orbits(4);
        assert_eq!(orbits.len(), 1);
        assert_eq!(orbits[0].positions.len(), 24);
    }

    #[test]
    fn test_get_orbits_5x5() {
        let orbits = get_orbits(5);
        assert_eq!(orbits.len(), 2);
        assert_eq!(orbits[0].positions.len(), 24);
        assert_eq!(orbits[1].positions.len(), 24);
    }

    #[test]
    fn test_get_orbits_6x6() {
        let orbits = get_orbits(6);
        // (1,1) type: 24
        // (1,2) type: 48 (Wait, (1,2) and (1,3) are the same orbit?)
        // r=1, c=2, s=5. vals = [1, 2, 4, 3].
        // r=1, c=3, s=5. vals = [1, 3, 4, 2]. Yes.
        // So (1,2) orbit has 8 positions per face? No, 24 positions total if it's "oblique" but symmetric?
        // Let's see: (1,2), (1,3), (2,1), (2,4), (3,1), (3,4), (4,2), (4,3). That's 8 per face.
        // 8 * 6 = 48.
        // (2,2) type: 24.
        // Total centers per face: 4 * 4 = 16.
        // 16 * 6 = 96.
        // 24 + 48 + 24 = 96. Correct.
        assert_eq!(orbits.len(), 3);
        let mut lens: Vec<usize> = orbits.iter().map(|o| o.positions.len()).collect();
        lens.sort();
        assert_eq!(lens, vec![24, 24, 48]);
    }
}
