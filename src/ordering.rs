use crate::eval::piece_value;
use cozy_chess::*;

pub fn legal_moves(b: &Board) -> Vec<Move> {
    let mut v = Vec::with_capacity(48);
    b.generate_moves(|ms| {
        v.extend(ms);
        false
    });
    v
}

// TT move primeiro, depois capturas por MVV-LVA
pub fn order(b: &Board, moves: &mut [Move], tt_move: Option<Move>) {
    let them = b.colors(!b.side_to_move());
    moves.sort_by_cached_key(|m| {
        if Some(*m) == tt_move {
            return -1_000_000;
        }
        if them.has(m.to) {
            let victim = b.piece_on(m.to).map_or(0, piece_value);
            let attacker = b.piece_on(m.from).map_or(0, piece_value);
            -(10_000 + victim * 10 - attacker / 10)
        } else {
            0
        }
    });
}
