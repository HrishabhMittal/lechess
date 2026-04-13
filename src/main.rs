mod board;
use board::*;
use std::time::Instant;
fn perft(board: &Board, depth: u8, is_white_turn: bool) -> u64 {
    if depth == 0 {
        return 1;
    }
    let moves = board.generate_legal_moves(is_white_turn);
    let mut nodes = 0;
    for m in moves {
        let next_board = board.make_move(&m, is_white_turn);
        nodes += perft(&next_board, depth - 1, !is_white_turn);
    }
    nodes
}
fn main() {
    let board = Board::new();
    let max_depth = 10; 
    for depth in 1..=max_depth {
        let start = Instant::now();
        let nodes = perft(&board, depth, true);
        let duration = start.elapsed();
        println!(
            "depth: {}, nodes: {}, time: {:?}",
            depth, nodes, duration
        );
    }
}
