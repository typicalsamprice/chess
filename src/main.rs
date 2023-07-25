use chess::movegen;
use chess::prelude::*;
use chess::{bitboard, perft};

use chess::macros::move_new;

fn main() {
    bitboard::initialize_bitboards();

    let mut state = State::new();
    let mut board = Board::new(Board::STARTPOS, &mut state).unwrap();

    let moves = [
        move_new!(Square::E2, Square::E3),
        move_new!(Square::E7, Square::E6),
        move_new!(Square::D2, Square::D4),
        move_new!(Square::F8, Square::B4),
    ];
    board.apply_moves(&mut state, &moves).unwrap();
    println!("{board}");

    perft::perft_on(&mut board, &mut state, 5 - moves.len());
}
