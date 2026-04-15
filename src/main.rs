use crate::{
    board::{Board, Move, MoveFlag},
    nn::NeuralNet,
};

mod board;
mod eval;
mod gen_dataset;
mod nn;

const MAX_PLY: usize = 128;

const MVV_LVA: [[i32; 7]; 7] = [
    [0, 0, 0, 0, 0, 0, 0],
    [50, 51, 52, 53, 54, 55, 0],
    [40, 41, 42, 43, 44, 45, 0],
    [30, 31, 32, 33, 34, 35, 0],
    [20, 21, 22, 23, 24, 25, 0],
    [10, 11, 12, 13, 14, 15, 0],
    [0, 0, 0, 0, 0, 0, 0],
];

fn piece_at(board: &Board, sq: u8) -> usize {
    let bb = 1u64 << sq;
    if (board.white.queen | board.black.queen) & bb != 0 {
        return 1;
    }
    if (board.white.rook | board.black.rook) & bb != 0 {
        return 2;
    }
    if (board.white.bishop | board.black.bishop) & bb != 0 {
        return 3;
    }
    if (board.white.knight | board.black.knight) & bb != 0 {
        return 4;
    }
    if (board.white.pawn | board.black.pawn) & bb != 0 {
        return 5;
    }
    if (board.white.king | board.black.king) & bb != 0 {
        return 0;
    }
    6
}

fn score_move(board: &Board, m: &Move, ply: usize, killers: &[[Option<Move>; 2]; MAX_PLY]) -> i32 {
    let mut score = 0;
    match m.flag {
        MoveFlag::PromoQueen | MoveFlag::PromoCaptureQueen => score += 900,
        MoveFlag::PromoRook | MoveFlag::PromoCaptureRook => score += 500,
        MoveFlag::PromoBishop | MoveFlag::PromoCaptureBishop => score += 300,
        MoveFlag::PromoKnight | MoveFlag::PromoCaptureKnight => score += 300,
        _ => {}
    }
    let is_capture = matches!(
        m.flag,
        MoveFlag::Capture
            | MoveFlag::EnPassant
            | MoveFlag::PromoCaptureQueen
            | MoveFlag::PromoCaptureRook
            | MoveFlag::PromoCaptureBishop
            | MoveFlag::PromoCaptureKnight
    );

    if is_capture {
        let attacker = piece_at(board, m.from);
        let victim = if m.flag == MoveFlag::EnPassant {
            5
        } else {
            piece_at(board, m.to)
        };
        score += MVV_LVA[victim][attacker] + 10000;
    } else if ply < MAX_PLY {
        if killers[ply][0] == Some(*m) {
            score += 9000;
        } else if killers[ply][1] == Some(*m) {
            score += 8000;
        }
    }
    score
}

fn evaluate_position(
    board: &Board,
    engine_nn: &NeuralNet,
    depth: u32,
    ply: usize,
    mut alpha: f32,
    beta: f32,
    killers: &mut [[Option<Move>; 2]; MAX_PLY],
) -> f32 {
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
    let mut scored_moves: Vec<(Move, i32)> = moves
        .into_iter()
        .map(|m| {
            let score = score_move(board, &m, ply, killers);
            (m, score)
        })
        .collect();
    scored_moves.sort_by(|a, b| b.1.cmp(&a.1));
    let mut max_val = f32::NEG_INFINITY;
    for (m, _) in scored_moves {
        let next_board = board.make_move(&m);
        let score = -evaluate_position(
            &next_board,
            engine_nn,
            depth - 1,
            ply + 1,
            -beta,
            -alpha,
            killers,
        );
        if score > max_val {
            max_val = score;
        }
        if alpha < score {
            alpha = score;
        }
        if alpha >= beta {
            let is_capture = matches!(
                m.flag,
                MoveFlag::Capture
                    | MoveFlag::EnPassant
                    | MoveFlag::PromoCaptureQueen
                    | MoveFlag::PromoCaptureRook
                    | MoveFlag::PromoCaptureBishop
                    | MoveFlag::PromoCaptureKnight
            );
            if !is_capture && ply < MAX_PLY {
                if killers[ply][0] != Some(m) {
                    killers[ply][1] = killers[ply][0];
                    killers[ply][0] = Some(m);
                }
            }
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
    let mut killers: [[Option<Move>; 2]; MAX_PLY] = [[None; 2]; MAX_PLY];
    let mut best_move = moves[0];
    let mut max_val = f32::NEG_INFINITY;
    let mut alpha = f32::NEG_INFINITY;
    let beta = f32::INFINITY;
    let mut scored_moves: Vec<(Move, i32)> = moves
        .into_iter()
        .map(|m| {
            let score = score_move(board, &m, 0, &killers);
            (m, score)
        })
        .collect();
    scored_moves.sort_by(|a, b| b.1.cmp(&a.1));
    for (m, _) in scored_moves {
        let next_board = board.make_move(&m);
        let score = -evaluate_position(
            &next_board,
            engine_nn,
            depth - 1,
            1,
            -beta,
            -alpha,
            &mut killers,
        );
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
    let search_depth = 4;
    for _ in 1..=100 {
        match find_best_move(&board, &engine_nn, search_depth) {
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
