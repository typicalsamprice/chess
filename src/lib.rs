#[warn(missing_debug_implementations)]
#[warn(unreachable_pub)]
#[warn(trivial_casts, trivial_numeric_casts)]
#[warn(unused_results)]
#[deny(missing_docs)]

// FIXME: Make this sane. Also, does #[deny(missing_docs)] do anything?

mod spine;

pub use spine::bitboard;
pub use spine::movegen;
pub use spine::piece_attacks;

pub(crate) mod macros; 

pub mod prelude {
    use super::spine;

    pub use spine::bitboard::{Bitboard, ShiftDir};
    pub use spine::rank::Rank;
    pub use spine::file::File;
    pub use spine::square::Square;
    pub use spine::color::Color;
    pub use spine::piece::{PieceType, Piece};
    pub use spine::chess_move::{Move, MoveFlag};
    pub use spine::board::{Board, CastleRights, State};
}
