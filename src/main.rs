use chess::{bitboard, perft};

use chess::move_new;
use chess::prelude::*;

fn main() {
    bitboard::initialize_bitboards();
    //chess::print_comp_flags();

    let fen = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -";
    let mut s = State::new();
    let mut b = Board::new(fen, &mut s).unwrap();

    let moves = [
        move_new!("b4b1"),
        move_new!("f4f3"),
        move_new!("a5b4"),
        move_new!("c7c5"),
    ];

    b.apply_moves(&mut s, &moves).unwrap();
    println!("{b}");
    let u = perft::perft_on(&mut b, &mut s, 5 - moves.len());
    println!("Nodes searched: {u}");
}
