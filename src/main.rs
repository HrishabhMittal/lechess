use std::{io, time::Instant};

use crate::{board::Board, nn::NeuralNet, tt::TranspositionTable};

mod best_move;
mod board;
mod eval;
mod gen_dataset;
mod nn;
mod tt;
fn main() {
    let engine_nn = NeuralNet::load("models/weights.json");
    let mut board = Board::new();
    let mut tt_table = TranspositionTable::new(64);
    println!("\x1B[H\x1B[2J{}", board);
    let mut line = String::new();
    let search_depth = 8;
    loop {
        let mut total = 0;
        let start = Instant::now();
        match best_move::find_best_move(&mut board, &engine_nn, search_depth, &mut tt_table, &mut total)
        {
            Some(best_move) => {
                let _ = board.make_move(&best_move);
                print!("\x1B[H\x1B[2J");
                println!("{}", board);
                line.clear();
            }
            None => {
                if board.is_in_check(board.white_to_move) {
                    println!("checkmate");
                } else {
                    println!("stalemate");
                }
                break;
            }
        }
        println!("nodes touched: {}, in time: {:?}", total, start.elapsed());
        let mut line = String::new();
        io::stdin().read_line(&mut line).unwrap();
    }
}
