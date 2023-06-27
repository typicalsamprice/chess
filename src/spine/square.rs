use super::{File, Rank};
use super::Bitboard;

use std::fmt;
use std::process::{abort, exit};

use super::bitboard::SQUARE_DIST;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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

    #[inline]
    pub const fn offset(self, os: i32) -> Option<Self> {
        let res = self.as_u8() as i32 + os;
        if res < 0 || res >= Self::COUNT as i32 {
            None
        } else {
            Some(Self(res as u8))
        }
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
        unsafe { SQUARE_DIST[self.as_u8() as usize][other.as_u8() as usize] }
    }
}

impl Into<Bitboard> for Square {
    fn into(self) -> Bitboard {
        Bitboard::new(1u64 << self.as_u8()) 
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let fc = ((self.0 & 7) + b'A') as char;
        let rc = ((self.0 >> 3) + b'1') as char;

        write!(f, "{fc}{rc}")
    }
}
