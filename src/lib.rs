#[warn(missing_debug_implementations)]
#[warn(unreachable_pub)]
#[warn(trivial_casts, trivial_numeric_casts)]
#[warn(unused_results)]
#[warn(missing_docs)]
mod spine;

pub use spine::bitboard;
#[doc(inline)]
pub use spine::movegen;
pub use spine::perft;
#[doc(inline)]
pub use spine::piece_attacks;

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
