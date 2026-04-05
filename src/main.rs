struct Peices {
    pawn: u64,
    knight: u64,
    bishop: u64,
    rook: u64,
    queen: u64,
    king: u64,
}

impl Peices {
    fn new_white() -> Self {
        todo!();
    }
    fn new_black() -> Self {
        todo!();
    }
}
struct Board {
    white: Peices,
    black: Peices,
}

impl Board {
    fn new() -> Self {
        todo!();
    }
}

fn main() {
    let _board = Board::new();
    println!("le chess moteur");
}
