mod eval;

use cozy_chess::Board;

fn main() {
    let start = Board::default();
    println!("posição inicial: {}", eval::evaluate(&start));

    // brancas sem a torre de h1
    let sem_torre = Board::from_fen(
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBN1 w Qkq - 0 1",
        false,
    )
    .unwrap();
    println!("brancas sem uma torre: {}", eval::evaluate(&sem_torre));
}
