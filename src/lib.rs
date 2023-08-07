#[warn(missing_debug_implementations)]
#[warn(unreachable_pub)]
#[warn(trivial_casts, trivial_numeric_casts)]
#[warn(unused_results)]
#[warn(missing_docs)]

/// semver of the program
pub const VERSION: &'static str = "0.0.1";

mod spine;

pub use spine::movegen;
pub use spine::perft;
pub use spine::piece_attacks;

pub mod flags;
pub mod macros;

pub mod prelude {
    use super::spine;

    pub use spine::Color;
    pub use spine::File;
    pub use spine::Movelist;
    pub use spine::Rank;
    pub use spine::Square;
    pub use spine::{Bitboard, ShiftDir};
    pub use spine::{Board, CastleRights, State};
    pub use spine::{Move, MoveFlag};
    pub use spine::{Piece, PieceType};
}

pub use spine::bitboard::initialize_bitboards;

pub fn print_comp_flags() {
    print!("Flags: ");
    #[cfg(feature = "pext")]
    print!("PEXT ");

    print!("\n");
    println!("Version {}", VERSION);
}
