use std::{fs::OpenOptions, io::Write, time::Instant};

use crate::{board::Board, nn::NeuralNet, tt::TranspositionTable};

mod best_move;
mod board;
mod gen_dataset;
mod move_list;
mod nn;
mod stockfish;
mod tt;

fn main() {
    let engine_nn = NeuralNet::load("models/weights.json");
    let mut board = Board::new();
    let mut tt_table = TranspositionTable::new(256);
    println!("\x1B[H\x1B[2J{}", board);
    let mut line = String::new();
    let search_depth = 10;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("save.txt")
        .expect("Failed to open save.txt");
    loop {
        let mut total = 0;
        let start = Instant::now();
        match best_move::find_best_move(
            &mut board,
            &engine_nn,
            search_depth,
            &mut tt_table,
            &mut total,
        ) {
            Some(best_move) => {
                let san = board.move_to_san(&best_move);

                if board.white_to_move {
                    write!(file, "{}. {} ", board.fullmove_number, san).unwrap();
                } else {
                    write!(file, "{} ", san).unwrap();
                }

                let _ = board.make_move(&best_move);
                print!("\x1B[H\x1B[2J");
                println!("{}", board);
                line.clear();
            }
            None => {
                if board.is_in_check(board.white_to_move) {
                    println!("checkmate");
                    writeln!(file, "{}", if board.white_to_move { "0-1" } else { "1-0" }).unwrap();
                } else {
                    println!("stalemate");
                    writeln!(file, "1/2-1/2").unwrap();
                }
                break;
            }
        }
        println!("nodes touched: {}, in time: {:?}", total, start.elapsed());
        // let mut line = String::new();
        // io::stdin().read_line(&mut line).unwrap();
    }
}
