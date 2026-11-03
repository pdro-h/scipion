mod eval;
mod ordering;
mod tt;

use cozy_chess::Board;
use tt::{Tt, EXACT, LOWER};

fn main() {
    let mut tt = Tt::new(1024);

    // um lance qualquer da posição inicial, só para guardar na tabela
    let mv = ordering::legal_moves(&Board::default())[0];

    // 1) tabela vazia: nada deve ser encontrado
    println!("vazia, chave 12345: {}", tt.probe(12345).is_none());

    // 2) guardar e consultar
    tt.store(5, 7, 150, EXACT, Some(mv));
    let e = tt.probe(5).expect("entrada deveria existir");
    println!(
        "guardado: depth={} score={} flag={} mv={} (esperado 7 150 0 {})",
        e.depth, e.score, e.flag, e.mv.unwrap(), mv
    );

    // 3) colisão: mesma casa (5 e 1029), chave diferente -> não pode devolver a entrada errada
    println!("chave 1029 (colide com 5): {}", tt.probe(1029).is_none());

    // 4) sobrescrita: guardar 1029 apaga a entrada 5
    tt.store(1029, 3, -40, LOWER, None);
    println!("após guardar 1029, chave 5 sumiu: {}", tt.probe(5).is_none());
    println!(
        "chave 1029 existe com score={} (esperado -40): {}",
        tt.probe(1029).unwrap().score,
        tt.probe(1029).is_some()
    );
}
