#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Color {
    White,
    Black,
}

impl Color {
    #[doc(hidden)]
    pub const COUNT: usize = 2;

    #[inline]
    pub const fn to_usize(&self) -> usize {
        *self as usize
    }

    #[inline]
    pub const fn multiplier(self) -> i32 {
        1 - (2 * self as i32)
    }
}

impl std::ops::Not for Color {
    type Output = Self;
    fn not(self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}
