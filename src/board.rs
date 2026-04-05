use core::fmt;

pub struct Peices {
    pawn: u64,
    knight: u64,
    bishop: u64,
    rook: u64,
    queen: u64,
    king: u64,
}

impl Peices {
    pub fn new_white() -> Self {
        Peices {
            pawn: 0xFF00,
            knight: 0b01000010,
            bishop: 0b00100100,
            rook: 0b10000001,
            queen: 0b00010000,
            king: 0b00001000,
        }
    }
    pub fn new_black() -> Self {
        Peices {
            pawn: 0xFF << 48,
            knight: 0b01000010 << 56,
            bishop: 0b00100100 << 56,
            rook: 0b10000001 << 56,
            queen: 0b00010000 << 56,
            king: 0b00001000 << 56,
        }
    }
    pub fn find(&self, i: u32, j: u32) -> Option<PeiceType> {
        if i > 7 || j > 7 {
            None
        } else {
            let pos: u64 = ((i << 3) + j).into();
            let mask = 1 << pos;
            if self.pawn & mask != 0 {
                Some(PeiceType::Pawn)
            } else if self.king & mask != 0 {
                Some(PeiceType::King)
            } else if self.queen & mask != 0 {
                Some(PeiceType::Queen)
            } else if self.rook & mask != 0 {
                Some(PeiceType::Rook)
            } else if self.knight & mask != 0 {
                Some(PeiceType::Knight)
            } else if self.bishop & mask != 0 {
                Some(PeiceType::Bishop)
            } else {
                None
            }
        }
    }
}
pub enum PeiceType {
    Pawn,
    Bishop,
    Knight,
    Rook,
    Queen,
    King,
}
pub enum Peice {
    White(PeiceType),
    Black(PeiceType),
}
pub struct Board {
    white: Peices,
    black: Peices,
}

impl Board {
    pub fn new() -> Self {
        Board {
            white: Peices::new_white(),
            black: Peices::new_black(),
        }
    }
    pub fn find(&self, i: u32, j: u32) -> Option<Peice> {
        match (self.white.find(i, j), self.black.find(i, j)) {
            (Some(p), None) => Some(Peice::White(p)),
            (None, Some(p)) => Some(Peice::Black(p)),
            _ => None,
        }
    }
    pub fn get_display(&self, i: u32, j: u32) -> u8 {
        let (capital, init, ch);
        match self.find(i, j) {
            Some(v) => match v {
                Peice::White(w) => {
                    capital = false;
                    init = true;
                    ch = w;
                }
                Peice::Black(b) => {
                    capital = true;
                    init = true;
                    ch = b;
                }
            },
            None => {
                capital = false;
                init = false;
                ch = PeiceType::Knight;
            }
        };
        let mut ch: u8 = match ch {
            PeiceType::Rook => b'r',
            PeiceType::Queen => b'q',
            PeiceType::King => b'k',
            PeiceType::Knight => b'n',
            PeiceType::Pawn => b'p',
            PeiceType::Bishop => b'b',
        };
        if !init {
            ch = b'.';
        } else if capital {
            ch -= b'a' - b'A';
        }
        ch
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "  a b c d e f g h")?;
        for i in 0..8 {
            write!(f, "{} ", 8 - i)?;
            for j in 0..8 {
                write!(f, "{} ", self.get_display(i, j) as char)?;
            }
            writeln!(f, "")?;
        }
        Ok(())
    }
}
