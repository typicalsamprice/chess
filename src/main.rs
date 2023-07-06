mod spine;
mod macros;

use spine::*;
use macros::move_new;

fn main() {
    bitboard::initialize_bitboards();

    let pin_fen_test = "8/8/5k2/2q5/3b4/2P5/2K5/8 w - - 0 1";
    let start_pos = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    let mut state = State::new();
    let mut b = Board::new(start_pos, &mut state).unwrap();

    let d = 3;
    let moves: Vec<Move> = vec![];
    let moves = vec![
        move_new!(Square::H2, Square::H4),
        move_new!(Square::G7, Square::G5)];
    b.apply_moves(&mut state, &moves).unwrap();
    let u = perft::perft_on(&mut b, &mut state, d - moves.len());
    println!("Nodes: {u}");
}
