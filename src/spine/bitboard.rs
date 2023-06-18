use std::ops;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Bitboard(u64);

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
    pub const fn unwrap(self) -> u64 {
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
