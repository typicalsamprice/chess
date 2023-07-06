mod spine;
mod macros;

use spine::*;

fn main() {
    bitboard::initialize_bitboards();

    let pin_fen_test = "8/8/5k2/2q5/3b4/2P5/2K5/8 w - - 0 1";
    let start_pos = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    let mut state = State::new();
    let mut _b = Board::new(start_pos, &mut state).unwrap();

    let u = perft::perft(2);
    println!("Nodes: {u}");
}
