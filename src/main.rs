use chess::{bitboard, perft};

use chess::move_new;
use chess::prelude::*;

fn main() {
    bitboard::initialize_bitboards();
    //chess::print_comp_flags();

    let mut s = State::new();
    let mut b = Board::new(Board::KIWIPETE, &mut s).unwrap();

    let moves = [
        move_new!(Square::A1, Square::B1),
        move_new!(Square::H3, Square::G2),
        move_new!(Square::H2, Square::H3),
        //move_new!(Square::E2, Square::E4),
        //move_new!(Square::E7, Square::E5),
        //move_new!(Square::D2, Square::D4),
        //move_new!(Square::F8, Square::B4),
    ];

    b.apply_moves(&mut s, &moves).unwrap();
    //println!("Fen: {}", Board::KIWIPETE);
    //println!("{b}");
    let u = perft::perft_on(&mut b, &mut s, 5 - moves.len());
    println!("Nodes searched: {u}");
}
