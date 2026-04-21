use crate::best_move;
use crate::board::Board;
use crate::nn::NeuralNet;
use crate::tt::TranspositionTable;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::sync::{mpsc, Arc};
use std::thread;

pub fn gen_dataset(
    openings_path: Option<String>,
    num_cores: Option<usize>,
    games_total: usize,
    search_depth: u8,
    file_name: String,
    weights_path: String,
) {
    let mut openings = vec!["rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string()];
    if let Some(path) = openings_path {
        if let Ok(file) = File::open(path) {
            let reader = BufReader::new(file);
            openings = reader.lines().filter_map(Result::ok).collect();
        }
    }
    let shared_openings = Arc::new(openings);
    let num_threads =
        num_cores.unwrap_or_else(|| std::thread::available_parallelism().unwrap().get());
    let games_per_thread = games_total / num_threads;
    println!("using {} threads", num_threads);
    let (tx, rx) = mpsc::channel::<String>();
    let mut handles = vec![];
    for t_id in 0..num_threads {
        let tx_clone = tx.clone();
        let ops = Arc::clone(&shared_openings);
        let weights = weights_path.clone();
        let handle = thread::spawn(move || {
            let nn = NeuralNet::load(&weights);
            let mut tt = TranspositionTable::new(64);
            for g in 0..games_per_thread {
                let mut board = Board::new();
                let op_idx = (t_id * games_per_thread + g) % ops.len();
                if let Ok(b) = Board::from_fen(&ops[op_idx]) {
                    board = b;
                }
                let mut game_history = Vec::new();
                let mut move_count = 0;
                let mut result = "0.5";
                loop {
                    if move_count > 150 {
                        break;
                    }
                    let moves = board.generate_legal_moves();
                    if moves.count == 0 {
                        if board.is_in_check(board.white_to_move) {
                            result = if board.white_to_move { "0.0" } else { "1.0" };
                        }
                        break;
                    }
                    let mut total_nodes = 0;
                    if let Some(best_move) = best_move::find_best_move(
                        &mut board,
                        &nn,
                        search_depth as u32,
                        &mut tt,
                        &mut total_nodes,
                        false,
                    ) {
                        let mut score = 0;
                        if let Some(entry) = tt.probe(board.zobrist_hash) {
                            score = entry.score;
                        }
                        let white_score = if board.white_to_move { score } else { -score };
                        game_history.push((board.to_fen(), white_score));
                        if score > 9000 {
                            result = if board.white_to_move { "1.0" } else { "0.0" };
                            break;
                        } else if score < -9000 {
                            result = if board.white_to_move { "0.0" } else { "1.0" };
                            break;
                        }
                        let _ = board.make_move(&best_move);
                        move_count += 1;
                    } else {
                        break;
                    }
                }
                for (fen, score) in game_history {
                    let line = format!("{} | {} | {}\n", fen, score, result);
                    tx_clone.send(line).unwrap();
                }
            }
        });
        handles.push(handle);
    }
    drop(tx);
    let mut out_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_name)
        .unwrap();
    let mut written = 0;
    for line in rx {
        out_file.write_all(line.as_bytes()).unwrap();
        written += 1;
        if written % 10000 == 0 {
            println!("Saved {} positions", written);
        }
    }
    for handle in handles {
        handle.join().unwrap();
    }
}
