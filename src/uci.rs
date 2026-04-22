use crate::best_move;
use crate::board::{Board, Move, MoveFlag};
use crate::nn::NeuralNet;
use crate::tt::TranspositionTable;
use std::io::{self, BufRead, Write};
fn move_to_uci(m: &Move) -> String {
    let from_file = (m.from % 8) as u8;
    let from_rank = (m.from / 8) as u8;
    let to_file = (m.to % 8) as u8;
    let to_rank = (m.to / 8) as u8;
    let from_str = format!(
        "{}{}",
        (b'a' + from_file) as char,
        (b'1' + from_rank) as char
    );
    let to_str = format!("{}{}", (b'a' + to_file) as char, (b'1' + to_rank) as char);
    let promo = match m.flag {
        MoveFlag::PromoQueen | MoveFlag::PromoCaptureQueen => "q",
        MoveFlag::PromoRook | MoveFlag::PromoCaptureRook => "r",
        MoveFlag::PromoBishop | MoveFlag::PromoCaptureBishop => "b",
        MoveFlag::PromoKnight | MoveFlag::PromoCaptureKnight => "n",
        _ => "",
    };
    format!("{}{}{}", from_str, to_str, promo)
}
fn parse_move(board: &mut Board, move_str: &str) -> Option<Move> {
    let bytes = move_str.as_bytes();
    if bytes.len() < 4 {
        return None;
    }
    let from_sq = (bytes[1] - b'1') * 8 + (bytes[0] - b'a');
    let to_sq = (bytes[3] - b'1') * 8 + (bytes[2] - b'a');
    let promo_char = if bytes.len() == 5 { bytes[4] } else { 0 };
    let legal_moves = board.generate_legal_moves();
    for m in legal_moves.iter() {
        if m.from == from_sq && m.to == to_sq {
            if promo_char != 0 {
                let matches_promo = match promo_char {
                    b'q' => matches!(m.flag, MoveFlag::PromoQueen | MoveFlag::PromoCaptureQueen),
                    b'r' => matches!(m.flag, MoveFlag::PromoRook | MoveFlag::PromoCaptureRook),
                    b'b' => matches!(m.flag, MoveFlag::PromoBishop | MoveFlag::PromoCaptureBishop),
                    b'n' => matches!(m.flag, MoveFlag::PromoKnight | MoveFlag::PromoCaptureKnight),
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
fn parse_position(board: &mut Board, tokens: &[&str]) {
    if tokens.len() < 2 {
        return;
    }
    let mut move_idx = 0;
    if tokens[1] == "startpos" {
        *board = Board::new();
        move_idx = 2;
    } else if tokens[1] == "fen" {
        let mut fen = String::new();
        for i in 2..tokens.len() {
            if tokens[i] == "moves" {
                move_idx = i;
                break;
            }
            fen.push_str(tokens[i]);
            fen.push(' ');
        }
        if let Ok(b) = Board::from_fen(fen.trim()) {
            *board = b;
        }
        if move_idx == 0 {
            move_idx = tokens.len();
        }
    }
    if move_idx < tokens.len() && tokens[move_idx] == "moves" {
        for i in (move_idx + 1)..tokens.len() {
            if let Some(m) = parse_move(board, tokens[i]) {
                let _ = board.make_move(&m);
            }
        }
    }
}
pub fn uci(weights_path: &str) {
    let engine_nn = NeuralNet::load(weights_path);
    let mut tt_table = TranspositionTable::new(256);
    let mut board = Board::new();
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let tokens: Vec<&str> = line.split_whitespace().collect();
        if tokens.is_empty() {
            continue;
        }
        match tokens[0] {
            "uci" => {
                println!("id name LeChess");
                println!("id author Hrishabh");
                println!("uciok");
            }
            "isready" => {
                println!("readyok");
            }
            "position" => {
                parse_position(&mut board, &tokens);
            }
            "go" => {
                let mut wtime = 0;
                let mut btime = 0;
                let mut winc = 0;
                let mut binc = 0;
                let mut depth_limit = 100;

                let mut i = 1;
                while i < tokens.len() {
                    match tokens[i] {
                        "wtime" => wtime = tokens[i + 1].parse().unwrap_or(0),
                        "btime" => btime = tokens[i + 1].parse().unwrap_or(0),
                        "winc" => winc = tokens[i + 1].parse().unwrap_or(0),
                        "binc" => binc = tokens[i + 1].parse().unwrap_or(0),
                        "depth" => depth_limit = tokens[i + 1].parse().unwrap_or(100),
                        _ => {}
                    }
                    i += 1;
                }

                let mut time_limit = None;
                if wtime > 0 || btime > 0 {
                    let my_time = if board.white_to_move { wtime } else { btime };
                    let my_inc = if board.white_to_move { winc } else { binc };

                    time_limit = Some((my_time / 30) + (my_inc / 2));
                }

                let mut total = 0;
                if let Some(best_move) = best_move::find_best_move(
                    &mut board,
                    &engine_nn,
                    depth_limit,
                    &mut tt_table,
                    &mut total,
                    true,
                    time_limit,
                ) {
                    println!("bestmove {}", move_to_uci(&best_move));
                } else {
                    println!("bestmove 0000");
                }
            }
            "quit" => break,
            _ => {}
        }
        io::stdout().flush().unwrap();
    }
}
