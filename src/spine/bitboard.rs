use super::initialize_magics;
use super::{Color, Square};
use super::piece_attacks::*;
use super::PieceType;
use std::ops;

pub(crate) static mut SQUARE_DIST: [[i32; 64]; 64] = [[0; 64]; 64];
pub(crate) static mut PAWN_ATTACKS: [[Bitboard; 64]; 2] = [[Bitboard::new(0); 64]; 2];
pub(crate) static mut PSEUDO_ATTACKS: [[Bitboard; 64]; 2] = [[Bitboard::new(0); 64]; 2];
static mut LINE_BB: [[Bitboard; 64]; 64] = [[Bitboard::new(0); 64]; 64];
static mut BETWEEN_BB: [[Bitboard; 64]; 64] = [[Bitboard::new(0); 64]; 64];

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Bitboard(u64);

pub(crate) fn initialize_bitboards() {
    initialize_magics();

    for i in 0..64 {
        let s = Square::new(i);

        unsafe {
            PAWN_ATTACKS[Color::White.as_usize()][i as usize] = pawn_attacks_by_board(s.into(), Color::White);
            PAWN_ATTACKS[Color::Black.as_usize()][i as usize] = pawn_attacks_by_board(s.into(), Color::Black);
            PSEUDO_ATTACKS[0][i as usize] = knight_attacks_by_board(s.into());
            PSEUDO_ATTACKS[1][i as usize] = {
                let mut rv = Bitboard::new(0);
                for shift in [1, 7, 8, 9, -1, -7, -8, -9] {
                    if let Some(off) = s.offset(shift) {
                        if s.distance(off) <= 2 {
                            rv |= off.into();
                        }
                    }
                }
                rv
            };
        }

        for j in 0..64 {
            let sj = Square::new(j);

            unsafe { SQUARE_DIST[i as usize][j as usize] = (s.file().as_usize().abs_diff(sj.file().as_usize())).max(
                    s.rank().as_usize().abs_diff(sj.rank().as_usize())) as i32; }
        }

        for pt in [PieceType::Bishop, PieceType::Rook] {
            for j in 0..64 {
                let sj = Square::new(j);
                if (piece_attack(s, Bitboard::new(0), pt, Color::White) & sj.into()).gtz() {
                    unsafe {
                        LINE_BB[i as usize][j as usize] =
                            piece_attack(s, Bitboard::new(0), pt, Color::White) &
                            piece_attack(sj, Bitboard::new(0), pt, Color::White) | s.into() | sj.into();
                        BETWEEN_BB[i as usize][j as usize] = piece_attack(s, sj.into(), pt, Color::White)
                            & piece_attack(sj, s.into(), pt, Color::White);
                    }
                }
                unsafe { BETWEEN_BB[i as usize][j as usize] |= sj.into(); }
            }
        }
    }
}

impl Default for Bitboard {
    fn default() -> Self {
        Self(0)
    }
}

impl Bitboard {
    #[inline(always)]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    #[inline(always)]
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    #[inline(always)]
    pub const fn gtz(self) -> bool {
        self.0 > 0
    }

    #[inline(always)]
    pub const fn popcount(self) -> u32 {
        self.0.count_zeros()
    }

    #[inline(always)]
    pub const fn more_than_one(self) -> bool {
        self.0 & (self.0 - 1) > 0
    }

    #[inline(always)]
    pub const fn const_or(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub const fn shift(self, value: i32) -> Self {
        debug_assert!(value.abs() < 64);
        if value > 0 {
            Self(self.0 << value)
        } else {
            Self(self.0 >> -value)
        }
    }

    #[inline(always)]
    pub const fn ctz(self) -> u32 {
        debug_assert!(self.gtz());
        self.0.trailing_zeros()
    }

    pub fn carry_ripple(self, carrier: Self) -> Self {
        Self(self.0.wrapping_sub(carrier.0) & carrier.0)
    }
}

impl ops::Not for Bitboard {
    type Output = Self;
    fn not(self) -> Self {
        Self(!self.0)
    }
}

impl ops::BitOr for Bitboard {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
} 

impl ops::BitOrAssign for Bitboard {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl ops::BitAnd for Bitboard {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
} 

impl ops::BitAndAssign for Bitboard {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl ops::BitXor for Bitboard {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self {
        Self(self.0 ^ rhs.0)
    }
}

impl ops::BitXorAssign for Bitboard {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}

impl ops::Shl<i32> for Bitboard {
    type Output = Self;
    fn shl(self, shift: i32) -> Self {
        Self(self.0 << shift)
    }
}

impl ops::Shr<i32> for Bitboard {
    type Output = Self;
    fn shr(self, shift: i32) -> Self {
        Self(self.0 >> shift)
    }
}
