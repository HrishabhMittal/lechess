mod board;
mod eval;
use board::*;
use std::time::Instant;

use rand::{prelude::*, rng};

use crate::eval::Stockfish;
fn perft(board: &Board, depth: u8) -> u64 {
    if depth == 0 {
        return 1;
    }
    let moves = board.generate_legal_moves();
    let mut nodes = 0;
    for m in moves {
        let next_board = board.make_move(&m);
        nodes += perft(&next_board, depth - 1);
    }
    nodes
}
fn main() {
    let mut board = Board::new();
    // let max_depth = 6;
    // for depth in 1..=max_depth {
    //     let start = Instant::now();
    //     let nodes = perft(&board, depth);
    //     let duration = start.elapsed();
    //     println!("depth: {}, nodes: {}, time: {:?}", depth, nodes, duration);
    // }
    let mut r = rng();
    for _ in 1..10 {
        let moves = board.generate_legal_moves();
        let mov = match moves.choose(&mut r) {
            Some(v) => v,
            None => panic!("no move found. prob check/stalemate"),
        };
        board = board.make_move(mov);
    }
    let fen = board.to_fen();
    let mut sf = Stockfish::new(None);
    let start = Instant::now();
    let eval = sf.get_eval(&fen, 22);
    let duration = start.elapsed();
    println!("evaluated {} to {} in {:?}", fen, eval, duration);
}
