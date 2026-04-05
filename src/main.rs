mod board;
use board::*;

fn main() {
    let board = Board::new();
    println!("le chess moteur");
    println!("test: ");
    println!("at a1: {}",board.get_display(Square::from("a1")) as char);
    println!("board:\n{}",board);
}
