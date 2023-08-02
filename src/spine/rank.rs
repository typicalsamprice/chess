use crate::prelude::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rank {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
}

impl Rank {
    pub const fn to_usize(self) -> usize {
        self as usize
    }

    pub const fn relative_to(self, color: Color) -> Self {
        match color {
            Color::White => self,
            Color::Black => unsafe { std::mem::transmute(7 - self as u8) },
        }
    }
}
