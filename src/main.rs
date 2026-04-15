use crate::{board::Board, nn::NeuralNet};

mod board;
mod eval;
mod gen_dataset;
mod nn;

fn main() {
    println!("Loading Neural Network...");
    let engine_nn = NeuralNet::load("models/weights.json");
    println!("Brain loaded successfully!\n");

    let mut board = Board::new();
    println!("Starting Position:");
    println!("\x1B[H\x1B[2J{}", board);
    let mut line = String::new();
    std::io::stdin().read_line(&mut line).expect("io error");
    for _ in 1..=50 {
        let moves = board.generate_legal_moves();
        
        if moves.is_empty() {
            if board.is_in_check(board.white_to_move) {
                println!("checkmate");
            } else {
                println!("stalemate");
            }
            break;
        }

        let mut best_move = moves[0];
        let mut best_score = f32::NEG_INFINITY; 

        for m in &moves {
            let next_board = board.make_move(m);
            let opponent_eval = engine_nn.evaluate(&next_board.to_features());
            let score = -opponent_eval;

            if score > best_score {
                best_score = score;
                best_move = *m;
            }
        }
        board = board.make_move(&best_move);
        println!("\x1B[H\x1B[2J{}", board);
        std::io::stdin().read_line(&mut line).expect("io error");
    }
}
