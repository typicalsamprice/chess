#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum File {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
}

impl File {
    #[inline(always)]
    pub const fn to_usize(self) -> usize {
        self as usize
    }

    #[inline(always)]
    pub const fn left(self) -> bool {
        self.to_usize() <= File::D.to_usize()
    }
    #[inline(always)]
    pub const fn middle(self) -> bool {
        let i = self.to_usize();
        i >= File::C.to_usize() && i <= File::F.to_usize()
    }
    #[inline(always)]
    pub const fn right(self) -> bool {
        self.to_usize() >= File::E.to_usize()
    }
}
