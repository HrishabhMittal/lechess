use crate::board::*;
use std::ops::{Deref, DerefMut};
pub struct MoveList {
    pub moves: [Move; 256],
    pub count: usize,
}
impl MoveList {
    pub fn new() -> Self {
        MoveList {
            moves: [Move {
                from: 0,
                to: 0,
                flag: MoveFlag::Quiet,
            }; 256],
            count: 0,
        }
    }
    pub fn push(&mut self, m: Move) {
        assert!(self.count < 256, "MoveList overflow!");
        self.moves[self.count] = m;
        self.count += 1;
    }
}
impl Deref for MoveList {
    type Target = [Move];

    fn deref(&self) -> &[Move] {
        &self.moves[..self.count]
    }
}
impl DerefMut for MoveList {
    fn deref_mut(&mut self) -> &mut [Move] {
        &mut self.moves[..self.count]
    }
}
