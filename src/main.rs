mod spine;
mod macros;

use spine::*;

fn main() {
    bitboard::initialize_bitboards();
    let pin_fen_test = "8/8/5k2/2q5/3b4/2P5/2K5/8 w - - 0 1";
    let start_pos = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    let mut state = State::new();
    let mut b = Board::new(start_pos, &mut state).unwrap();
    let m = Move::new(Square::E2, Square::E4, MoveFlag::Normal, PieceType::Pawn);
    let m2 = Move::new(Square::E7, Square::E5, MoveFlag::Normal, PieceType::Pawn);
    b.do_move(&mut state, m);
    b.do_move(&mut state, m2);
    b.undo_move(&mut state, m2);
    b.undo_move(&mut state, m);
    println!("{b}");
}
