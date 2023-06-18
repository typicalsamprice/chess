use crate::spine::{File, Rank};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Square(u8);

impl Square {
    pub const COUNT: usize = 64;

    #[inline(always)]
    pub const fn new(value: u8) -> Self {
        debug_assert!(value < Self::COUNT as u8);
        Self(value)
    }

    #[inline]
    pub const fn build(file: File, rank: Rank) -> Self {
        let i = file.as_usize() + rank.as_usize() * 8;
        Self(i as u8)
    }

    #[inline(always)]
    pub const fn as_u8(self) -> u8 {
        self.0
    }

    #[inline]
    pub const fn file(self) -> File {
        debug_assert!(self.is_ok());
        unsafe { std::mem::transmute(self.0 & 7) }
    }

    #[inline]
    pub const fn rank(self) -> File {
        debug_assert!(self.is_ok());
        unsafe { std::mem::transmute(self.0 >> 3) }
    }

    #[inline(always)]
    pub const fn is_ok(self) -> bool {
        self.0 < Self::COUNT as u8
    }

    pub fn is_in_line(self, other: Self) -> bool {
        if self.rank() == other.rank() || self.file() == other.file() {
            return true;
        }

        let diff = self.as_u8().abs_diff(other.as_u8());
        diff % 7 == 0 || diff % 9 == 0
    }

    pub fn distance(self, other: Self) -> i32 {
        (self.file().as_usize().abs_diff(other.file().as_usize())).max(self.rank().as_usize().abs_diff(other.rank().as_usize())) as i32
    }
}  
