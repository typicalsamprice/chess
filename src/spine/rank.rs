use crate::prelude::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rank {
    One, Two, Three, Four,
    Five, Six, Seven, Eight
}

impl Rank {
    #[inline]
    pub const fn as_usize(self) -> usize {
        self as usize
    }

    #[inline]
    pub const fn relative_to(self, color: Color) -> Self {
        match color {
            Color::White => self,
            Color::Black => unsafe { std::mem::transmute(7 - self as u8) }
        }
    }
}
