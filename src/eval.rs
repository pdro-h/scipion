use cozy_chess::*;

pub fn piece_value(p: Piece) -> i32 {
    match p {
        Piece::Pawn => 100,
        Piece::Knight => 320,
        Piece::Bishop => 330,
        Piece::Rook => 500,
        Piece::Queen => 900,
        Piece::King => 0,
    }
}

// Avaliação do ponto de vista de quem joga.
// TODO: piece-square tables + tapered eval, depois NNUE
pub fn evaluate(b: &Board) -> i32 {
    let mut s = 0;
    for &p in &Piece::ALL {
        let w = (b.pieces(p) & b.colors(Color::White)).len() as i32;
        let k = (b.pieces(p) & b.colors(Color::Black)).len() as i32;
        s += piece_value(p) * (w - k);
    }
    if b.side_to_move() == Color::White { s } else { -s }
}
