use std::fmt;

use crate::bitboard::{self, SQUARE_DIST};
use crate::prelude::*;

use ShiftDir::*;

/// A square on the chessboard
#[allow(dead_code)]
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Square(u8);

impl Square {
    /// The number of squares on the board
    pub const COUNT: usize = 64;

    /// Create a new [`Square`] from a raw `u8`
    ///
    /// # Panics
    ///
    /// This method panics if `value` is not within [0, 64)
    #[inline]
    pub const fn new(value: u8) -> Self {
        debug_assert!(value <= 63);
        Self(value)
    }

    /// Create a new [`Square`] from its constituent [`File`] and [`Rank`]
    #[inline]
    pub const fn build(file: File, rank: Rank) -> Self {
        let i = file.as_usize() + (rank.as_usize() << 3);
        Self(i as u8)
    }

    /// Unwrap the [`Square`] to its inner `u8`
    #[inline]
    pub const fn as_u8(self) -> u8 {
        self.0
    }
    /// Unwrap the [`Square`] to its inner `u8` and convert to a `usize`
    #[inline]
    pub const fn as_usize(self) -> usize {
        self.0 as usize
    }

    /// Get the [`File`] a [`Square`] lands on
    ///
    /// # Panics
    ///
    /// This will panic if the square is invalid
    #[inline]
    pub const fn file(self) -> File {
        debug_assert!(self.is_ok());
        unsafe { std::mem::transmute(self.0 & 7) }
    }

    /// Get the [`Rank`] a [`Square`] lands on
    ///
    /// # Panics
    ///
    /// This will panic if the square is invalid
    #[inline]
    pub const fn rank(self) -> Rank {
        debug_assert!(self.is_ok());
        unsafe { std::mem::transmute(self.0 >> 3) }
    }

    /// Get the [`Square`] relative to the viewer. This means that
    /// if `color` is [`Color::Black`], it will "flip" the rank
    /// so that it is all from a standard reference point.
    #[inline]
    pub const fn relative_to(self, color: Color) -> Self {
        Self(self.0 ^ (color.as_usize() as u8 * 56))
    }

    /// Calculates the [`Square`] a certain offset away from `self`.
    /// This may go off the board, and so is an `Option<Square>`
    pub const fn offset(self, os: i32) -> Option<Self> {
        let res = self.as_u8() as i32 + os;
        if res < 0 || res >= Self::COUNT as i32 {
            None
        } else {
            Some(Self(res as u8))
        }
    }

    /// Checks whether a [`Square`] is valid.
    #[inline]
    pub const fn is_ok(self) -> bool {
        self.0 < Self::COUNT as u8
    }

    /// Checks whether two [`Square`]s are on a horizontal,
    /// vertical or diagonal line
    #[inline]
    pub fn in_line(self, other: Self) -> bool {
        if !(self.is_ok() && other.is_ok()) {
            return false;
        }
        self.distance(other) <= 1
            || (bitboard::line(self, other) ^ Bitboard::from([self, other])).gtz()
    }

    /// Checks whether three [`Square`]s are on the same line
    #[inline]
    pub fn in_line2(self, other: Self, other2: Self) -> bool {
        bitboard::line(self, other) & other2 == other2.into()
    }

    /// Fetches the (precomputed) distance between two [`Square`]s
    #[inline]
    pub fn distance(self, other: Self) -> i32 {
        unsafe { SQUARE_DIST[self.as_usize()][other.as_usize()] }
    }

    /// Converts a [`Square`] to a [`Bitboard`] with only the relevant bit set
    #[inline]
    pub const fn to_bitboard(self) -> Bitboard {
        Bitboard::new(1 << self.0)
    }
}

impl std::ops::Add<ShiftDir> for Square {
    type Output = Self;
    fn add(self, rhs: ShiftDir) -> Self::Output {
        let offset = match rhs {
            Forward(Color::White) | Backward(Color::Black) => 8,
            Backward(Color::White) | Forward(Color::Black) => -8,
        };
        let s = self.offset(offset);
        // Safety of unwrap(): this should only be used
        // when going *back* or *forward* to a known square.
        s.unwrap()
    }
}

/// These associated constants are just named squares, corresponding to their
/// standard names on the board.
#[allow(missing_docs)] // Dear god I don't want to document 64 constants...
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
        let fc = ((self.0 & 7) + b'a') as char;
        let rc = ((self.0 >> 3) + b'1') as char;

        write!(f, "{fc}{rc}")
    }
}
