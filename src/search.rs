use crate::eval::evaluate;
use crate::ordering::{legal_moves, order};
use crate::tt::{Tt, EXACT, LOWER, UPPER};
use cozy_chess::*;
use std::time::{Duration, Instant};

pub const INF: i32 = 1_000_000;
pub const MATE: i32 = 100_000;
const MATE_BOUND: i32 = MATE - 1000;

pub struct Searcher {
    tt: Tt,
    start: Instant,
    limit: Duration,
    nodes: u64,
    stop: bool,
    iter_best: Option<Move>,
    path: Vec<u64>,
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
            path: Vec::new(),
        }
    }

    fn time_up(&mut self) -> bool {
        if self.nodes & 2047 == 0 && self.start.elapsed() >= self.limit {
            self.stop = true;
        }
        self.stop
    }

    /// Pré-condição: o último elemento de `path` é o hash de `b`.
    fn is_draw(&self, b: &Board) -> bool {
        let hm = b.halfmove_clock() as usize;
        if hm >= 100 {
            return true;
        }
        let n = self.path.len();
        let start = n.saturating_sub(1 + hm);
        self.path[start..n - 1].contains(&b.hash())
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
        if ply > 0 && self.is_draw(b) {
            return 0;
        }
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
            self.path.push(nb.hash());
            let s = -self.negamax(&nb, depth - 1, ply + 1, -beta, -alpha);
            self.path.pop();
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

    pub fn think(&mut self, b: &Board, history: &[u64], max_depth: i32, limit: Duration) -> Move {
        self.start = Instant::now();
        self.limit = limit;
        self.stop = false;
        self.nodes = 0;
        self.path.clear();
        self.path.extend_from_slice(history);
        self.path.push(b.hash());
        let mut best = legal_moves(b)[0];
        for d in 1..=max_depth {
            self.iter_best = None;
            let s = self.negamax(b, d, 0, -INF, INF);
            if self.stop {
                break;
            }
            if let Some(m) = self.iter_best {
                best = m;
            }
            let score_str = if s > MATE_BOUND {
                format!("mate {}", (MATE - s + 1) / 2)
            } else if s < -MATE_BOUND {
                format!("mate -{}", (MATE + s + 1) / 2)
            } else {
                format!("cp {}", s)
            };
            println!(
                "info depth {} score {} nodes {} time {} pv {}",
                d,
                score_str,
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

fn score_to_tt(score: i32, ply: i32) -> i32 {
    if score > MATE_BOUND {
        score + ply
    } else if score < -MATE_BOUND {
        score - ply
    } else {
        score
    }
}

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
        let stored = score_to_tt(MATE - 5, 3);
        assert_eq!(stored, MATE - 2);

        assert_eq!(score_from_tt(stored, 1), MATE - 3);
        assert_eq!(score_from_tt(stored, 3), MATE - 5);

        let stored = score_to_tt(-MATE + 5, 3);
        assert_eq!(stored, -MATE + 2);
        assert_eq!(score_from_tt(stored, 1), -MATE + 3);

        assert_eq!(score_to_tt(150, 7), 150);
        assert_eq!(score_from_tt(-40, 7), -40);
    }

    #[test]
    fn repetition_and_fifty_move_are_draws() {
        let mut s = Searcher::new(1 << 10);

        let b = Board::from_fen("4k3/8/8/8/8/8/8/R3K3 w - - 100 80", false).unwrap();
        s.path = vec![b.hash()];
        assert!(s.is_draw(&b));

        let b = Board::default();
        s.path = vec![b.hash()];
        assert!(!s.is_draw(&b));

        let mut b = Board::default();
        s.path = vec![b.hash()];
        for (i, m) in ["g1f3", "g8f6", "f3g1", "f6g8"].iter().enumerate() {
            let mv = util::parse_uci_move(&b, m).unwrap();
            b.play_unchecked(mv);
            s.path.push(b.hash());
            assert_eq!(s.is_draw(&b), i == 3, "falhou após o lance {}", m);
        }
    }

    #[test]
    fn mate_score_is_read_from_tt_at_a_different_ply() {
        let p0 = Board::from_fen("4k3/8/8/8/8/8/R7/1R4K1 w - - 0 1", false).unwrap();
        let mut q = p0.clone();
        q.play_unchecked(util::parse_uci_move(&q, "a2a7").unwrap());

        let mut warm = Searcher::new(1 << 16);
        for reply in ["e8d8", "e8f8"] {
            let mut p1 = q.clone();
            p1.play_unchecked(util::parse_uci_move(&p1, reply).unwrap());
            let mate_mv = util::parse_uci_move(&p1, "b1b8").unwrap();
            warm.tt.store(p1.hash(), 10, MATE - 1, EXACT, Some(mate_mv));
        }
        warm.path = vec![q.hash()];
        let s_warm = warm.negamax(&q, 4, 0, -INF, INF);

        let mut cold = Searcher::new(1 << 16);
        cold.path = vec![q.hash()];
        let s_cold = cold.negamax(&q, 4, 0, -INF, INF);

        assert!(warm.nodes < cold.nodes, "a TT não foi usada");
        assert_eq!(s_cold, -MATE + 2);
        assert_eq!(s_warm, s_cold);
    }
}
