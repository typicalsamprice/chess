use chess::movegen;
use chess::prelude::*;
use chess::{bitboard, perft};

use chess::macros::move_new;

fn main() {
    bitboard::initialize_bitboards();

    let mut state = State::new();
    let mut board = Board::new(Board::KIWIPETE, &mut state).unwrap();

    let moves = [
        move_new!(Square::A2, Square::A3),
        move_new!(Square::H3, Square::G2),
        move_new!(Square::B2, Square::B3),
    ];
    let ill = move_new!(
        Square::G2,
        Square::H1,
        MoveFlag::Promotion,
        PieceType::Bishop
    );
    //let moves = [];

    board
        .apply_moves(&mut state, &moves)
        .expect("tried to play illegal move");
    eprintln!("{board}");

    let u = perft::perft_on(&mut board, &mut state, 5 - moves.len());
    println!("Nodes searched: {u}");
}
