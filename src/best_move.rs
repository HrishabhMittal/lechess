use std::time::Instant;

use crate::{
    board::{Board, Move, MoveFlag},
    nn::{Accumulator, NeuralNet},
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

fn move_to_uci(m: &Move) -> String {
    let from_file = (m.from % 8) as u8;
    let from_rank = (m.from / 8) as u8;
    let to_file = (m.to % 8) as u8;
    let to_rank = (m.to / 8) as u8;
    let from_str = format!(
        "{}{}",
        (b'a' + from_file) as char,
        (b'1' + from_rank) as char
    );
    let to_str = format!("{}{}", (b'a' + to_file) as char, (b'1' + to_rank) as char);
    let promo = match m.flag {
        MoveFlag::PromoQueen | MoveFlag::PromoCaptureQueen => "q",
        MoveFlag::PromoRook | MoveFlag::PromoCaptureRook => "r",
        MoveFlag::PromoBishop | MoveFlag::PromoCaptureBishop => "b",
        MoveFlag::PromoKnight | MoveFlag::PromoCaptureKnight => "n",
        _ => "",
    };
    format!("{}{}{}", from_str, to_str, promo)
}
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
pub fn build_lmr_table() -> [[u32; 64]; 64] {
    let mut table = [[0; 64]; 64];
    for depth in 1..64 {
        for move_idx in 1..64 {
            let d = depth as f64;
            let m = move_idx as f64;
            let reduction = 0.75 + (d.ln() * m.ln() / 2.25);
            table[depth][move_idx] = reduction as u32;
        }
    }
    table
}
fn pv(board: &mut Board, tt: &TranspositionTable, depth: u32) -> Vec<String> {
    let mut pv = Vec::new();
    let mut out = Vec::new();
    let mut undos = Vec::new();

    for _ in 0..depth {
        if let Some(entry) = tt.probe(board.zobrist_hash) {
            if let Some(best_move) = entry.best_move {
                pv.push(best_move);
                out.push(move_to_uci(&best_move));

                undos.push(board.make_move(&best_move));
            } else {
                break;
            }
        } else {
            break;
        }
    }

    for (m, undo) in pv.iter().zip(undos.iter()).rev() {
        board.unmake_move(m, undo);
    }

    out
}
fn score_move(
    board: &Board,
    m: &Move,
    ply: usize,
    killers: &[[Option<Move>; 2]; MAX_PLY],
    history: &[[[i32; 64]; 64]; 2],
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
        let mut killer_matched = false;
        if ply < MAX_PLY {
            if killers[ply][0] == Some(*m) {
                score += 9000;
                killer_matched = true;
            } else if killers[ply][1] == Some(*m) {
                score += 8000;
                killer_matched = true;
            }
        }
        if !killer_matched {
            let color_idx = if board.white_to_move { 0 } else { 1 };
            score += history[color_idx][m.from as usize][m.to as usize];
        }
    }
    score
}
fn qs(
    board: &mut Board,
    engine_nn: &NeuralNet,
    mut alpha: i32,
    beta: i32,
    total: &mut u32,
    acc: &Accumulator,
) -> i32 {
    *total += 1;

    let stand_pat = engine_nn.evaluate_from_acc(acc, board.white_to_move);

    if stand_pat >= beta {
        return beta;
    }
    if alpha < stand_pat {
        alpha = stand_pat;
    }

    let moves = board.generate_pseudo_legal_moves();

    let mut captures = [(
        Move {
            from: 0,
            to: 0,
            flag: MoveFlag::Quiet,
        },
        0i32,
    ); 256];
    let mut cap_count = 0;

    for m in moves.iter().copied() {
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

            let score = MVV_LVA[victim][attacker] + 10000;
            captures[cap_count] = (m, score);
            cap_count += 1;
        }
    }

    let cap_slice = &mut captures[..cap_count];
    cap_slice.sort_unstable_by(|a, b| b.1.cmp(&a.1));

    for (m, _) in cap_slice.iter() {
        let is_promo = matches!(m.flag, MoveFlag::PromoQueen | MoveFlag::PromoCaptureQueen);
        if !is_promo {
            let victim = if m.flag == MoveFlag::EnPassant {
                5
            } else {
                piece_at(board, m.to)
            };
            let victim_val = match victim {
                1 => 900,
                2 => 500,
                3 => 300,
                4 => 300,
                5 => 100,
                _ => 0,
            };

            if stand_pat + victim_val + 200 < alpha {
                continue;
            }
        }
        let undo = board.make_move(m);

        if board.is_in_check(!board.white_to_move) {
            board.unmake_move(m, &undo);
            continue;
        }

        let mut next_acc = *acc;
        engine_nn.update_from_move(board, m, &undo, &mut next_acc);

        let score = -qs(board, engine_nn, -beta, -alpha, total, &next_acc);

        board.unmake_move(m, &undo);

        if score >= beta {
            return beta;
        }
        if score > alpha {
            alpha = score;
        }
    }

    alpha
}
fn evaluate_position(
    board: &mut Board,
    engine_nn: &NeuralNet,
    mut depth: u32,
    ply: usize,
    mut alpha: i32,
    mut beta: i32,
    killers: &mut [[Option<Move>; 2]; MAX_PLY],
    history: &mut [[[i32; 64]; 64]; 2],
    lmr_table: &[[u32; 64]; 64],
    tt_table: &mut TranspositionTable,
    total: &mut u32,
    acc: &Accumulator,
) -> i32 {
    *total += 1;
    let og_alpha = alpha;
    let mut moves = board.generate_pseudo_legal_moves();

    if moves.is_empty() {
        if board.is_in_check(board.white_to_move) {
            return -10000 + ply as i32;
        } else {
            return 0;
        }
    }

    if depth == 0 {
        if board.is_in_check(board.white_to_move) {
            depth += 1;
        } else {
            return qs(board, engine_nn, alpha, beta, total, acc);
        }
    }

    let mut tt_move: Option<Move> = None;

    if let Some(tt_entry) = tt_table.probe(board.zobrist_hash) {
        tt_move = tt_entry.best_move;
        if tt_entry.depth >= depth {
            let mut tt_score = tt_entry.score;
            if tt_score > 9000 {
                tt_score -= ply as i32;
            } else if tt_score < -9000 {
                tt_score += ply as i32;
            }
            match tt_entry.flag {
                TTFlag::Exact => return tt_score,
                TTFlag::LowerBound => alpha = alpha.max(tt_score),
                TTFlag::UpperBound => beta = beta.min(tt_score),
            }
            if alpha >= beta {
                return tt_score;
            }
        }
    }

    let static_eval = engine_nn.evaluate_from_acc(acc, board.white_to_move);
    let in_check = board.is_in_check(board.white_to_move);

    if depth <= 5 && !in_check && ply > 0 {
        let rfp_margin = 75 * depth as i32;
        if static_eval - rfp_margin >= beta {
            return static_eval;
        }
    }

    if depth >= 3 && !in_check && ply > 0 && static_eval >= beta {
        let mut null_board = board.clone();

        null_board.white_to_move = !null_board.white_to_move;
        null_board.en_passant_target = None;

        let z = crate::board::get_zobrist();
        null_board.zobrist_hash ^= z.side_to_move;
        if let Some(sq) = board.en_passant_target {
            null_board.zobrist_hash ^= z.en_passant[(sq % 8) as usize];
        }
        let r = 3 + (depth / 3);
        let reduced_depth = depth.saturating_sub(1 + r);
        let null_score = -evaluate_position(
            &mut null_board,
            engine_nn,
            reduced_depth,
            ply + 1,
            -beta,
            -beta + 1,
            killers,
            history,
            lmr_table,
            tt_table,
            total,
            acc,
        );

        if null_score >= beta {
            return beta;
        }
    }

    moves.sort_unstable_by(|a, b| {
        let score_a = score_move(board, a, ply, &killers, &history, tt_move);
        let score_b = score_move(board, b, ply, &killers, &history, tt_move);
        score_b.cmp(&score_a)
    });

    let mut max_val = -50000;
    let mut tt_best = None;
    let mut legal_played = 0;

    for m in moves.iter().copied() {
        let undo = board.make_move(&m);

        if board.is_in_check(!board.white_to_move) {
            board.unmake_move(&m, &undo);
            continue;
        }

        legal_played += 1;
        let mut next_acc = *acc;
        engine_nn.update_from_move(board, &m, &undo, &mut next_acc);

        let is_tactical = matches!(
            m.flag,
            MoveFlag::Capture
                | MoveFlag::EnPassant
                | MoveFlag::PromoQueen
                | MoveFlag::PromoRook
                | MoveFlag::PromoBishop
                | MoveFlag::PromoKnight
                | MoveFlag::PromoCaptureQueen
                | MoveFlag::PromoCaptureRook
                | MoveFlag::PromoCaptureBishop
                | MoveFlag::PromoCaptureKnight
        );

        let gives_check = board.is_in_check(board.white_to_move);

        if depth <= 3 && !in_check && !is_tactical && !gives_check && legal_played > 1 {
            let futility_margin = 150 * depth as i32;
            if static_eval + futility_margin <= alpha {
                board.unmake_move(&m, &undo);
                continue;
            }
        }

        let mut score;

        if legal_played == 1 {
            score = -evaluate_position(
                board,
                engine_nn,
                depth - 1,
                ply + 1,
                -beta,
                -alpha,
                killers,
                history,
                lmr_table,
                tt_table,
                total,
                &next_acc,
            );
        } else {
            let mut reduction = 0;

            if depth >= 3 && legal_played > 3 && !is_tactical && !gives_check {
                let d = (depth as usize).min(63);
                let m_idx = legal_played.min(63);

                reduction = lmr_table[d][m_idx];

                if killers[ply][0] == Some(m) || killers[ply][1] == Some(m) {
                    if reduction > 0 {
                        reduction -= 1;
                    }
                }
            }

            if reduction > 0 {
                let reduced_depth = (depth - 1).saturating_sub(reduction);
                score = -evaluate_position(
                    board,
                    engine_nn,
                    reduced_depth,
                    ply + 1,
                    -alpha - 1,
                    -alpha,
                    killers,
                    history,
                    lmr_table,
                    tt_table,
                    total,
                    &next_acc,
                );

                if score > alpha {
                    score = -evaluate_position(
                        board,
                        engine_nn,
                        depth - 1,
                        ply + 1,
                        -alpha - 1,
                        -alpha,
                        killers,
                        history,
                        lmr_table,
                        tt_table,
                        total,
                        &next_acc,
                    );
                }
            } else {
                score = -evaluate_position(
                    board,
                    engine_nn,
                    depth - 1,
                    ply + 1,
                    -alpha - 1,
                    -alpha,
                    killers,
                    history,
                    lmr_table,
                    tt_table,
                    total,
                    &next_acc,
                );
            }

            if score > alpha && score < beta {
                score = -evaluate_position(
                    board,
                    engine_nn,
                    depth - 1,
                    ply + 1,
                    -beta,
                    -alpha,
                    killers,
                    history,
                    lmr_table,
                    tt_table,
                    total,
                    &next_acc,
                );
            }
        }

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
            let color_idx = if !board.white_to_move { 0 } else { 1 };
            let bonus = (depth * depth) as i32;
            let entry = &mut history[color_idx][m.from as usize][m.to as usize];

            *entry = (*entry + bonus).min(7999);
            board.unmake_move(&m, &undo);
            break;
        }
        board.unmake_move(&m, &undo);
    }

    if legal_played == 0 {
        if board.is_in_check(board.white_to_move) {
            return -10000 + ply as i32;
        } else {
            return 0;
        }
    }

    let flag = if max_val <= og_alpha {
        TTFlag::UpperBound
    } else if max_val >= beta {
        TTFlag::LowerBound
    } else {
        TTFlag::Exact
    };

    let mut store_score = max_val;
    if store_score > 9000 {
        store_score += ply as i32;
    } else if store_score < -9000 {
        store_score -= ply as i32;
    }
    tt_table.store(board.zobrist_hash, depth, store_score, flag, tt_best);

    max_val
}
pub fn find_best_move(
    board: &mut Board,
    engine_nn: &NeuralNet,
    depth: u32,
    tt_table: &mut TranspositionTable,
    total: &mut u32,
    verbose: bool,
    time_limit_ms: Option<u128>,
) -> Option<Move> {
    let moves = board.generate_legal_moves();
    if moves.is_empty() {
        return None;
    }

    let acc = engine_nn.refresh_accumulator(board);

    let mut killers: [[Option<Move>; 2]; MAX_PLY] = [[None; 2]; MAX_PLY];
    let mut history: [[[i32; 64]; 64]; 2] = [[[0; 64]; 64]; 2];

    let lmr_table = build_lmr_table();

    let mut best_move = moves[0];
    let mut completed_best_move = moves[0];
    let start_time = Instant::now();

    for cur_depth in 1..=depth {
        let mut max_val = -50000;
        let mut alpha = -50000;
        let beta = 50000;
        let mut aborted = false;

        let mut tt_move: Option<Move> = None;
        if let Some(tt_entry) = tt_table.probe(board.zobrist_hash) {
            tt_move = tt_entry.best_move;
        }

        let mut scored_moves: Vec<(Move, i32)> = moves
            .iter()
            .map(|m| {
                let score = score_move(board, m, 0, &killers, &history, tt_move);
                (*m, score)
            })
            .collect();

        scored_moves.sort_by(|a, b| b.1.cmp(&a.1));

        for (m, _) in scored_moves.iter() {
            if let Some(limit) = time_limit_ms {
                if start_time.elapsed().as_millis() > limit {
                    aborted = true;
                    break;
                }
            }

            let mut next_acc = acc;
            let undo = board.make_move(&m);

            engine_nn.update_from_move(board, m, &undo, &mut next_acc);

            let score = -evaluate_position(
                board,
                engine_nn,
                cur_depth - 1,
                1,
                -beta,
                -alpha,
                &mut killers,
                &mut history,
                &lmr_table,
                tt_table,
                total,
                &next_acc,
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
        if aborted {
            break;
        }

        completed_best_move = best_move;

        tt_table.store(
            board.zobrist_hash,
            cur_depth,
            max_val,
            TTFlag::Exact,
            Some(best_move),
        );

        if verbose {
            let elapsed_ms = start_time.elapsed().as_millis().max(1);
            let nps = (*total as u128 * 1000) / elapsed_ms;

            let score_string = if max_val > 9000 {
                let moves_to_mate = (10000 - max_val + 1) / 2;
                format!("mate {}", moves_to_mate)
            } else if max_val < -9000 {
                let moves_to_mate = (-10000 - max_val) / 2;
                format!("mate {}", -moves_to_mate)
            } else {
                format!("cp {}", max_val)
            };

            let pv_moves = pv(board, tt_table, cur_depth);
            let pv_string = pv_moves.join(" ");

            println!(
                "info depth {} score {} nodes {} nps {} time {} pv {}",
                cur_depth, score_string, *total, nps, elapsed_ms, pv_string
            );
        }
    }
    Some(completed_best_move)
}
