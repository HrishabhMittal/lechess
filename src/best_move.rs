use crate::{
    board::{Board, Move, MoveFlag},
    nn::NeuralNet,
    tt::{TTFlag, TranspositionTable},
};
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

fn score_move(
    board: &Board,
    m: &Move,
    ply: usize,
    killers: &[[Option<Move>; 2]; MAX_PLY],
    tt_move: Option<Move>,
) -> i32 {
    if Some(*m) == tt_move {
        return 30000;
    }
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
    board: &mut Board,
    engine_nn: &NeuralNet,
    depth: u32,
    ply: usize,
    mut alpha: f32,
    mut beta: f32,
    killers: &mut [[Option<Move>; 2]; MAX_PLY],
    tt_table: &mut TranspositionTable,
    total: &mut u32,
) -> f32 {
    *total += 1;
    let og_alpha = alpha;
    let mut moves = board.generate_pseudo_legal_moves();
    if moves.is_empty() {
        if board.is_in_check(board.white_to_move) {
            return -10000.0 - (depth as f32);
        } else {
            return 0.0;
        }
    }
    if depth == 0 {
        // return engine_nn.evaluate(&board.to_features());
        return board.static_eval_color_neutral() as f32;
    }
    let mut tt_move: Option<Move> = None;

    if let Some(tt_entry) = tt_table.probe(board.zobrist_hash) {
        tt_move = tt_entry.best_move;
        if tt_entry.depth >= depth {
            match tt_entry.flag {
                TTFlag::Exact => return tt_entry.score,
                TTFlag::LowerBound => alpha = alpha.max(tt_entry.score),
                TTFlag::UpperBound => beta = beta.min(tt_entry.score),
            }
            if alpha >= beta {
                return tt_entry.score;
            }
        }
    }
    if depth >= 3 && !board.is_in_check(board.white_to_move) && ply > 0 {
        let mut null_board = board.clone();

        null_board.white_to_move = !null_board.white_to_move;
        null_board.en_passant_target = None;

        let z = crate::board::get_zobrist();
        null_board.zobrist_hash ^= z.side_to_move;
        if let Some(sq) = board.en_passant_target {
            null_board.zobrist_hash ^= z.en_passant[(sq % 8) as usize];
        }

        let null_score = -evaluate_position(
            &mut null_board,
            engine_nn,
            depth - 1 - 2,
            ply + 1,
            -beta,
            -beta + 1.0,
            killers,
            tt_table,
            total,
        );

        if null_score >= beta {
            return beta;
        }
    }
    moves.sort_unstable_by(|a, b| {
        let score_a = score_move(board, a, ply, &killers, tt_move);
        let score_b = score_move(board, b, ply, &killers, tt_move);
        score_b.cmp(&score_a)
    });
    let mut max_val = -50000.0;
    let mut tt_best = None;
    let mut legal_played = 0;
    for m in moves.iter().copied() {
        let undo = board.make_move(&m);
        if board.is_in_check(!board.white_to_move) {
            board.unmake_move(&m, &undo);
            continue;
        }
        legal_played += 1;
        let is_capture = matches!(
            m.flag,
            MoveFlag::Capture
                | MoveFlag::EnPassant
                | MoveFlag::PromoCaptureQueen
                | MoveFlag::PromoCaptureRook
                | MoveFlag::PromoCaptureBishop
                | MoveFlag::PromoCaptureKnight
        );
        let score;
        if legal_played > 3 && depth >= 3 && !is_capture && !board.is_in_check(board.white_to_move)
        {
            let reduced_score = -evaluate_position(
                board,
                engine_nn,
                depth - 2,
                ply + 1,
                -beta,
                -alpha,
                killers,
                tt_table,
                total,
            );

            if reduced_score > alpha {
                score = -evaluate_position(
                    board,
                    engine_nn,
                    depth - 1,
                    ply + 1,
                    -beta,
                    -alpha,
                    killers,
                    tt_table,
                    total,
                );
            } else {
                score = reduced_score;
            }
        } else {
            score = -evaluate_position(
                board,
                engine_nn,
                depth - 1,
                ply + 1,
                -beta,
                -alpha,
                killers,
                tt_table,
                total,
            );
        };
        if score > max_val {
            max_val = score;
            tt_best = Some(m);
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
            board.unmake_move(&m, &undo);
            break;
        }
        board.unmake_move(&m, &undo);
    }
    if legal_played == 0 {
        if board.is_in_check(board.white_to_move) {
            return -10000.0 - (ply as f32);
        } else {
            return 0.0;
        }
    }
    let flag = if max_val <= og_alpha {
        TTFlag::UpperBound
    } else if max_val >= beta {
        TTFlag::LowerBound
    } else {
        TTFlag::Exact
    };
    tt_table.store(board.zobrist_hash, depth, max_val, flag, tt_best);
    max_val
}

pub fn find_best_move(
    board: &mut Board,
    engine_nn: &NeuralNet,
    depth: u32,
    tt_table: &mut TranspositionTable,
    total: &mut u32,
) -> Option<Move> {
    let moves = board.generate_legal_moves();
    if moves.is_empty() {
        return None;
    }
    let mut killers: [[Option<Move>; 2]; MAX_PLY] = [[None; 2]; MAX_PLY];
    let mut best_move = moves[0];
    for cur_depth in 1..=depth {
        let mut max_val = -50000.0;
        let mut alpha = -50000.0;
        let beta = 50000.0;
        let mut tt_move: Option<Move> = None;
        if let Some(tt_entry) = tt_table.probe(board.zobrist_hash) {
            tt_move = tt_entry.best_move;
        }
        let mut scored_moves: Vec<(Move, i32)> = moves
            .iter()
            .map(|m| {
                let score = score_move(board, m, 0, &killers, tt_move);
                (*m, score)
            })
            .collect();
        scored_moves.sort_by(|a, b| b.1.cmp(&a.1));
        for (m, _) in scored_moves.iter() {
            let undo = board.make_move(&m);
            let score = -evaluate_position(
                board,
                engine_nn,
                cur_depth - 1,
                1,
                -beta,
                -alpha,
                &mut killers,
                tt_table,
                total,
            );
            if score > max_val {
                max_val = score;
                best_move = *m;
            }
            if alpha < score {
                alpha = score;
            }
            board.unmake_move(&m, &undo);
        }
        tt_table.store(
            board.zobrist_hash,
            cur_depth,
            max_val,
            TTFlag::Exact,
            Some(best_move),
        );
    }
    Some(best_move)
}
