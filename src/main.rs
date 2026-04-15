use crate::{
    board::Board,
    nn::NeuralNet,
};

mod board;
mod eval;
mod gen_dataset;
mod nn;
mod best_move;

fn main() {
    let engine_nn = NeuralNet::load("models/weights.json");
    let mut board = Board::new();

    println!("\x1B[H\x1B[2J{}", board);
    let mut line = String::new();
    let search_depth = 4;
    for _ in 1..=100 {
        match best_move::find_best_move(&board, &engine_nn, search_depth) {
            Some(best_move) => {
                board = board.make_move(&best_move);
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
    }
}
