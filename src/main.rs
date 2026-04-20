use crate::{
    board::{Board, Move},
    gen_dataset::gen_dataset,
    nn::NeuralNet,
    stockfish::Stockfish,
    tt::TranspositionTable,
};
use clap::Parser;
use std::{
    fs::{File, OpenOptions},
    io::Write,
};

mod best_move;
mod board;
mod gen_dataset;
mod move_list;
mod nn;
mod stockfish;
mod tt;
mod uci;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = 10)]
    depth: u32,

    #[arg(long, short, default_value_t = false)]
    play_stockfish: bool,

    #[arg(long, short, default_value_t = false)]
    as_black: bool,

    #[arg(long, short, default_value_t = false)]
    self_play: bool,

    #[arg(long, short, default_value_t = false)]
    gen_dataset: bool,

    #[arg(long, default_value_t = 1000)]
    data_size: usize,

    #[arg(long, default_value_t = String::from("dataset.csv"))]
    dataset_file: String,

    #[arg(long, short, default_value_t = String::from("models/weights.json"))]
    weights_file: String,

    #[arg(long, short, default_value_t = false)]
    record: bool,

    #[arg(long, default_value_t = String::from("save.txt"))]
    record_file: String,
}
fn make_move(
    board: &mut Board,
    engine_nn: &NeuralNet,
    depth: u32,
    tt_table: &mut TranspositionTable,
    file: &mut Option<File>,
) -> bool {
    let mut total = 0;
    match best_move::find_best_move(board, engine_nn, depth, tt_table, &mut total) {
        Some(best_move) => {
            record_to_file(file, board, Some(best_move));
            let _ = board.make_move(&best_move);
            print!("\x1B[H\x1B[2J");
            println!("{}", board);
            true
        }
        None => {
            record_to_file(file, board, None);
            if board.is_in_check(board.white_to_move) {
                println!("checkmate");
            } else {
                println!("stalemate");
            }
            false
        }
    }
}
fn stockfish_make_move(
    stockfish: &mut Stockfish,
    board: &mut Board,
    depth: u8,
    file: &mut Option<File>,
) -> bool {
    match stockfish.get_best_move(board, depth) {
        Some(best_move) => {
            record_to_file(file, board, Some(best_move));
            let _ = board.make_move(&best_move);
            print!("\x1B[H\x1B[2J");
            println!("{}", board);
            true
        }
        None => {
            record_to_file(file, board, None);
            if board.is_in_check(board.white_to_move) {
                println!("checkmate");
            } else {
                println!("stalemate");
            }
            false
        }
    }
}
fn record_to_file(file: &mut Option<File>, board: &mut Board, b_move: Option<Move>) {
    let file = match file.as_mut() {
        Some(f) => f,
        None => return,
    };
    if let Some(b_move) = b_move {
        let san = board.move_to_san(&b_move);
        if board.white_to_move {
            write!(file, "{}. {} ", board.fullmove_number, san).unwrap();
        } else {
            write!(file, "{} ", san).unwrap();
        }
    } else {
        if board.is_in_check(board.white_to_move) {
            writeln!(file, "{}", if board.white_to_move { "0-1" } else { "1-0" }).unwrap();
        } else {
            writeln!(file, "1/2-1/2").unwrap();
        }
    }
}
fn play_self(depth: u32, record: bool, file_name: String, weights: String) {
    let engine_nn = NeuralNet::load(&weights);
    let mut board = Board::new();
    let mut tt_table = TranspositionTable::new(256);
    println!("\x1B[H\x1B[2J{}", board);

    let mut file = if record {
        Some(
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(file_name)
                .expect("Failed to open file"),
        )
    } else {
        None
    };
    loop {
        if !make_move(&mut board, &engine_nn, depth, &mut tt_table, &mut file) {
            break;
        }
    }
}

fn play_stockfish(depth: u32, as_white: bool, record: bool, file_name: String, weights: String) {
    let engine_nn = NeuralNet::load(&weights);
    let mut stockfish = Stockfish::new(None);
    let mut board = Board::new();
    let mut tt_table = TranspositionTable::new(256);
    println!("\x1B[H\x1B[2J{}", board);

    let mut file = if record {
        Some(
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(file_name)
                .expect("Failed to open file"),
        )
    } else {
        None
    };
    if as_white {
        loop {
            if !make_move(&mut board, &engine_nn, depth, &mut tt_table, &mut file) {
                break;
            }
            if !stockfish_make_move(&mut stockfish, &mut board, depth as u8, &mut file) {
                break;
            }
        }
    } else {
        loop {
            if !stockfish_make_move(&mut stockfish, &mut board, depth as u8, &mut file) {
                break;
            }
            if !make_move(&mut board, &engine_nn, depth, &mut tt_table, &mut file) {
                break;
            }
        }
    }
}
fn main() {
    let args = Args::parse();
    if args.self_play {
        play_self(args.depth, args.record, args.record_file, args.weights_file);
    } else if args.play_stockfish {
        play_stockfish(args.depth, !args.as_black, args.record, args.record_file, args.weights_file);
    } else if args.gen_dataset {
        gen_dataset(
            gen_dataset::Encoding::Fen,
            None,
            args.data_size,
            args.depth as u8,
            args.dataset_file,
        );
    } else {
        uci::uci();
    }
}
