mod eval;
mod ordering;

use cozy_chess::Board;

fn main() {
    // 1) contagem de lances legais (valores conhecidos)
    let start = Board::default();
    println!(
        "posição inicial: {} lances (esperado 20)",
        ordering::legal_moves(&start).len()
    );

    let kiwipete = Board::from_fen(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        false,
    )
    .unwrap();
    println!(
        "kiwipete: {} lances (esperado 48)",
        ordering::legal_moves(&kiwipete).len()
    );

    // 2) ordenação: depois de 1.e4 d5, a captura exd5 deve vir primeiro
    let b = Board::from_fen(
        "rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2",
        false,
    )
    .unwrap();
    let mut moves = ordering::legal_moves(&b);
    ordering::order(&b, &mut moves, None);
    println!("primeiro lance ordenado: {} (esperado e4d5)", moves[0]);
}
