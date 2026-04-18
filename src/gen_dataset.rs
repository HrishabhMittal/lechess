use crate::{board::*, stockfish::Stockfish};
use rand::{prelude::*, rng};
use std::num::NonZero;
use std::sync::mpsc::Sender;
use std::{fs::File, io::Write, sync::mpsc, thread};
#[derive(Clone, Copy)]
pub enum Encoding {
    Simple,
    Fen,
}

fn get_random_evaluations(tx: Sender<String>, simple_or_fen: Encoding, total: usize, depth: u8) {
    let mut sf = Stockfish::new(None);
    let mut r = rng();
    for _ in 1..total {
        let mut board = Board::new();
        for _ in 1..10 {
            let mut inner_broken = false;
            for _ in 1..5 {
                let moves = board.generate_legal_moves();
                let mov = moves.choose(&mut r);
                match mov {
                    Some(v) => {
                        let _ = board.make_move(v);
                    }
                    None => {}
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
                    str = format!("{},{}\n", fen, sf.get_eval(&fen, depth));
                }
                Encoding::Simple => {
                    let simple = board.to_simple();
                    str = format!("{},{}\n", simple, sf.get_eval(&fen, depth));
                }
            }
            tx.send(str).unwrap();
            if inner_broken {
                break;
            }
        }
    }
}

pub fn gen_dataset(
    simple_or_fen: Encoding,
    num_cores: Option<usize>,
    lines: usize,
    depth: u8,
    file_name: String,
) {
    let mut file = File::create(file_name).unwrap();
    file.write_all("board,Analysis\n".as_bytes()).unwrap();
    let num_cores = match num_cores {
        Some(v) => v,
        None => thread::available_parallelism()
            .unwrap_or(NonZero::new(4).unwrap())
            .get(),
    };
    let per_core = lines / num_cores;
    let last_core_extra = lines - per_core * num_cores;
    let mut handles = vec![];
    let (tx, rx) = mpsc::channel::<String>();
    for i in 1..=num_cores {
        let tx_clone = tx.clone();
        let work = if i == num_cores {
            per_core + last_core_extra
        } else {
            per_core
        };
        let handle = thread::spawn(move || {
            get_random_evaluations(tx_clone, simple_or_fen, work, depth);
        });
        handles.push(handle);
    }
    drop(tx);
    let mut written = 0;
    for line in rx {
        if written % 500 == 0 {
            println!("written {} lines", written);
        }
        file.write_all(line.as_bytes()).unwrap();
        written += 1;
    }
    for handle in handles {
        handle.join().unwrap();
    }
}
