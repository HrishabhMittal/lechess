use crate::board::{Board, Move, MoveFlag, UndoState};
use serde::Deserialize;
use std::{fs::File, io::BufReader};
fn piece_on_sq(board: &Board, sq: u8) -> Option<(bool, usize)> {
    let bb = 1u64 << sq;
    if (board.white.pawn & bb) != 0 {
        return Some((true, 0));
    }
    if (board.black.pawn & bb) != 0 {
        return Some((false, 0));
    }
    if (board.white.knight & bb) != 0 {
        return Some((true, 1));
    }
    if (board.black.knight & bb) != 0 {
        return Some((false, 1));
    }
    if (board.white.bishop & bb) != 0 {
        return Some((true, 2));
    }
    if (board.black.bishop & bb) != 0 {
        return Some((false, 2));
    }
    if (board.white.rook & bb) != 0 {
        return Some((true, 3));
    }
    if (board.black.rook & bb) != 0 {
        return Some((false, 3));
    }
    if (board.white.queen & bb) != 0 {
        return Some((true, 4));
    }
    if (board.black.queen & bb) != 0 {
        return Some((false, 4));
    }
    if (board.white.king & bb) != 0 {
        return Some((true, 5));
    }
    if (board.black.king & bb) != 0 {
        return Some((false, 5));
    }
    None
}
#[derive(Deserialize)]
pub struct Weights {
    #[serde(rename = "fc1.weight")]
    fc1_w: Vec<Vec<i16>>,
    #[serde(rename = "fc1.bias")]
    fc1_b: Vec<i16>,
    #[serde(rename = "fc2.weight")]
    fc2_w: Vec<Vec<i16>>,
    #[serde(rename = "fc2.bias")]
    fc2_b: Vec<i16>,
    #[serde(rename = "fc3.weight")]
    fc3_w: Vec<Vec<i16>>,
    #[serde(rename = "fc3.bias")]
    fc3_b: Vec<i16>,
}
#[derive(Clone, Copy)]
pub struct Accumulator {
    pub white: [i16; 256],
    pub black: [i16; 256],
}
pub struct NeuralNet {
    fc1_w_transposed: Box<[i16]>,
    fc1_b: Box<[i16]>,
    fc2_w: Box<[i16]>,
    fc2_b: Box<[i16]>,
    fc3_w: Box<[i16]>,
    fc3_b: Box<[i16]>,
    scale: i32,
}
impl NeuralNet {
    pub fn load(path: &str) -> Self {
        let file = File::open(path).expect("couldnt open weights");
        let reader = BufReader::new(file);
        let weights: Weights = serde_json::from_reader(reader).expect("couldnt parse weights");
        let mut fc1_w_transposed = vec![0i16; 45056 * 256];
        for i in 0..256 {
            for j in 0..45056 {
                fc1_w_transposed[j * 256 + i] = weights.fc1_w[i][j];
            }
        }
        let mut fc2_w = vec![0i16; 32 * 256];
        for i in 0..32 {
            for j in 0..256 {
                fc2_w[i * 256 + j] = weights.fc2_w[i][j];
            }
        }
        let mut fc3_w = vec![0i16; 32];
        for i in 0..32 {
            fc3_w[i] = weights.fc3_w[0][i];
        }
        NeuralNet {
            fc1_w_transposed: fc1_w_transposed.into_boxed_slice(),
            fc1_b: weights.fc1_b.into_boxed_slice(),
            fc2_w: fc2_w.into_boxed_slice(),
            fc2_b: weights.fc2_b.into_boxed_slice(),
            fc3_w: fc3_w.into_boxed_slice(),
            fc3_b: weights.fc3_b.into_boxed_slice(),
            scale: 127,
        }
    }
    #[inline(always)]
    fn add_to_white(
        &self,
        acc: &mut [i16; 256],
        w_king_sq: usize,
        sq: usize,
        piece_idx: usize,
        is_white_piece: bool,
    ) {
        let p_idx = if is_white_piece {
            piece_idx
        } else {
            piece_idx + 5
        };
        let idx = w_king_sq * 704 + p_idx * 64 + sq;
        let weights = &self.fc1_w_transposed[idx * 256..(idx + 1) * 256];
        for (a, &w) in acc.iter_mut().zip(weights) {
            *a += w;
        }
    }
    #[inline(always)]
    fn sub_from_white(
        &self,
        acc: &mut [i16; 256],
        w_king_sq: usize,
        sq: usize,
        piece_idx: usize,
        is_white_piece: bool,
    ) {
        let p_idx = if is_white_piece {
            piece_idx
        } else {
            piece_idx + 5
        };
        let idx = w_king_sq * 704 + p_idx * 64 + sq;
        let weights = &self.fc1_w_transposed[idx * 256..(idx + 1) * 256];
        for (a, &w) in acc.iter_mut().zip(weights) {
            *a -= w;
        }
    }
    #[inline(always)]
    fn add_to_black(
        &self,
        acc: &mut [i16; 256],
        b_king_sq: usize,
        sq: usize,
        piece_idx: usize,
        is_white_piece: bool,
    ) {
        let p_idx = if !is_white_piece {
            piece_idx
        } else {
            piece_idx + 5
        };
        let b_king_mapped = b_king_sq ^ 56;
        let sq_mapped = sq ^ 56;
        let idx = b_king_mapped * 704 + p_idx * 64 + sq_mapped;
        let weights = &self.fc1_w_transposed[idx * 256..(idx + 1) * 256];
        for (a, &w) in acc.iter_mut().zip(weights) {
            *a += w;
        }
    }
    #[inline(always)]
    fn sub_from_black(
        &self,
        acc: &mut [i16; 256],
        b_king_sq: usize,
        sq: usize,
        piece_idx: usize,
        is_white_piece: bool,
    ) {
        let p_idx = if !is_white_piece {
            piece_idx
        } else {
            piece_idx + 5
        };
        let b_king_mapped = b_king_sq ^ 56;
        let sq_mapped = sq ^ 56;
        let idx = b_king_mapped * 704 + p_idx * 64 + sq_mapped;
        let weights = &self.fc1_w_transposed[idx * 256..(idx + 1) * 256];
        for (a, &w) in acc.iter_mut().zip(weights) {
            *a -= w;
        }
    }
    fn refresh_white_acc(&self, board: &Board, white_acc: &mut [i16; 256]) {
        white_acc.copy_from_slice(&self.fc1_b);
        let w_king_sq = board.white.king.trailing_zeros() as usize;
        let mut add_pieces = |mut bb: u64, is_white_piece: bool, piece_idx: usize| {
            while bb != 0 {
                let sq = bb.trailing_zeros() as usize;
                if !(is_white_piece && piece_idx == 5) {
                    self.add_to_white(white_acc, w_king_sq, sq, piece_idx, is_white_piece);
                }
                bb &= bb - 1;
            }
        };
        add_pieces(board.white.pawn, true, 0);
        add_pieces(board.white.knight, true, 1);
        add_pieces(board.white.bishop, true, 2);
        add_pieces(board.white.rook, true, 3);
        add_pieces(board.white.queen, true, 4);
        add_pieces(board.black.pawn, false, 0);
        add_pieces(board.black.knight, false, 1);
        add_pieces(board.black.bishop, false, 2);
        add_pieces(board.black.rook, false, 3);
        add_pieces(board.black.queen, false, 4);
        add_pieces(board.black.king, false, 5);
    }
    fn refresh_black_acc(&self, board: &Board, black_acc: &mut [i16; 256]) {
        black_acc.copy_from_slice(&self.fc1_b);
        let b_king_sq = board.black.king.trailing_zeros() as usize;
        let mut add_pieces = |mut bb: u64, is_white_piece: bool, piece_idx: usize| {
            while bb != 0 {
                let sq = bb.trailing_zeros() as usize;
                if !(!is_white_piece && piece_idx == 5) {
                    self.add_to_black(black_acc, b_king_sq, sq, piece_idx, is_white_piece);
                }
                bb &= bb - 1;
            }
        };
        add_pieces(board.white.pawn, true, 0);
        add_pieces(board.white.knight, true, 1);
        add_pieces(board.white.bishop, true, 2);
        add_pieces(board.white.rook, true, 3);
        add_pieces(board.white.queen, true, 4);
        add_pieces(board.white.king, true, 5);
        add_pieces(board.black.pawn, false, 0);
        add_pieces(board.black.knight, false, 1);
        add_pieces(board.black.bishop, false, 2);
        add_pieces(board.black.rook, false, 3);
        add_pieces(board.black.queen, false, 4);
    }
    pub fn refresh_accumulator(&self, board: &Board) -> Accumulator {
        let mut acc = Accumulator {
            white: [0; 256],
            black: [0; 256],
        };
        self.refresh_white_acc(board, &mut acc.white);
        self.refresh_black_acc(board, &mut acc.black);
        acc
    }
    pub fn update_from_move(
        &self,
        board_after: &Board,
        m: &Move,
        undo: &UndoState,
        acc: &mut Accumulator,
    ) {
        let is_white_move = !board_after.white_to_move;
        let w_king_sq = board_after.white.king.trailing_zeros() as usize;
        let b_king_sq = board_after.black.king.trailing_zeros() as usize;
        let (_, added_idx) =
            piece_on_sq(board_after, m.to).expect("Piece must be at m.to after move");
        let moved_idx = match m.flag {
            MoveFlag::PromoQueen
            | MoveFlag::PromoRook
            | MoveFlag::PromoBishop
            | MoveFlag::PromoKnight
            | MoveFlag::PromoCaptureQueen
            | MoveFlag::PromoCaptureRook
            | MoveFlag::PromoCaptureBishop
            | MoveFlag::PromoCaptureKnight => 0,
            _ => added_idx,
        };
        let mut ops = [(0, 0, false, false); 4];
        let mut ops_count = 0;
        ops[ops_count] = (m.from as usize, moved_idx, is_white_move, false);
        ops_count += 1;
        ops[ops_count] = (m.to as usize, added_idx, is_white_move, true);
        ops_count += 1;
        if m.flag == MoveFlag::EnPassant {
            let cap_sq = if is_white_move { m.to - 8 } else { m.to + 8 };
            ops[ops_count] = (cap_sq as usize, 0, !is_white_move, false);
            ops_count += 1;
        } else if let Some(cap_piece_val) = undo.captured_piece {
            let cap_idx = (cap_piece_val - 1) as usize;
            ops[ops_count] = (m.to as usize, cap_idx, !is_white_move, false);
            ops_count += 1;
        }
        if m.flag == MoveFlag::KingCastle {
            let (r_from, r_to) = if is_white_move { (7, 5) } else { (63, 61) };
            ops[ops_count] = (r_from, 3, is_white_move, false);
            ops_count += 1;
            ops[ops_count] = (r_to, 3, is_white_move, true);
            ops_count += 1;
        } else if m.flag == MoveFlag::QueenCastle {
            let (r_from, r_to) = if is_white_move { (0, 3) } else { (56, 59) };
            ops[ops_count] = (r_from, 3, is_white_move, false);
            ops_count += 1;
            ops[ops_count] = (r_to, 3, is_white_move, true);
            ops_count += 1;
        }
        let refresh_white = is_white_move && moved_idx == 5;
        let refresh_black = !is_white_move && moved_idx == 5;
        if refresh_white {
            self.refresh_white_acc(board_after, &mut acc.white);
        } else {
            for i in 0..ops_count {
                let (sq, p_idx, is_w, is_add) = ops[i];
                if !(is_w && p_idx == 5) {
                    if is_add {
                        self.add_to_white(&mut acc.white, w_king_sq, sq, p_idx, is_w);
                    } else {
                        self.sub_from_white(&mut acc.white, w_king_sq, sq, p_idx, is_w);
                    }
                }
            }
        }
        if refresh_black {
            self.refresh_black_acc(board_after, &mut acc.black);
        } else {
            for i in 0..ops_count {
                let (sq, p_idx, is_w, is_add) = ops[i];
                if !(!is_w && p_idx == 5) {
                    if is_add {
                        self.add_to_black(&mut acc.black, b_king_sq, sq, p_idx, is_w);
                    } else {
                        self.sub_from_black(&mut acc.black, b_king_sq, sq, p_idx, is_w);
                    }
                }
            }
        }
    }
    pub fn evaluate_from_acc(&self, acc: &Accumulator, white_to_move: bool) -> i32 {
        let active_acc = if white_to_move {
            &acc.white
        } else {
            &acc.black
        };
        let mut l1 = [0i16; 256];
        l1.iter_mut()
            .zip(active_acc.iter())
            .for_each(|(out, &inp)| {
                *out = (inp as i32).clamp(0, self.scale) as i16;
            });
        let mut l2 = [0i16; 32];
        for (i, weight_row) in self.fc2_w.chunks_exact(256).enumerate() {
            let dot_product: i32 = l1
                .iter()
                .zip(weight_row.iter())
                .map(|(&a, &w)| (a as i32) * (w as i32))
                .sum();
            let sum = (self.fc2_b[i] as i32) * self.scale + dot_product;
            l2[i] = (sum / self.scale).clamp(0, self.scale) as i16;
        }
        let dot_product: i32 = l2
            .iter()
            .zip(self.fc3_w.iter())
            .map(|(&a, &w)| (a as i32) * (w as i32))
            .sum();
        let output = (self.fc3_b[0] as i32) * self.scale + dot_product;
        (output * 400) / (self.scale * self.scale)
    }
}
