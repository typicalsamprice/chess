use crate::piece_attacks::*;
use crate::prelude::*;
use crate::spine::magic::initialize_magics;

use std::fmt;
use std::mem::transmute;
use std::ops;

use bitintr::Andn;
use bitintr::Blsi;
use bitintr::{Blsmsk, Bzhi, Popcnt, Tzcnt};

pub(crate) static mut SQUARE_DIST: [[i32; 64]; 64] = [[0; 64]; 64];
pub(crate) static mut PAWN_ATTACKS: [[Bitboard; 64]; 2] = [[Bitboard::ZERO; 64]; 2];
pub(crate) static mut PSEUDO_ATTACKS: [[Bitboard; 64]; 2] = [[Bitboard::ZERO; 64]; 2];
static mut LINE_BB: [[Bitboard; 64]; 64] = [[Bitboard::ZERO; 64]; 64];
static mut BETWEEN_BB: [[Bitboard; 64]; 64] = [[Bitboard::ZERO; 64]; 64];

/// A wrapper around a `u64` that denotes a chessboard and the occupied squares
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Bitboard(u64);

#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShiftDir {
    Forward(Color),
    Backward(Color),
}

pub fn between<const KEEP_END: bool>(from: Square, to: Square) -> Bitboard {
    debug_assert!(from.is_ok() && to.is_ok());
    let k = unsafe { BETWEEN_BB[from.to_usize()][to.to_usize()] };
    if !KEEP_END {
        // Remove the end bit
        k ^ to
    } else {
        k
    }
}
pub fn line(a: Square, b: Square) -> Bitboard {
    debug_assert!(a.is_ok() && b.is_ok());
    unsafe { LINE_BB[a.to_usize()][b.to_usize()] }
}

pub fn initialize_bitboards() {
    for i in 0..64 {
        for j in 0..64 {
            let s = Square::new(i);
            let sj = Square::new(j);

            let r1 = s.rank().to_usize();
            let r2 = sj.rank().to_usize();
            let f1 = s.file().to_usize();
            let f2 = sj.file().to_usize();

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
            PAWN_ATTACKS[Color::White.to_usize()][i as usize] =
                pawn_attacks_by_board(s.into(), Color::White);
            PAWN_ATTACKS[Color::Black.to_usize()][i as usize] =
                pawn_attacks_by_board(s.into(), Color::Black);
            PSEUDO_ATTACKS[0][i as usize] = knight_attacks_by_board(s.into());
            PSEUDO_ATTACKS[1][i as usize] = king_attacks_comp(s);
        }

        for pt in [PieceType::Bishop, PieceType::Rook] {
            for j in 0..64 {
                let sj = Square::new(j);
                let att = match pt {
                    PieceType::Bishop => |a, o| bishop_attacks(a, o),
                    PieceType::Rook => |a, o| rook_attacks(a, o),
                    _ => unreachable!(),
                };
                if (att(s, Bitboard::ZERO) & sj.to_bitboard()).gtz() {
                    unsafe {
                        LINE_BB[i as usize][j as usize] = att(s, Bitboard::ZERO)
                            & att(sj, Bitboard::ZERO)
                            | Bitboard::from([s, sj]);
                        BETWEEN_BB[i as usize][j as usize] = att(s, sj.into()) & att(sj, s.into());
                    }
                }
                unsafe {
                    BETWEEN_BB[i as usize][j as usize] |= sj.to_bitboard();
                }
            }
        }
    }
}

impl Default for Bitboard {
    fn default() -> Self {
        Self::ZERO
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
    pub const fn inner(self) -> u64 {
        self.0
    }

    #[inline(always)]
    pub const fn gtz(self) -> bool {
        self.0 != 0
    }

    #[inline(always)]
    pub fn popcount(self) -> u32 {
        self.0.popcnt() as u32
    }

    #[inline(always)]
    pub const fn more_than_one(self) -> bool {
        self.0 & (self.0.wrapping_sub(1)) > 0
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
    pub fn lsb(self) -> Square {
        let x = self.0.tzcnt();
        Square::new(x as u8)
    }

    pub fn pop_lsb(&mut self) -> Option<Square> {
        if self.gtz() {
            let s = self.lsb();
            *self &= self.blsi();
            return Some(s);
        }
        return None;
    }

    pub fn and_not<T>(self, arg: T) -> Self
    where
        T: Into<Self>,
    {
        arg.into().andn(self)
    }
}

// The bitintr stuff for Bitboards
impl Bzhi for Bitboard {
    fn bzhi(self, bit_position: u32) -> Self {
        Self(self.0.bzhi(bit_position))
    }
}
impl Blsmsk for Bitboard {
    fn blsmsk(self) -> Self {
        Self(self.0.blsmsk())
    }
}
impl Blsi for Bitboard {
    fn blsi(self) -> Self {
        Self(self.0.blsi())
    }
}
impl Andn for Bitboard {
    fn andn(self, y: Self) -> Self {
        Self(self.0.andn(y.0))
    }
}

impl ops::Not for Bitboard {
    type Output = Self;
    fn not(self) -> Self {
        Self(!self.0)
    }
}
impl ops::Neg for Bitboard {
    type Output = Self;
    fn neg(self) -> Self {
        Self(self.0.wrapping_neg())
    }
}

impl<T> ops::BitOr<T> for Bitboard
where
    T: Into<Self>,
{
    type Output = Self;
    fn bitor(self, rhs: T) -> Self {
        Self(self.0 | rhs.into().0)
    }
}

impl<T> ops::BitOrAssign<T> for Bitboard
where
    T: Into<Self>,
    Self: ops::BitOr<T, Output = Self>,
{
    fn bitor_assign(&mut self, rhs: T) {
        *self = *self | rhs;
    }
}

impl<T> ops::BitAnd<T> for Bitboard
where
    T: Into<Self>,
{
    type Output = Self;
    fn bitand(self, rhs: T) -> Self {
        Self(self.0 & rhs.into().0)
    }
}

impl<T> ops::BitAndAssign<T> for Bitboard
where
    T: Into<Self>,
    Self: ops::BitAnd<T, Output = Self>,
{
    fn bitand_assign(&mut self, rhs: T) {
        *self = *self & rhs;
    }
}

impl<T> ops::BitXor<T> for Bitboard
where
    T: Into<Self>,
{
    type Output = Self;
    fn bitxor(self, rhs: T) -> Self {
        Self(self.0 ^ rhs.into().0)
    }
}

impl<T> ops::BitXorAssign<T> for Bitboard
where
    T: Into<Self>,
    Self: ops::BitXor<T, Output = Self>,
{
    fn bitxor_assign(&mut self, rhs: T) {
        *self = *self ^ rhs;
    }
}

impl<T> ops::Shl<T> for Bitboard
where
    u64: ops::Shl<T, Output = u64>,
{
    type Output = Self;
    fn shl(self, shift: T) -> Self {
        Self(self.0.shl(shift))
    }
}

impl<T> ops::Shr<T> for Bitboard
where
    u64: ops::Shr<T, Output = u64>,
{
    type Output = Self;
    fn shr(self, shift: T) -> Self {
        Self(self.0.shr(shift))
    }
}

impl ops::Shl<ShiftDir> for Bitboard {
    type Output = Self;
    fn shl(self, rhs: ShiftDir) -> Self {
        match rhs {
            ShiftDir::Forward(Color::White) => self << 8,
            ShiftDir::Forward(Color::Black) => self >> 8,
            ShiftDir::Backward(Color::White) => self >> 8,
            ShiftDir::Backward(Color::Black) => self << 8,
        }
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
        for sq in sqs.iter().take(N) {
            debug_assert!(sq.is_ok());
            s |= 1 << sq.to_usize();
        }
        Self(s)
    }
}
impl From<u64> for Bitboard {
    fn from(v: u64) -> Self {
        Self(v)
    }
}

impl From<Square> for Bitboard {
    fn from(s: Square) -> Self {
        s.to_bitboard()
    }
}
impl From<Option<Square>> for Bitboard {
    fn from(opts: Option<Square>) -> Self {
        match opts {
            Some(s) => Self::from(s),
            None => Self::ZERO,
        }
    }
}

impl From<File> for Bitboard {
    fn from(f: File) -> Self {
        Self(0x0101010101010101_u64 << f.to_usize())
    }
}
impl From<Rank> for Bitboard {
    fn from(r: Rank) -> Self {
        Self(0xff_u64 << (8 * r.to_usize()))
    }
}

impl ops::BitAnd<Bitboard> for (Bitboard, Bitboard) {
    type Output = Self;
    fn bitand(self, m: Bitboard) -> Self::Output {
        (self.0 & m, self.1 & m)
    }
}

impl Iterator for Bitboard {
    type Item = Square;
    fn next(&mut self) -> Option<Self::Item> {
        if self.0 == 0 {
            return None;
        }
        let l = self.lsb();
        *self ^= l;
        Some(l)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let l = self.popcount() as usize;
        (l, Some(l))
    }
}
impl ExactSizeIterator for Bitboard {}

impl fmt::Debug for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Bitboard({:#016x?})", self.0)
    }
}
impl fmt::Display for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = String::with_capacity(72);
        for i in 0..8 {
            for j in 0..8 {
                let r = 7 - i;
                let f = j;
                let b: Bitboard =
                    unsafe { Square::build(transmute(f as u8), transmute(r as u8)).into() };
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
