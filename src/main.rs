mod eval;
mod ordering;
mod search;
mod tt;

use cozy_chess::Board;
use search::Searcher;
use std::time::Duration;

fn testa(nome: &str, fen: &str, esperado: &str) {
    println!("--- {} ---", nome);
    let b = Board::from_fen(fen, false).unwrap();
    let mut s = Searcher::new(1 << 20);
    let mv = s.think(&b, 6, Duration::from_secs(2));
    println!("escolheu: {} (esperado {})\n", mv, esperado);
}

fn main() {
    // mate de corredor: Ta8#
    testa("mate em 1", "6k1/5ppp/8/8/8/8/8/R5K1 w - - 0 1", "a1a8");

    // dama preta pendurada na coluna d: Dxd5
    testa("dama pendurada", "4k3/8/8/3q4/8/8/8/3QK3 w - - 0 1", "d1d5");
}
