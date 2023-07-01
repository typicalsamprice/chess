mod spine;
mod macros;

use spine::*;
use std::borrow::BorrowMut;

fn main() {
    initialize_bitboards();

    let mut state = Box::new(State::new(None));
    let b = Board::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", state.borrow_mut()).unwrap();
    println!("{:?}", b.get_piece(Square::H8));
    println!("{b}");
}
