use core::fmt;
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MoveFlag {
    Quiet,
    DoublePawnPush,
    Capture,
    EnPassant,
    KingCastle,
    QueenCastle,
    PromoKnight,
    PromoBishop,
    PromoRook,
    PromoQueen,
    PromoCaptureKnight,
    PromoCaptureBishop,
    PromoCaptureRook,
    PromoCaptureQueen,
}
#[derive(Debug, Clone, Copy)]
pub struct Move {
    pub from: u8,
    pub to: u8,
    pub flag: MoveFlag,
}
#[derive(Clone, Copy)]
pub struct Pieces {
    pub pawn: u64,
    pub knight: u64,
    pub bishop: u64,
    pub rook: u64,
    pub queen: u64,
    pub king: u64,
}
impl Pieces {
    pub fn new_white() -> Self {
        Pieces {
            pawn: 0x0000_0000_0000_FF00,
            knight: 0x0000_0000_0000_0042,
            bishop: 0x0000_0000_0000_0024,
            rook: 0x0000_0000_0000_0081,
            queen: 0x0000_0000_0000_0008,
            king: 0x0000_0000_0000_0010,
        }
    }
    pub fn new_black() -> Self {
        Pieces {
            pawn: 0x00FF_0000_0000_0000,
            knight: 0x4200_0000_0000_0000,
            bishop: 0x2400_0000_0000_0000,
            rook: 0x8100_0000_0000_0000,
            queen: 0x0800_0000_0000_0000,
            king: 0x1000_0000_0000_0000,
        }
    }
    pub fn all(&self) -> u64 {
        self.pawn | self.knight | self.bishop | self.rook | self.queen | self.king
    }
}
#[derive(Clone)]
pub struct Board {
    pub white: Pieces,
    pub black: Pieces,
    pub en_passant_target: Option<u8>,
    pub castling_rights: u8,
}
impl Board {
    pub fn new() -> Self {
        Board {
            white: Pieces::new_white(),
            black: Pieces::new_black(),
            en_passant_target: None,
            castling_rights: 0b1111,
        }
    }
    pub fn is_square_attacked(&self, sq: u8, by_white: bool) -> bool {
        let enemy = if by_white { &self.white } else { &self.black };
        let all_occ = self.white.all() | self.black.all();
        let bb = 1u64 << sq;
        let pawn_attacks = if by_white {
            let down_right = (bb >> 7) & 0xfefefefefefefefe;
            let down_left = (bb >> 9) & 0x7f7f7f7f7f7f7f7f;
            down_right | down_left
        } else {
            let up_left = (bb << 7) & 0x7f7f7f7f7f7f7f7f;
            let up_right = (bb << 9) & 0xfefefefefefefefe;
            up_left | up_right
        };
        if (pawn_attacks & enemy.pawn) != 0 {
            return true;
        }
        if (knight_attacks(sq) & enemy.knight) != 0 {
            return true;
        }
        if (king_attacks(sq) & enemy.king) != 0 {
            return true;
        }
        if (get_bishop_attacks(sq, all_occ) & (enemy.bishop | enemy.queen)) != 0 {
            return true;
        }
        if (get_rook_attacks(sq, all_occ) & (enemy.rook | enemy.queen)) != 0 {
            return true;
        }
        false
    }
    pub fn is_in_check(&self, is_white: bool) -> bool {
        let king_bb = if is_white {
            self.white.king
        } else {
            self.black.king
        };
        if king_bb == 0 {
            return false;
        }
        let king_sq = king_bb.trailing_zeros() as u8;
        self.is_square_attacked(king_sq, !is_white)
    }
    pub fn make_move(&self, m: &Move, is_white: bool) -> Board {
        let mut next = self.clone();
        if m.from == 4 {
            next.castling_rights &= !0b0011;
        }
        if m.from == 60 {
            next.castling_rights &= !0b1100;
        }
        if m.from == 7 || m.to == 7 {
            next.castling_rights &= !0b0001;
        }
        if m.from == 0 || m.to == 0 {
            next.castling_rights &= !0b0010;
        }
        if m.from == 63 || m.to == 63 {
            next.castling_rights &= !0b0100;
        }
        if m.from == 56 || m.to == 56 {
            next.castling_rights &= !0b1000;
        }
        let from_bb = 1u64 << m.from;
        let to_bb = 1u64 << m.to;
        let move_mask = from_bb | to_bb;
        let (friendly, enemy) = if is_white {
            (&mut next.white, &mut next.black)
        } else {
            (&mut next.black, &mut next.white)
        };
        if m.flag == MoveFlag::Capture
            || m.flag == MoveFlag::PromoCaptureQueen
            || m.flag == MoveFlag::PromoCaptureRook
            || m.flag == MoveFlag::PromoCaptureBishop
            || m.flag == MoveFlag::PromoCaptureKnight
        {
            enemy.pawn &= !to_bb;
            enemy.knight &= !to_bb;
            enemy.bishop &= !to_bb;
            enemy.rook &= !to_bb;
            enemy.queen &= !to_bb;
        } else if m.flag == MoveFlag::EnPassant {
            let cap_sq = if is_white { m.to - 8 } else { m.to + 8 };
            enemy.pawn &= !(1u64 << cap_sq);
        }
        if (friendly.pawn & from_bb) != 0 {
            friendly.pawn ^= move_mask;
            if m.flag == MoveFlag::PromoQueen || m.flag == MoveFlag::PromoCaptureQueen {
                friendly.pawn &= !to_bb;
                friendly.queen |= to_bb;
            } else if m.flag == MoveFlag::PromoRook || m.flag == MoveFlag::PromoCaptureRook {
                friendly.pawn &= !to_bb;
                friendly.rook |= to_bb;
            } else if m.flag == MoveFlag::PromoBishop || m.flag == MoveFlag::PromoCaptureBishop {
                friendly.pawn &= !to_bb;
                friendly.bishop |= to_bb;
            } else if m.flag == MoveFlag::PromoKnight || m.flag == MoveFlag::PromoCaptureKnight {
                friendly.pawn &= !to_bb;
                friendly.knight |= to_bb;
            }
        } else if (friendly.knight & from_bb) != 0 {
            friendly.knight ^= move_mask;
        } else if (friendly.bishop & from_bb) != 0 {
            friendly.bishop ^= move_mask;
        } else if (friendly.rook & from_bb) != 0 {
            friendly.rook ^= move_mask;
        } else if (friendly.queen & from_bb) != 0 {
            friendly.queen ^= move_mask;
        } else if (friendly.king & from_bb) != 0 {
            friendly.king ^= move_mask;
            if m.flag == MoveFlag::KingCastle {
                let (r_from, r_to) = if is_white { (7, 5) } else { (63, 61) };
                friendly.rook ^= (1u64 << r_from) | (1u64 << r_to);
            } else if m.flag == MoveFlag::QueenCastle {
                let (r_from, r_to) = if is_white { (0, 3) } else { (56, 59) };
                friendly.rook ^= (1u64 << r_from) | (1u64 << r_to);
            }
        }
        next.en_passant_target = None;
        if m.flag == MoveFlag::DoublePawnPush {
            next.en_passant_target = Some(if is_white { m.to - 8 } else { m.to + 8 });
        }
        next
    }
    pub fn get_occupancy(&self) -> (u64, u64, u64) {
        let w = self.white.all();
        let b = self.black.all();
        (w, b, w | b)
    }
    pub fn generate_legal_moves(&self, is_white: bool) -> Vec<Move> {
        let pseudo_moves = self.generate_pseudo_legal_moves(is_white);
        let mut legal_moves = Vec::with_capacity(pseudo_moves.len());
        for m in pseudo_moves {
            let next_state = self.make_move(&m, is_white);
            if !next_state.is_in_check(is_white) {
                legal_moves.push(m);
            }
        }
        legal_moves
    }
    pub fn generate_pseudo_legal_moves(&self, is_white: bool) -> Vec<Move> {
        let mut moves = Vec::with_capacity(128);
        let (friendly, _enemy) = if is_white {
            (&self.white, &self.black)
        } else {
            (&self.black, &self.white)
        };
        let (w_occ, b_occ, all_occ) = self.get_occupancy();
        let (friend_occ, enemy_occ) = if is_white {
            (w_occ, b_occ)
        } else {
            (b_occ, w_occ)
        };
        let empty = !all_occ;
        self.generate_pawn_moves(&mut moves, friendly.pawn, empty, enemy_occ, is_white);
        self.generate_knight_moves(&mut moves, friendly.knight, friend_occ, enemy_occ);
        self.generate_king_moves(&mut moves, friendly.king, friend_occ, enemy_occ, is_white);
        self.generate_sliding_moves(
            &mut moves,
            friendly.bishop,
            friendly.rook,
            friendly.queen,
            friend_occ,
            enemy_occ,
            all_occ,
        );
        moves
    }
    fn generate_pawn_moves(
        &self,
        moves: &mut Vec<Move>,
        pawns: u64,
        empty: u64,
        enemy_occ: u64,
        is_white: bool,
    ) {
        let (push_shift, rank3, promo_rank) = if is_white {
            (8, 0x0000000000FF0000, 0xFF00000000000000)
        } else {
            (-8, 0x0000FF0000000000, 0x00000000000000FF)
        };
        let single_pushes = shift_bb(pawns, push_shift) & empty;
        let double_pushes = shift_bb(single_pushes & rank3, push_shift) & empty;
        let attacks_left = shift_bb(pawns, push_shift - 1) & enemy_occ & 0x7f7f7f7f7f7f7f7f;
        let attacks_right = shift_bb(pawns, push_shift + 1) & enemy_occ & 0xfefefefefefefefe;
        self.extract_pawn_moves(
            moves,
            single_pushes,
            -push_shift,
            MoveFlag::Quiet,
            promo_rank,
        );
        self.extract_pawn_moves(
            moves,
            double_pushes,
            -push_shift * 2,
            MoveFlag::DoublePawnPush,
            0,
        );
        self.extract_pawn_moves(
            moves,
            attacks_left,
            -(push_shift - 1),
            MoveFlag::Capture,
            promo_rank,
        );
        self.extract_pawn_moves(
            moves,
            attacks_right,
            -(push_shift + 1),
            MoveFlag::Capture,
            promo_rank,
        );
        if let Some(ep_sq) = self.en_passant_target {
            let ep_bb = 1u64 << ep_sq;
            let ep_left = shift_bb(pawns, push_shift - 1) & ep_bb & 0x7f7f7f7f7f7f7f7f;
            let ep_right = shift_bb(pawns, push_shift + 1) & ep_bb & 0xfefefefefefefefe;
            if ep_left != 0 {
                moves.push(Move {
                    from: (ep_sq as i8 - (push_shift - 1)) as u8,
                    to: ep_sq,
                    flag: MoveFlag::EnPassant,
                });
            }
            if ep_right != 0 {
                moves.push(Move {
                    from: (ep_sq as i8 - (push_shift + 1)) as u8,
                    to: ep_sq,
                    flag: MoveFlag::EnPassant,
                });
            }
        }
    }
    fn extract_pawn_moves(
        &self,
        moves: &mut Vec<Move>,
        mut bb: u64,
        offset: i8,
        default_flag: MoveFlag,
        promo_rank: u64,
    ) {
        while bb != 0 {
            let to = bb.trailing_zeros() as u8;
            let from = (to as i8 + offset) as u8;
            let to_bb = 1u64 << to;
            if (to_bb & promo_rank) != 0 {
                let is_cap = default_flag == MoveFlag::Capture;
                moves.push(Move {
                    from,
                    to,
                    flag: if is_cap {
                        MoveFlag::PromoCaptureQueen
                    } else {
                        MoveFlag::PromoQueen
                    },
                });
                moves.push(Move {
                    from,
                    to,
                    flag: if is_cap {
                        MoveFlag::PromoCaptureRook
                    } else {
                        MoveFlag::PromoRook
                    },
                });
                moves.push(Move {
                    from,
                    to,
                    flag: if is_cap {
                        MoveFlag::PromoCaptureBishop
                    } else {
                        MoveFlag::PromoBishop
                    },
                });
                moves.push(Move {
                    from,
                    to,
                    flag: if is_cap {
                        MoveFlag::PromoCaptureKnight
                    } else {
                        MoveFlag::PromoKnight
                    },
                });
            } else {
                moves.push(Move {
                    from,
                    to,
                    flag: default_flag,
                });
            }
            bb &= bb - 1;
        }
    }
    fn generate_king_moves(
        &self,
        moves: &mut Vec<Move>,
        king: u64,
        friend_occ: u64,
        enemy_occ: u64,
        is_white: bool,
    ) {
        let from_sq = king.trailing_zeros() as u8;
        let k_moves = king_attacks(from_sq) & !friend_occ;
        self.extract_moves(moves, k_moves & !enemy_occ, from_sq, MoveFlag::Quiet);
        self.extract_moves(moves, k_moves & enemy_occ, from_sq, MoveFlag::Capture);
        let occ = friend_occ | enemy_occ;
        if is_white {
            if (self.castling_rights & 1) != 0 && (occ & 0x0000000000000060) == 0 {
                if !self.is_square_attacked(4, false)
                    && !self.is_square_attacked(5, false)
                    && !self.is_square_attacked(6, false)
                {
                    moves.push(Move {
                        from: 4,
                        to: 6,
                        flag: MoveFlag::KingCastle,
                    });
                }
            }
            if (self.castling_rights & 2) != 0 && (occ & 0x000000000000000E) == 0 {
                if !self.is_square_attacked(4, false)
                    && !self.is_square_attacked(3, false)
                    && !self.is_square_attacked(2, false)
                {
                    moves.push(Move {
                        from: 4,
                        to: 2,
                        flag: MoveFlag::QueenCastle,
                    });
                }
            }
        } else {
            if (self.castling_rights & 4) != 0 && (occ & 0x6000000000000000) == 0 {
                if !self.is_square_attacked(60, true)
                    && !self.is_square_attacked(61, true)
                    && !self.is_square_attacked(62, true)
                {
                    moves.push(Move {
                        from: 60,
                        to: 62,
                        flag: MoveFlag::KingCastle,
                    });
                }
            }
            if (self.castling_rights & 8) != 0 && (occ & 0x0E00000000000000) == 0 {
                if !self.is_square_attacked(60, true)
                    && !self.is_square_attacked(59, true)
                    && !self.is_square_attacked(58, true)
                {
                    moves.push(Move {
                        from: 60,
                        to: 58,
                        flag: MoveFlag::QueenCastle,
                    });
                }
            }
        }
    }
    fn generate_knight_moves(
        &self,
        moves: &mut Vec<Move>,
        mut knights: u64,
        friend_occ: u64,
        enemy_occ: u64,
    ) {
        while knights != 0 {
            let from_sq = knights.trailing_zeros() as u8;
            let attacks = knight_attacks(from_sq) & !friend_occ;
            self.extract_moves(moves, attacks & !enemy_occ, from_sq, MoveFlag::Quiet);
            self.extract_moves(moves, attacks & enemy_occ, from_sq, MoveFlag::Capture);
            knights &= knights - 1;
        }
    }
    fn generate_sliding_moves(
        &self,
        moves: &mut Vec<Move>,
        bishops: u64,
        rooks: u64,
        queens: u64,
        friend: u64,
        enemy: u64,
        occ: u64,
    ) {
        let b_q = bishops | queens;
        let r_q = rooks | queens;
        let mut temp_bq = b_q;
        while temp_bq != 0 {
            let sq = temp_bq.trailing_zeros() as u8;
            let attacks = get_bishop_attacks(sq, occ) & !friend;
            self.extract_moves(moves, attacks & !enemy, sq, MoveFlag::Quiet);
            self.extract_moves(moves, attacks & enemy, sq, MoveFlag::Capture);
            temp_bq &= temp_bq - 1;
        }
        let mut temp_rq = r_q;
        while temp_rq != 0 {
            let sq = temp_rq.trailing_zeros() as u8;
            let attacks = get_rook_attacks(sq, occ) & !friend;
            self.extract_moves(moves, attacks & !enemy, sq, MoveFlag::Quiet);
            self.extract_moves(moves, attacks & enemy, sq, MoveFlag::Capture);
            temp_rq &= temp_rq - 1;
        }
    }
    fn extract_moves(&self, moves: &mut Vec<Move>, mut bb: u64, from: u8, flag: MoveFlag) {
        while bb != 0 {
            let to = bb.trailing_zeros() as u8;
            moves.push(Move { from, to, flag });
            bb &= bb - 1;
        }
    }
}
fn shift_bb(bb: u64, shift: i8) -> u64 {
    if shift > 0 {
        bb << shift
    } else {
        bb >> (-shift)
    }
}
fn knight_attacks(sq: u8) -> u64 {
    let bb = 1u64 << sq;
    let l1 = (bb >> 1) & 0x7f7f7f7f7f7f7f7f;
    let l2 = (bb >> 2) & 0x3f3f3f3f3f3f3f3f;
    let r1 = (bb << 1) & 0xfefefefefefefefe;
    let r2 = (bb << 2) & 0xfcfcfcfcfcfcfcfc;
    (l1 << 16)
        | (l1 >> 16)
        | (l2 << 8)
        | (l2 >> 8)
        | (r1 << 16)
        | (r1 >> 16)
        | (r2 << 8)
        | (r2 >> 8)
}
fn king_attacks(sq: u8) -> u64 {
    let bb = 1u64 << sq;
    let attacks = shift_bb(bb, 8) | shift_bb(bb, -8);
    let left = shift_bb(bb, -1) & 0x7f7f7f7f7f7f7f7f;
    let right = shift_bb(bb, 1) & 0xfefefefefefefefe;
    attacks
        | left
        | right
        | shift_bb(left, 8)
        | shift_bb(left, -8)
        | shift_bb(right, 8)
        | shift_bb(right, -8)
}
fn get_bishop_attacks(sq: u8, occ: u64) -> u64 {
    let mut attacks = 0;
    let dirs = [7, 9, -7, -9];
    for &dir in &dirs {
        let mut curr_sq = sq as i8;
        loop {
            if (dir == 7 || dir == -9) && (curr_sq % 8 == 0) {
                break;
            }
            if (dir == 9 || dir == -7) && (curr_sq % 8 == 7) {
                break;
            }
            curr_sq += dir;
            if curr_sq < 0 || curr_sq > 63 {
                break;
            }
            attacks |= 1u64 << curr_sq;
            if (occ & (1u64 << curr_sq)) != 0 {
                break;
            }
        }
    }
    attacks
}
fn get_rook_attacks(sq: u8, occ: u64) -> u64 {
    let mut attacks = 0;
    let dirs = [8, -8, 1, -1];
    for &dir in &dirs {
        let mut curr_sq = sq as i8;
        loop {
            if dir == 1 && (curr_sq % 8 == 7) {
                break;
            }
            if dir == -1 && (curr_sq % 8 == 0) {
                break;
            }
            curr_sq += dir;
            if curr_sq < 0 || curr_sq > 63 {
                break;
            }
            attacks |= 1u64 << curr_sq;
            if (occ & (1u64 << curr_sq)) != 0 {
                break;
            }
        }
    }
    attacks
}
impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for rank in (0..8).rev() {
            for file in 0..8 {
                let sq = rank * 8 + file;
                let mask = 1u64 << sq;
                let c = if (self.white.pawn & mask) != 0 {
                    'P'
                } else if (self.black.pawn & mask) != 0 {
                    'p'
                } else if (self.white.knight & mask) != 0 {
                    'N'
                } else if (self.black.knight & mask) != 0 {
                    'n'
                } else if (self.white.bishop & mask) != 0 {
                    'B'
                } else if (self.black.bishop & mask) != 0 {
                    'b'
                } else if (self.white.rook & mask) != 0 {
                    'R'
                } else if (self.black.rook & mask) != 0 {
                    'r'
                } else if (self.white.queen & mask) != 0 {
                    'Q'
                } else if (self.black.queen & mask) != 0 {
                    'q'
                } else if (self.white.king & mask) != 0 {
                    'K'
                } else if (self.black.king & mask) != 0 {
                    'k'
                } else {
                    '.'
                };
                write!(f, "{} ", c)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
