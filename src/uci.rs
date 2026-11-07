use std::io::{self, BufRead};
use std::time::Duration;

use cozy_chess::{util, Board, Color};

use crate::ordering::legal_moves;
use crate::search::Searcher;

const ENGINE_NAME: &str = "Scipion";
const ENGINE_AUTHOR: &str = "pdro-h";

/// Número de entradas, não bytes.
const TT_SIZE: usize = 1 << 22;

const MOVE_OVERHEAD_MS: u64 = 30;

const MAX_DEPTH: i32 = 64;

const NO_TIME_LIMIT: Duration = Duration::from_secs(3600);

#[derive(Default)]
struct GoParams {
    wtime: Option<u64>,
    btime: Option<u64>,
    winc: Option<u64>,
    binc: Option<u64>,
    movetime: Option<u64>,
    depth: Option<i32>,
}

fn next_ms<'a>(it: &mut impl Iterator<Item = &'a str>) -> Option<u64> {
    it.next()?.parse::<i64>().ok().map(|v| v.max(0) as u64)
}

fn parse_go(args: &[&str]) -> GoParams {
    let mut p = GoParams::default();
    let mut it = args.iter().copied();
    while let Some(tok) = it.next() {
        match tok {
            "wtime" => p.wtime = next_ms(&mut it),
            "btime" => p.btime = next_ms(&mut it),
            "winc" => p.winc = next_ms(&mut it),
            "binc" => p.binc = next_ms(&mut it),
            "movetime" => p.movetime = next_ms(&mut it),
            "depth" => p.depth = it.next().and_then(|v| v.parse().ok()),
            _ => {}
        }
    }
    p
}

fn time_budget(board: &Board, p: &GoParams) -> Duration {
    if let Some(mt) = p.movetime {
        let ms = mt.saturating_sub(MOVE_OVERHEAD_MS).max(1);
        return Duration::from_millis(ms);
    }

    let (time, inc) = match board.side_to_move() {
        Color::White => (p.wtime, p.winc.unwrap_or(0)),
        Color::Black => (p.btime, p.binc.unwrap_or(0)),
    };

    let Some(time) = time else { return NO_TIME_LIMIT };

    let budget = time / 30 + inc / 2;
    let hard_cap = time.saturating_sub(MOVE_OVERHEAD_MS);
    Duration::from_millis(budget.min(hard_cap).max(1))
}

fn parse_position(args: &[&str]) -> Option<(Board, Vec<u64>)> {
    let (setup, moves) = match args.iter().position(|&t| t == "moves") {
        Some(i) => (&args[..i], &args[i + 1..]),
        None => (args, &args[args.len()..]),
    };

    let mut board = match *setup.first()? {
        "startpos" => Board::default(),
        "fen" => Board::from_fen(&setup[1..].join(" "), false).ok()?,
        _ => return None,
    };

    let mut history = Vec::new();

    for m in moves {
        let mv = util::parse_uci_move(&board, m).ok()?;
        if !board.is_legal(mv) {
            return None;
        }
        // O hash entra antes do lance: a posição atual não pode constar no histórico.
        history.push(board.hash());
        board.play_unchecked(mv);
    }
    Some((board, history))
}

pub fn run() {
    let mut board = Board::default();
    let mut history: Vec<u64> = Vec::new();
    let mut searcher = Searcher::new(TT_SIZE);

    for line in io::stdin().lock().lines() {
        let Ok(line) = line else { break };
        let tokens: Vec<&str> = line.split_whitespace().collect();
        let Some((&cmd, args)) = tokens.split_first() else { continue };

        match cmd {
            "uci" => {
                println!("id name {ENGINE_NAME}");
                println!("id author {ENGINE_AUTHOR}");
                println!("uciok");
            }
            "isready" => println!("readyok"),
            "ucinewgame" => {
                board = Board::default();
                history.clear();
                searcher = Searcher::new(TT_SIZE);
            }
            "position" => {
                if let Some((b, h)) = parse_position(args) {
                    board = b;
                    history = h;
                }
            }
            "go" => {
                if legal_moves(&board).is_empty() {
                    println!("bestmove 0000");
                    continue;
                }
                let p = parse_go(args);
                let depth = p.depth.unwrap_or(MAX_DEPTH);
                let limit = time_budget(&board, &p);
                let mv = searcher.think(&board, &history, depth, limit);
                println!("bestmove {}", util::display_uci_move(&board, mv));
            }
            "quit" => break,
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn castling_is_printed_in_standard_uci() {
        let b = Board::from_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1", false).unwrap();
        let out: Vec<String> = legal_moves(&b)
            .iter()
            .map(|m| util::display_uci_move(&b, *m).to_string())
            .collect();
        assert!(out.contains(&"e1g1".to_string()));
        assert!(out.contains(&"e1c1".to_string()));
        assert!(!out.contains(&"e1h1".to_string()));
        assert!(!out.contains(&"e1a1".to_string()));
    }
}
