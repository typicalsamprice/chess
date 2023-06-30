use super::Bitboard;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum File {
    A, B, C, D, E, F, G, H
}

impl File {
    #[inline(always)]
    pub const fn as_usize(self) -> usize {
        self as usize
    }

    #[inline(always)]
    pub const fn to_bitboard(self) -> Bitboard {
        Bitboard::new(0x0101010101010101u64 << self.as_usize())
    }
    
    #[inline(always)]
    pub const fn left(self) -> bool {
        self.as_usize() <= File::D.as_usize()
    }
    #[inline(always)]
    pub const fn middle(self) -> bool {
        let i = self.as_usize();
        i >= File::C.as_usize() && i <= File::F.as_usize()
    }
    #[inline(always)]
    pub const fn right(self) -> bool {
        self.as_usize() >= File::E.as_usize() 
    }
}

impl Into<Bitboard> for File {
    fn into(self) -> Bitboard {
        self.to_bitboard()
    }
}
