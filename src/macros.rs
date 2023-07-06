#[cfg(feature = "pext")]
use bitintr;

macro_rules! pext_u64 {
    ($a:expr, $b:expr) => {
        if cfg!(feature = "pext") {
            bitintr::Pext($a, $b)
        } else {
            0
        }
    }
}

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


pub(crate) use pext_u64;
pub(crate) use move_new;
