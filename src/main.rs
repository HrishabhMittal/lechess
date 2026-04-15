use crate::{
    board::{Board, Move},
    nn::NeuralNet,
};
mod board;
mod eval;
mod gen_dataset;
mod nn;
fn evaluate_position(board: &Board, engine_nn: &NeuralNet, depth: u32, mut alpha: f32, beta: f32) -> f32 {
    let moves = board.generate_legal_moves();
    if moves.is_empty() {
        if board.is_in_check(board.white_to_move) {
            return -10000.0 - (depth as f32);
        } else {
            return 0.0;
        }
    }
    if depth == 0 {
        return engine_nn.evaluate(&board.to_features());
    }
    let mut scored_moves: Vec<(Move, f32)> = moves
        .into_iter()
        .map(|m| {
            let next_board = board.make_move(&m);
            let score = -engine_nn.evaluate(&next_board.to_features());
            (m, score)
        })
        .collect();
    scored_moves.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let mut max_val = f32::NEG_INFINITY;
    for (m, _) in scored_moves {
        let next_board = board.make_move(&m);
        let score = -evaluate_position(&next_board, engine_nn, depth - 1, -beta, -alpha);
        if score > max_val {
            max_val = score;
        }
        if alpha < score {
            alpha = score;
        }
        if alpha >= beta {
            break;
        }
    }
    max_val
}
fn find_best_move(board: &Board, engine_nn: &NeuralNet, depth: u32) -> Option<Move> {
    let moves = board.generate_legal_moves();
    if moves.is_empty() {
        return None;
    }
    let mut best_move = moves[0];
    let mut max_val = f32::NEG_INFINITY;
    let mut alpha = f32::NEG_INFINITY;
    let beta = f32::INFINITY;
    let mut scored_moves: Vec<(Move, f32)> = moves
        .into_iter()
        .map(|m| {
            let next_board = board.make_move(&m);
            let score = -engine_nn.evaluate(&next_board.to_features());
            (m, score)
        })
        .collect();
    scored_moves.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    for (m, _) in scored_moves {
        let next_board = board.make_move(&m);
        let score = -evaluate_position(&next_board, engine_nn, depth - 1, -beta, -alpha);
        if score > max_val {
            max_val = score;
            best_move = m;
        }
        if alpha < score {
            alpha = score;
        }
    }
    Some(best_move)
}
fn main() {
    let engine_nn = NeuralNet::load("models/weights.json");
    let mut board = Board::new();

    println!("\x1B[H\x1B[2J{}", board);
    let mut line = String::new();
    std::io::stdin().read_line(&mut line).expect("io error");
    let search_depth = 4;
    for half_move in 1..=100 {
        match find_best_move(&board, &engine_nn, search_depth) {
            Some(best_move) => {
                board = board.make_move(&best_move);
                let player = if board.white_to_move {
                    "Black"
                } else {
                    "White"
                };
                print!("\x1B[H\x1B[2J");
                println!("--- Half-move {}: {} played ---", half_move, player);
                println!("{}", board);
                line.clear();
                std::io::stdin().read_line(&mut line).expect("io error");
            }
            None => {
                if board.is_in_check(board.white_to_move) {
                    println!("Checkmate!");
                } else {
                    println!("Stalemate!");
                }
                break;
            }
        }
    }
}
