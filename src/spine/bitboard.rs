use super::initialize_magics;
use super::{Color, Square};
use super::piece_attacks::*;
use super::PieceType;

use std::ops;
use std::fmt;
use std::mem::transmute;

pub(crate) static mut SQUARE_DIST: [[i32; 64]; 64] = [[0; 64]; 64];
pub(crate) static mut PAWN_ATTACKS: [[Bitboard; 64]; 2] = [[Bitboard::ZERO; 64]; 2];
pub(crate) static mut PSEUDO_ATTACKS: [[Bitboard; 64]; 2] = [[Bitboard::ZERO; 64]; 2];
static mut LINE_BB: [[Bitboard; 64]; 64] = [[Bitboard::ZERO; 64]; 64];
static mut BETWEEN_BB: [[Bitboard; 64]; 64] = [[Bitboard::ZERO; 64]; 64];

pub fn between<const KEEP_END: bool>(a: Square, b: Square) -> Bitboard {
    debug_assert!(a.is_ok() && b.is_ok());
    let k = unsafe { BETWEEN_BB[a.as_usize()][b.as_usize()] };
    if !KEEP_END { // Remove the end bit
        k ^ Bitboard::from([b])
    } else {
        k
    }
}
pub fn line(a: Square, b: Square) -> Bitboard {
    debug_assert!(a.is_ok() && b.is_ok());
    unsafe { LINE_BB[a.as_usize()][b.as_usize()] }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Bitboard(u64);

pub(crate) fn initialize_bitboards() {
    for i in 0..64 {
        for j in 0..64 {
            let s = Square::new(i);
            let sj = Square::new(j);

            let r1 = s.rank().as_usize();
            let r2 = sj.rank().as_usize();
            let f1 = s.file().as_usize();
            let f2 = sj.file().as_usize();

            let rd = r1.abs_diff(r2);
            let fd = f1.abs_diff(f2);

            unsafe {
                SQUARE_DIST[i as usize][j as usize] = rd.max(fd) as i32;
            }
        }
    }

    initialize_magics();

    for i in 0..64 {
        let s = Square::new(i);

        unsafe {
            PAWN_ATTACKS[Color::White.as_usize()][i as usize] = pawn_attacks_by_board(s.into(), Color::White);
            PAWN_ATTACKS[Color::Black.as_usize()][i as usize] = pawn_attacks_by_board(s.into(), Color::Black);
            PSEUDO_ATTACKS[0][i as usize] = knight_attacks_by_board(s.into());
            PSEUDO_ATTACKS[1][i as usize] = king_attacks_comp(s);
        }

        for pt in [PieceType::Bishop, PieceType::Rook] {
            for j in 0..64 {
                let sj = Square::new(j);
                if (piece_attack(s, Bitboard::ZERO, pt, Color::White) & sj.to_bitboard()).gtz() {
                    unsafe {
                        LINE_BB[i as usize][j as usize] =
                            piece_attack(s, Bitboard::ZERO, pt, Color::White) &
                            piece_attack(sj, Bitboard::ZERO, pt, Color::White)
                            | Bitboard::from([s, sj]);
                        BETWEEN_BB[i as usize][j as usize] = piece_attack(s, sj.into(), pt, Color::White)
                            & piece_attack(sj, s.into(), pt, Color::White);
                    }
                }
                unsafe { BETWEEN_BB[i as usize][j as usize] |= sj.to_bitboard(); }
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
    pub const ZERO: Self = Self(0);
    pub const MAX: Self = Self(u64::MAX);

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
        self.0.count_ones()
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
    pub const fn lsb(self) -> Square {
        debug_assert!(self.gtz());
        Square::new(self.0.trailing_zeros() as u8)
    }
    #[inline]
    pub fn pop_lsb(&mut self) -> Option<Square> {
        if self.gtz() {
            let s = self.lsb();
            self.0 &= self.0 - 1;
            return Some(s);
        } 
        return None;
    }

}

impl ops::Not for Bitboard {
    type Output = Self;
    fn not(self) -> Self {
        Self(!self.0)
    }
}

impl<T> ops::BitOr<T> for Bitboard
    where T: Into<Self>
{
    type Output = Self;
    fn bitor(self, rhs: T) -> Self {
        Self(self.0 | rhs.into().0)
    }
} 

impl<T> ops::BitOrAssign<T> for Bitboard
    where T: Into<Self>, Self: ops::BitOr<T>
{
    fn bitor_assign(&mut self, rhs: T) {
        self.0 |= rhs.into().0;
    }
}

impl<T> ops::BitAnd<T> for Bitboard
    where T: Into<Self>
{
    type Output = Self;
    fn bitand(self, rhs: T) -> Self {
        Self(self.0 & rhs.into().0)
    }
} 

impl<T> ops::BitAndAssign<T> for Bitboard
    where T: Into<Self>, Self: ops::BitAnd<T>
{
    fn bitand_assign(&mut self, rhs: T) {
        self.0 &= rhs.into().0;
    }
}

impl<T> ops::BitXor<T> for Bitboard
    where T: Into<Self>
{
    type Output = Self;
    fn bitxor(self, rhs: T) -> Self {
        Self(self.0 ^ rhs.into().0)
    }
}

impl<T> ops::BitXorAssign<T> for Bitboard
    where T: Into<Self>, Self: ops::BitXor<T>
{
    fn bitxor_assign(&mut self, rhs: T) {
        self.0 ^= rhs.into().0;
    }
}

impl<T> ops::Shl<T> for Bitboard
    where u64: ops::Shl<T, Output = u64>
{
    type Output = Self;
    fn shl(self, shift: T) -> Self {
        Self(self.0.shl(shift))
    }
}

impl<T> ops::Shr<T> for Bitboard
    where u64: ops::Shr<T, Output = u64>
{
    type Output = Self;
    fn shr(self, shift: T) -> Self {
        Self(self.0.shr(shift))
    }
}

impl ops::Sub for Bitboard {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self(self.0.wrapping_sub(rhs.0))
    }
}
impl ops::Mul for Bitboard {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Self(self.0.wrapping_mul(rhs.0))
    }
}

impl<const N: usize> From<[Square; N]> for Bitboard {
    fn from(sqs: [Square; N]) -> Self {
        debug_assert!(N > 0);
        let mut s = 0;
        for i in 0..N {
            s |= 1 << sqs[i].as_u8();
        }
        Self(s)
    }
}
impl From<u64> for Bitboard {
    fn from(v: u64) -> Self { Self(v) }
}

impl fmt::Display for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = String::with_capacity(72);
        for i in 0..8 {
            for j in 0..8 {
                let r = 7 - i;
                let f = j;
                let b: Bitboard = unsafe { Square::build(transmute(f as u8), transmute(r as u8)).into() };
                if (*self & b).gtz() {
                    s.push('1');
                } else {
                    s.push('0');
                }
            }
            s.push('\n');
        }
        write!(f, "{s}")
    }
}
