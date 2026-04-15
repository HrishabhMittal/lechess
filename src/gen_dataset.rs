use crate::{board::*, eval::Stockfish};
use rand::{prelude::*, rng};
use std::sync::mpsc::Sender;
use std::{fs::File, io::Write, sync::mpsc, thread};
#[derive(Clone, Copy)]
pub enum Encoding {
    Simple,
    Fen,
}

fn get_random_evaluations(tx: Sender<String>, simple_or_fen: Encoding) {
    let mut sf = Stockfish::new(None);
    let mut r = rng();
    let total = 1000;
    for _ in 1..total {
        let mut board = Board::new();
        for _ in 1..10 {
            let mut inner_broken = false;
            for _ in 1..5 {
                let moves = board.generate_legal_moves();
                let mov = moves.choose(&mut r);
                board = match mov {
                    Some(v) => board.make_move(v),
                    None => board,
                };
                if mov.is_none() {
                    inner_broken = true;
                    break;
                }
            }
            let str;
            let fen = board.to_fen();
            match simple_or_fen {
                Encoding::Fen => {
                    str = format!("{},{}\n", fen, sf.get_eval(&fen, 22));
                }
                Encoding::Simple => {
                    let simple = board.to_simple();
                    str = format!("{},{}\n", simple, sf.get_eval(&fen, 22));
                }
            }
            tx.send(str).unwrap();
            if inner_broken {
                break;
            }
        }
    }
}

pub fn gen_dataset(simple_or_fen: Encoding) {
    let mut file = File::create("dataset.csv").unwrap();
    file.write_all("board,Analysis\n".as_bytes()).unwrap();
    let num_cores = thread::available_parallelism().unwrap().get();
    let mut handles = vec![];
    let (tx, rx) = mpsc::channel::<String>();
    for _ in 0..num_cores {
        let tx_clone = tx.clone();
        let handle = thread::spawn(move || {
            get_random_evaluations(tx_clone, simple_or_fen);
        });
        handles.push(handle);
    }
    drop(tx);
    let mut written = 0;
    for line in rx {
        if written % 100 == 0 {
            println!("written {} lines", written);
        }
        file.write_all(line.as_bytes()).unwrap();
        written += 1;
    }
    for handle in handles {
        handle.join().unwrap();
    }
}
