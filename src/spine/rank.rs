use crate::spine::Bitboard;
use crate::spine::Color;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rank {
    One, Two, Three, Four,
    Five, Six, Seven, Eight
}

impl Rank {
    #[inline(always)]
    pub const fn as_usize(self) -> usize {
        self as usize
    }

    #[inline(always)]
    pub const fn to_bitboard(self) -> Bitboard {
        Bitboard::new(0xFFu64 << (self.as_usize() * 8))
    }
    
    #[inline(always)]
    pub const fn relative_to(self, color: Color) -> Self {
        match color {
            Color::White => self,
            Color::Black => unsafe { std::mem::transmute(7 - self.as_usize() as u8) }
        }
    }
}
