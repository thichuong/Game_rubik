use crate::state::Cube;

impl Cube {
    pub fn apply_moves(&mut self, moves: &str) {
        for m in moves.split_whitespace() {
            self.apply_move(m);
        }
    }
}

pub fn invert_move(m: &str) -> String {
    m.strip_suffix('\'').map_or_else(|| {
        if m.ends_with('2') {
            m.to_string()
        } else {
            format!("{m}'")
        }
    }, std::string::ToString::to_string)
}

pub fn invert_moves(moves: &str) -> String {
    moves.split_whitespace().rev().map(invert_move).collect::<Vec<_>>().join(" ")
}
