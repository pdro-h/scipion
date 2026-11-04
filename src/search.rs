use crate::eval::evaluate;
use crate::ordering::{legal_moves, order};
use crate::tt::{Tt, EXACT, LOWER, UPPER};
use cozy_chess::*;
use std::time::{Duration, Instant};

pub const INF: i32 = 1_000_000;
pub const MATE: i32 = 100_000;

pub struct Searcher {
    tt: Tt,
    start: Instant,
    limit: Duration,
    nodes: u64,
    stop: bool,
    iter_best: Option<Move>,
}

impl Searcher {
    pub fn new(tt_size: usize) -> Self {
        Self {
            tt: Tt::new(tt_size),
            start: Instant::now(),
            limit: Duration::from_secs(1),
            nodes: 0,
            stop: false,
            iter_best: None,
        }
    }

    fn time_up(&mut self) -> bool {
        if self.nodes & 2047 == 0 && self.start.elapsed() >= self.limit {
            self.stop = true;
        }
        self.stop
    }

    fn qsearch(&mut self, b: &Board, mut alpha: i32, beta: i32) -> i32 {
        self.nodes += 1;
        if self.time_up() {
            return 0;
        }
        let stand = evaluate(b);
        if stand >= beta {
            return stand;
        }
        if stand > alpha {
            alpha = stand;
        }
        let them = b.colors(!b.side_to_move());
        let mut moves = legal_moves(b);
        moves.retain(|m| them.has(m.to));
        order(b, &mut moves, None);
        for m in moves {
            let mut nb = b.clone();
            nb.play_unchecked(m);
            let s = -self.qsearch(&nb, -beta, -alpha);
            if self.stop {
                return 0;
            }
            if s >= beta {
                return s;
            }
            if s > alpha {
                alpha = s;
            }
        }
        alpha
    }

    fn negamax(&mut self, b: &Board, depth: i32, ply: i32, mut alpha: i32, beta: i32) -> i32 {
        self.nodes += 1;
        if self.time_up() {
            return 0;
        }
        // extensão de xeque
        let depth = if b.checkers().is_empty() { depth } else { depth + 1 };
        if depth <= 0 {
            return self.qsearch(b, alpha, beta);
        }

        let key = b.hash();
        let mut tt_move = None;
        if let Some(e) = self.tt.probe(key) {
            tt_move = e.mv;
            if ply > 0 && e.depth as i32 >= depth {
                let score = score_from_tt(e.score, ply);
                match e.flag {
                    EXACT => return score,
                    LOWER if score >= beta => return score,
                    UPPER if score <= alpha => return score,
                    _ => {}
                }
            }
        }
        

        let mut moves = legal_moves(b);
        if moves.is_empty() {
            return if b.checkers().is_empty() { 0 } else { -MATE + ply };
        }
        order(b, &mut moves, tt_move);

        let orig_alpha = alpha;
        let mut best = -INF;
        let mut best_mv = None;
        for m in moves {
            let mut nb = b.clone();
            nb.play_unchecked(m);
            let s = -self.negamax(&nb, depth - 1, ply + 1, -beta, -alpha);
            if self.stop {
                return 0;
            }
            if s > best {
                best = s;
                best_mv = Some(m);
                if ply == 0 {
                    self.iter_best = Some(m);
                }
            }
            if s > alpha {
                alpha = s;
            }
            if alpha >= beta {
                break;
            }
        }

        let flag = if best <= orig_alpha {
            UPPER
        } else if best >= beta {
            LOWER
        } else {
            EXACT
        };
        self.tt.store(key, depth, score_to_tt(best, ply), flag, best_mv);
        best
    }

    pub fn think(&mut self, b: &Board, max_depth: i32, limit: Duration) -> Move {
        self.start = Instant::now();
        self.limit = limit;
        self.stop = false;
        self.nodes = 0;
        let mut best = legal_moves(b)[0];
        for d in 1..=max_depth {
            self.iter_best = None;
            let s = self.negamax(b, d, 0, -INF, INF);
            if self.stop {
                break; // descarta iteração incompleta
            }
            if let Some(m) = self.iter_best {
                best = m;
            }
            println!(
                "info depth {} score cp {} nodes {} time {} pv {}",
                d,
                s,
                self.nodes,
                self.start.elapsed().as_millis(),
                util::display_uci_move(b, best)
            );
            if s.abs() > MATE - 1000 {
                break;
            }
        }
        best
    }
}

/// Notas de mate acima disso são tratadas como "mate".
const MATE_BOUND: i32 = MATE - 1000;

/// Raiz -> TT: guarda o mate relativo ao nó, não à raiz.
fn score_to_tt(score: i32, ply: i32) -> i32 {
    if score > MATE_BOUND {
        score + ply
    } else if score < -MATE_BOUND {
        score - ply
    } else {
        score
    }
}

/// TT -> raiz: converte o mate de volta para o ply atual.
fn score_from_tt(score: i32, ply: i32) -> i32 {
    if score > MATE_BOUND {
        score - ply
    } else if score < -MATE_BOUND {
        score + ply
    } else {
        score
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mate_score_adjusts_with_ply() {
        // Nó no ply 3 acha mate no ply 5: nota MATE - 5.
        let stored = score_to_tt(MATE - 5, 3);
        assert_eq!(stored, MATE - 2); // 2 jogadas a partir do nó

        // Lido em outro ply, a distância ao mate se ajusta.
        assert_eq!(score_from_tt(stored, 1), MATE - 3);
        assert_eq!(score_from_tt(stored, 3), MATE - 5);

        // Mate contra: mesmo raciocínio, sinal invertido.
        let stored = score_to_tt(-MATE + 5, 3);
        assert_eq!(stored, -MATE + 2);
        assert_eq!(score_from_tt(stored, 1), -MATE + 3);

        // Notas normais não mudam.
        assert_eq!(score_to_tt(150, 7), 150);
        assert_eq!(score_from_tt(-40, 7), -40);
    }
}
