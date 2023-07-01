use crate::spine::Color;

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PieceType {
    Pawn, Knight, Bishop,
    Rook, Queen, King
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Piece {
    kind: PieceType,
    color: Color
}

impl PieceType {
    pub const COUNT: usize = 6;

    #[inline(always)]
    pub const fn as_usize(self) -> usize {
        self as usize
    }
}

impl Piece {
    #[inline(always)]
    pub const fn new(kind: PieceType, color: Color) -> Self {
        Self { kind, color }
    }

    #[inline(always)]
    pub const fn as_usize(&self) -> usize {
        self.color.as_usize() * 8 + self.kind.as_usize()
    } 

    #[inline(always)]
    pub const fn kind(&self) -> PieceType { self.kind }
    #[inline(always)]
    pub const fn color(&self) -> Color { self.color }

    pub fn is_slider(&self) -> bool {
        self.kind <= PieceType::Queen && self.kind >= PieceType::Bishop
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let pcs = b"pnbrqk";
        let ch = pcs[self.kind().as_usize()] as char;
        let c = if self.color() == Color::White { ch.to_ascii_uppercase() } else { ch };
        write!(f, "{c}")
    }
}
