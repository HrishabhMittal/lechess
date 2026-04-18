use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use crate::board::{Board, Move, MoveFlag};
pub struct Stockfish {
    process: Child,
    stdin: ChildStdin,
    reader: BufReader<ChildStdout>,
}
impl Stockfish {
    pub fn new(path: Option<&str>) -> Self {
        let path = match path {
            Some(v) => v,
            None => "/usr/bin/stockfish",
        };
        let mut process = Command::new(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("failed to start stockfish, couldn't launch /usr/bin/stockfish.");
        let mut stdin = process.stdin.take().expect("failed to take stdin");
        let stdout = process.stdout.take().expect("failed to take stdout");
        let mut reader = BufReader::new(stdout);
        writeln!(stdin, "uci").unwrap();
        Self::wait_for(&mut reader, "uciok");
        writeln!(stdin, "setoption name Threads value 1").unwrap();
        writeln!(stdin, "isready").unwrap();
        Self::wait_for(&mut reader, "readyok");
        Stockfish {
            process,
            stdin,
            reader,
        }
    }
    pub fn get_eval(&mut self, fen: &str, depth: u8) -> i32 {
        writeln!(self.stdin, "position fen {}", fen).unwrap();
        writeln!(self.stdin, "go depth {}", depth).unwrap();
        let mut final_score = 0;
        let mut line = String::new();
        loop {
            line.clear();
            self.reader.read_line(&mut line).unwrap();
            let text = line.trim();
            if text.starts_with("info") {
                let tokens: Vec<&str> = text.split_whitespace().collect();
                if let Some(score_idx) = tokens.iter().position(|&x| x == "score") {
                    if tokens.len() > score_idx + 2 {
                        let score_type = tokens[score_idx + 1];
                        let score_val = tokens[score_idx + 2].parse::<i32>().unwrap_or(0);
                        if score_type == "cp" {
                            final_score = score_val;
                        } else if score_type == "mate" {
                            final_score = if score_val > 0 { 8500 } else { -8500 };
                        }
                    }
                }
            } else if text.starts_with("bestmove") {
                break;
            }
        }
        let is_black_turn = fen.split_whitespace().nth(1).unwrap_or("w") == "b";
        if is_black_turn {
            -final_score
        } else {
            final_score
        }
    }
    pub fn get_best_move(&mut self, board: &mut Board, depth: u8) -> Option<Move> {
        let fen = board.to_fen();
        writeln!(self.stdin, "position fen {}", fen).unwrap();
        writeln!(self.stdin, "go depth {}", depth).unwrap();

        let mut bestmove_str = String::new();
        let mut line = String::new();
        loop {
            line.clear();
            self.reader.read_line(&mut line).unwrap();
            let text = line.trim();

            if text.starts_with("bestmove") {
                let tokens: Vec<&str> = text.split_whitespace().collect();
                if tokens.len() > 1 {
                    bestmove_str = tokens[1].to_string();
                }
                break;
            }
        }

        if bestmove_str.is_empty() || bestmove_str == "(none)" {
            return None;
        }

        let bytes = bestmove_str.as_bytes();
        if bytes.len() < 4 {
            return None;
        }

        let from_file = bytes[0] - b'a';
        let from_rank = bytes[1] - b'1';
        let from_sq = from_rank * 8 + from_file;

        let to_file = bytes[2] - b'a';
        let to_rank = bytes[3] - b'1';
        let to_sq = to_rank * 8 + to_file;

        let is_promo = bytes.len() == 5;
        let promo_char = if is_promo { bytes[4] } else { 0 };

        let legal_moves = board.generate_legal_moves();
        for m in legal_moves.iter() {
            if m.from == from_sq && m.to == to_sq {
                if is_promo {
                    let matches_promo = match promo_char {
                        b'q' => {
                            matches!(m.flag, MoveFlag::PromoQueen | MoveFlag::PromoCaptureQueen)
                        }
                        b'r' => matches!(m.flag, MoveFlag::PromoRook | MoveFlag::PromoCaptureRook),
                        b'b' => {
                            matches!(m.flag, MoveFlag::PromoBishop | MoveFlag::PromoCaptureBishop)
                        }
                        b'n' => {
                            matches!(m.flag, MoveFlag::PromoKnight | MoveFlag::PromoCaptureKnight)
                        }
                        _ => false,
                    };

                    if matches_promo {
                        return Some(*m);
                    }
                } else {
                    return Some(*m);
                }
            }
        }

        None
    }
    fn wait_for(reader: &mut BufReader<ChildStdout>, target: &str) {
        let mut line = String::new();
        loop {
            line.clear();
            reader.read_line(&mut line).unwrap();
            if line.contains(target) {
                break;
            }
        }
    }
}
impl Drop for Stockfish {
    fn drop(&mut self) {
        let _ = writeln!(self.stdin, "quit");
        let _ = self.process.wait();
    }
}
