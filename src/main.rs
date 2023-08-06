use chess::{bitboard, perft};

use chess::move_new;
use chess::prelude::*;

fn main() {
    bitboard::initialize_bitboards();
    //chess::print_comp_flags();

    let mut s = State::new();
    let mut b = Board::new(Board::STARTPOS, &mut s).unwrap();

    let moves = [move_new!("d2d4")];

    b.apply_moves(&mut s, &moves).unwrap();
    //println!("Fen: {}", Board::KIWIPETE);
    //println!("{b}");
    let u = perft::perft_on(&mut b, &mut s, 7 - moves.len());
    println!("Nodes searched: {u}");
}
