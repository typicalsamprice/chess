#[cfg(feature = "pext")]
use bitintr::Pext;

#[macro_export]
macro_rules! move_new {
    ($from:expr, $to:expr) => {
        move_new!($from, $to, MoveFlag::Normal, PieceType::Pawn)
    };
    ($from:expr, $to:expr, $ty:expr) => {
        move_new!($from, $to, $ty, PieceType::Pawn)
    };
    ($from:expr, $to:expr, $ty:expr, $promt:expr) => {
        Move::new($from, $to, $ty, $promt)
    };
}

#[cfg(feature = "pext")]
pub(crate) fn pext_u64(a: u64, b: u64) -> u64 {
    a.pext(b)
}

#[cfg(not(feature = "pext"))]
pub(crate) const fn pext_u64(_: u64, _: u64) -> u64 {
    0
}

pub use move_new;
