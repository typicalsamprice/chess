use chess::{bitboard, perft};

fn main() {
    bitboard::initialize_bitboards();
    let u = perft::perft(4);
    println!("Nodes searched: {u}");
}
