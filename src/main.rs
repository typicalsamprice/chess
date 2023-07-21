use chess::movegen;
use chess::prelude::*;
use chess::{bitboard, perft};

use chess::macros::move_new;

fn main() {
    bitboard::initialize_bitboards();

    let mut state = State::new();
    let mut _board = Board::new(Board::STARTPOS, &mut state).unwrap();

    let pin_test_fen = "rnbqkbnr/pppppppp/8/8/Q1P5/8/PP1PPPPP/RNB1KBNR b KQkq - 0 1";
    let mut board = Board::new(pin_test_fen, &mut state).unwrap();
    println!("{board}");
    let m = move_new!(Square::D7, Square::D6);
    board.do_move(&mut state, m).unwrap();
}
