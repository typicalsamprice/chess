#![allow(unused)]
use super::{File, Rank, bitboard};
use super::Bitboard;
use super::Color;

use std::fmt;

use super::bitboard::SQUARE_DIST;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Square(u8);

impl Square {
    pub const COUNT: usize = 64;

    #[inline(always)]
    pub const fn new(value: u8) -> Self {
        debug_assert!(value < Self::COUNT as u8);
        Self(value)
    }

    #[inline(always)]
    pub const fn build(file: File, rank: Rank) -> Self {
        let i = file.as_usize() + (rank.as_usize() << 3);
        Self(i as u8)
    }

    #[inline(always)]
    pub const fn as_u8(self) -> u8 {
        self.0
    }
    #[inline(always)]
    pub const fn as_usize(self) -> usize {
        self.0 as usize
    }

    #[inline(always)]
    pub const fn file(self) -> File {
        debug_assert!(self.is_ok());
        unsafe { std::mem::transmute(self.0 & 7) }
    }

    #[inline(always)]
    pub const fn rank(self) -> Rank {
        debug_assert!(self.is_ok());
        unsafe { std::mem::transmute(self.0 >> 3) }
    }

    #[inline(always)]
    pub const fn relative_to(self, color: Color) -> Self {
        Self(self.0 ^ (color.as_usize() as u8 * 56))
    }

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

    pub fn in_line(self, other: Self) -> bool {
        if !(self.is_ok() && other.is_ok()) {
            return false;
        }
        bitboard::line(self, other).gtz()
    }

    #[inline(always)]
    pub fn in_line2(self, other: Self, other2: Self) -> bool {
        bitboard::line(self, other) & other2 == other2.into()
    }

    #[inline(always)]
    pub fn distance(self, other: Self) -> i32 {
        unsafe { SQUARE_DIST[self.as_usize()][other.as_usize()] }
    }

    #[inline(always)]
    pub const fn to_bitboard(self) -> Bitboard {
        Bitboard::new(1 << self.0)
    }
}

impl Square {
    pub const A1: Self = Self(0);
    pub const B1: Self = Self(1);
    pub const C1: Self = Self(2);
    pub const D1: Self = Self(3);
    pub const E1: Self = Self(4);
    pub const F1: Self = Self(5);
    pub const G1: Self = Self(6);
    pub const H1: Self = Self(7);
    pub const A2: Self = Self(8);
    pub const B2: Self = Self(9);
    pub const C2: Self = Self(10);
    pub const D2: Self = Self(11);
    pub const E2: Self = Self(12);
    pub const F2: Self = Self(13);
    pub const G2: Self = Self(14);
    pub const H2: Self = Self(15);
    pub const A3: Self = Self(16);
    pub const B3: Self = Self(17);
    pub const C3: Self = Self(18);
    pub const D3: Self = Self(19);
    pub const E3: Self = Self(20);
    pub const F3: Self = Self(21);
    pub const G3: Self = Self(22);
    pub const H3: Self = Self(23);
    pub const A4: Self = Self(24);
    pub const B4: Self = Self(25);
    pub const C4: Self = Self(26);
    pub const D4: Self = Self(27);
    pub const E4: Self = Self(28);
    pub const F4: Self = Self(29);
    pub const G4: Self = Self(30);
    pub const H4: Self = Self(31);

    pub const A5: Self = Self(32);
    pub const B5: Self = Self(33);
    pub const C5: Self = Self(34);
    pub const D5: Self = Self(35);
    pub const E5: Self = Self(36);
    pub const F5: Self = Self(37);
    pub const G5: Self = Self(38);
    pub const H5: Self = Self(39);
    pub const A6: Self = Self(40);
    pub const B6: Self = Self(41);
    pub const C6: Self = Self(42);
    pub const D6: Self = Self(43);
    pub const E6: Self = Self(44);
    pub const F6: Self = Self(45);
    pub const G6: Self = Self(46);
    pub const H6: Self = Self(47);
    pub const A7: Self = Self(48);
    pub const B7: Self = Self(49);
    pub const C7: Self = Self(50);
    pub const D7: Self = Self(51);
    pub const E7: Self = Self(52);
    pub const F7: Self = Self(53);
    pub const G7: Self = Self(54);
    pub const H7: Self = Self(55);
    pub const A8: Self = Self(56);
    pub const B8: Self = Self(57);
    pub const C8: Self = Self(58);
    pub const D8: Self = Self(59);
    pub const E8: Self = Self(60);
    pub const F8: Self = Self(61);
    pub const G8: Self = Self(62);
    pub const H8: Self = Self(63);
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let fc = ((self.0 & 7) + b'A') as char;
        let rc = ((self.0 >> 3) + b'1') as char;

        write!(f, "{fc}{rc}")
    }
}
