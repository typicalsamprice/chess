#[warn(missing_debug_implementations)]
#[warn(unreachable_pub)]
#[warn(trivial_casts, trivial_numeric_casts)]
#[warn(unused_results)]
#[warn(missing_docs)]
// FIXME: Make this sane. Also, does #[deny(missing_docs)] do anything?
mod spine;

pub use spine::*;

pub(crate) mod macros;

pub mod prelude {
    use super::spine;

    pub use spine::bitboard::{Bitboard, ShiftDir};
    pub use spine::movelist::Movelist;
    pub use spine::board::{Board, CastleRights, State};
    pub use spine::chess_move::{Move, MoveFlag};
    pub use spine::color::Color;
    pub use spine::file::File;
    pub use spine::piece::{Piece, PieceType};
    pub use spine::rank::Rank;
    pub use spine::square::Square;
}
