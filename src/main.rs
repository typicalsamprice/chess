use chess::movegen;
use chess::prelude::*;
use chess::{bitboard, perft};

use chess::macros::move_new;

fn main() {
    bitboard::initialize_bitboards();

    let mut state = State::new();
    let mut board = Board::new(Board::STARTPOS, &mut state).unwrap();

    let moves = [
        move_new!(Square::D2, Square::D4),
        move_new!(Square::E7, Square::E5),
        move_new!(Square::E1, Square::D2),
        move_new!(Square::D8, Square::E7),
        move_new!(Square::D2, Square::E3),
        move_new!(Square::E5, Square::D4),
    ];
    let moves = [];

    board
        .apply_moves(&mut state, &moves)
        .expect("tried to play illegal move");

    println!("{board}");
    println!(
        "Checkers: {}",
        state
            .checkers()
            .map(|x| format!("{x}"))
            .collect::<Vec<_>>()
            .join(" ")
    );
    let u = perft::perft_on(&mut board, &mut state, 7 - moves.len());
    println!("Nodes searched: {u}");
}
