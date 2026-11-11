use cozy_chess::*;

const PHASE_TOTAL: i32 = 24;

const MG_PST: [[i32; 64]; 6] = [[0; 64]; 6];
const EG_PST: [[i32; 64]; 6] = [[0; 64]; 6];

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

fn phase_weight(p: Piece) -> i32 {
    match p {
        Piece::Knight | Piece::Bishop => 1,
        Piece::Rook => 2,
        Piece::Queen => 4,
        Piece::Pawn | Piece::King => 0,
    }
}

pub fn pst_index(sq: Square, color: Color) -> usize {
    let i = sq.rank() as usize * 8 + sq.file() as usize;
    if color == Color::White { i } else { i ^ 56 }
}

pub fn phase(b: &Board) -> i32 {
    let mut ph = 0;
    for &p in &Piece::ALL {
        ph += phase_weight(p) * b.pieces(p).len() as i32;
    }
    ph.min(PHASE_TOTAL)
}

fn white_score(b: &Board) -> i32 {
    let mut mg = 0;
    let mut eg = 0;
    for &c in &Color::ALL {
        let sign = if c == Color::White { 1 } else { -1 };
        for &p in &Piece::ALL {
            for sq in b.pieces(p) & b.colors(c) {
                let i = pst_index(sq, c);
                let v = piece_value(p);
                mg += sign * (v + MG_PST[p as usize][i]);
                eg += sign * (v + EG_PST[p as usize][i]);
            }
        }
    }
    let ph = phase(b);
    (mg * ph + eg * (PHASE_TOTAL - ph)) / PHASE_TOTAL
}

pub fn evaluate(b: &Board) -> i32 {
    let s = white_score(b);
    if b.side_to_move() == Color::White { s } else { -s }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn board(fen: &str) -> Board {
        fen.parse().unwrap()
    }

    #[test]
    fn phase_matches_material_on_board() {
        assert_eq!(phase(&Board::default()), 24);
        assert_eq!(phase(&board("8/8/8/8/8/8/8/K6k w - - 0 1")), 0);
        assert_eq!(phase(&board("4k3/8/8/8/8/8/8/3QK3 w - - 0 1")), 4);
        assert_eq!(phase(&board("r3k3/8/8/8/8/8/8/1N2K3 w - - 0 1")), 3);
    }

    #[test]
    fn pst_index_mirrors_black_vertically() {
        assert_eq!(pst_index(Square::A1, Color::White), 0);
        assert_eq!(pst_index(Square::E1, Color::White), 4);
        assert_eq!(pst_index(Square::H8, Color::White), 63);
        assert_eq!(pst_index(Square::A8, Color::Black), 0);
        assert_eq!(pst_index(Square::E8, Color::Black), 4);
        assert_eq!(pst_index(Square::H1, Color::Black), 63);
    }

    #[test]
    fn evaluation_is_color_symmetric() {
        assert_eq!(evaluate(&Board::default()), 0);
        let a = board("4k3/8/8/3n4/8/8/4P3/4K3 w - - 0 1");
        let b = board("4k3/4p3/8/8/3N4/8/8/4K3 b - - 0 1");
        assert_eq!(evaluate(&a), -220);
        assert_eq!(evaluate(&a), evaluate(&b));
    }
}
