#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Color { White, Black }

impl Color {
    pub const COUNT: usize = 2;

    #[inline(always)]
    pub const fn as_usize(self) -> usize {
        self as usize
    }
}

impl std::ops::Not for Color {
    type Output = Self; 
    fn not(self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White
        }
    }
}
