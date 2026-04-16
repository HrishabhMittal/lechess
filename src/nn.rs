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
    pub white: [i16; 1024],
    pub black: [i16; 1024],
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

        let mut fc1_w_transposed = vec![0i16; 772 * 1024];
        for i in 0..1024 {
            for j in 0..772 {
                fc1_w_transposed[j * 1024 + i] = weights.fc1_w[i][j];
            }
        }

        let mut fc2_w = vec![0i16; 128 * 1024];
        for i in 0..128 {
            for j in 0..1024 {
                fc2_w[i * 1024 + j] = weights.fc2_w[i][j];
            }
        }

        let mut fc3_w = vec![0i16; 128];
        for i in 0..128 {
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

    fn new_accumulator(&self) -> Accumulator {
        let mut acc = Accumulator {
            white: [0; 1024],
            black: [0; 1024],
        };

        acc.white.copy_from_slice(&self.fc1_b);
        acc.black.copy_from_slice(&self.fc1_b);
        acc
    }

    pub fn toggle_piece(
        &self,
        acc: &mut Accumulator,
        sq: u8,
        is_white: bool,
        piece_idx: usize,
        add: bool,
    ) {
        let w_color_offset = if is_white { 0 } else { 6 };
        let w_idx = (w_color_offset + piece_idx) * 64 + (sq as usize);

        let b_color_offset = if !is_white { 0 } else { 6 };
        let b_idx = (b_color_offset + piece_idx) * 64 + ((sq as usize) ^ 56);

        let w_weights = &self.fc1_w_transposed[w_idx * 1024..(w_idx + 1) * 1024];
        let b_weights = &self.fc1_w_transposed[b_idx * 1024..(b_idx + 1) * 1024];

        if add {
            acc.white
                .iter_mut()
                .zip(w_weights)
                .for_each(|(a, &w)| *a += w);
            acc.black
                .iter_mut()
                .zip(b_weights)
                .for_each(|(a, &w)| *a += w);
        } else {
            acc.white
                .iter_mut()
                .zip(w_weights)
                .for_each(|(a, &w)| *a -= w);
            acc.black
                .iter_mut()
                .zip(b_weights)
                .for_each(|(a, &w)| *a -= w);
        }
    }

    pub fn refresh_accumulator(&self, board: &Board) -> Accumulator {
        let mut acc = self.new_accumulator();

        let mut add_pieces = |mut bb: u64, is_white_piece: bool, piece_idx: usize| {
            while bb != 0 {
                let sq = bb.trailing_zeros() as u8;
                self.toggle_piece(&mut acc, sq, is_white_piece, piece_idx, true);
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
        add_pieces(board.black.king, false, 5);

        let cr = board.castling_rights;
        let mut apply_cr = |mask: u8, w_idx: usize, b_idx: usize| {
            if (cr & mask) != 0 {
                let w_weights = &self.fc1_w_transposed[w_idx * 1024..(w_idx + 1) * 1024];
                let b_weights = &self.fc1_w_transposed[b_idx * 1024..(b_idx + 1) * 1024];
                acc.white
                    .iter_mut()
                    .zip(w_weights)
                    .for_each(|(a, &w)| *a += w);
                acc.black
                    .iter_mut()
                    .zip(b_weights)
                    .for_each(|(a, &w)| *a += w);
            }
        };
        apply_cr(1, 768, 770);
        apply_cr(2, 769, 771);
        apply_cr(4, 770, 768);
        apply_cr(8, 771, 769);

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

        let (added_is_white, added_idx) =
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

        self.toggle_piece(acc, m.from, is_white_move, moved_idx, false);
        self.toggle_piece(acc, m.to, added_is_white, added_idx, true);

        if m.flag == MoveFlag::EnPassant {
            let cap_sq = if is_white_move { m.to - 8 } else { m.to + 8 };
            self.toggle_piece(acc, cap_sq, !is_white_move, 0, false);
        } else if let Some(cap_piece_val) = undo.captured_piece {
            let cap_idx = (cap_piece_val - 1) as usize;
            self.toggle_piece(acc, m.to, !is_white_move, cap_idx, false);
        }

        if m.flag == MoveFlag::KingCastle {
            let (r_from, r_to) = if is_white_move { (7, 5) } else { (63, 61) };
            self.toggle_piece(acc, r_from, is_white_move, 3, false);
            self.toggle_piece(acc, r_to, is_white_move, 3, true);
        } else if m.flag == MoveFlag::QueenCastle {
            let (r_from, r_to) = if is_white_move { (0, 3) } else { (56, 59) };
            self.toggle_piece(acc, r_from, is_white_move, 3, false);
            self.toggle_piece(acc, r_to, is_white_move, 3, true);
        }

        let old_cr = undo.castling_rights;
        let new_cr = board_after.castling_rights;
        if old_cr != new_cr {
            let changed = old_cr ^ new_cr;
            let mut apply_cr = |mask: u8, w_idx: usize, b_idx: usize| {
                if (changed & mask) != 0 {
                    let w_weights = &self.fc1_w_transposed[w_idx * 1024..(w_idx + 1) * 1024];
                    let b_weights = &self.fc1_w_transposed[b_idx * 1024..(b_idx + 1) * 1024];
                    if (new_cr & mask) != 0 {
                        acc.white
                            .iter_mut()
                            .zip(w_weights)
                            .for_each(|(a, &w)| *a += w);
                        acc.black
                            .iter_mut()
                            .zip(b_weights)
                            .for_each(|(a, &w)| *a += w);
                    } else {
                        acc.white
                            .iter_mut()
                            .zip(w_weights)
                            .for_each(|(a, &w)| *a -= w);
                        acc.black
                            .iter_mut()
                            .zip(b_weights)
                            .for_each(|(a, &w)| *a -= w);
                    }
                }
            };
            apply_cr(1, 768, 770);
            apply_cr(2, 769, 771);
            apply_cr(4, 770, 768);
            apply_cr(8, 771, 769);
        }
    }

    pub fn evaluate_from_acc(&self, acc: &Accumulator, white_to_move: bool) -> i32 {
        let active_acc = if white_to_move {
            &acc.white
        } else {
            &acc.black
        };

        let mut l1 = [0i16; 1024];
        l1.iter_mut()
            .zip(active_acc.iter())
            .for_each(|(out, &inp)| {
                *out = (inp as i32).clamp(0, self.scale) as i16;
            });

        let mut l2 = [0i16; 128];

        for (i, weight_row) in self.fc2_w.chunks_exact(1024).enumerate() {
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
