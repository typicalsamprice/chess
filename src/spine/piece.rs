use crate::prelude::Color;

use std::fmt;

/// An enum detailing the valid types of pieces one may
/// have on a chessboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[allow(missing_docs)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

/// A structure representing a piece on the board,
/// containing both a type (queen, rook, etc.) and a color (black or white)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Piece {
    kind: PieceType,
    color: Color,
}

impl PieceType {
    pub const COUNT: usize = 6;

    pub const fn to_usize(self) -> usize {
        self as usize
    }
}

impl Piece {
    pub const fn new(kind: PieceType, color: Color) -> Self {
        Self { kind, color }
    }

    pub const fn to_usize(&self) -> usize {
        self.color.to_usize() * 8 + self.kind.to_usize()
    }

    pub const fn kind(&self) -> PieceType {
        self.kind
    }
    pub const fn color(&self) -> Color {
        self.color
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let pcs = b"pnbrqk";
        let ch = pcs[self.kind().to_usize()] as char;
        let c = if self.color() == Color::White {
            ch.to_ascii_uppercase()
        } else {
            ch
        };
        write!(f, "{c}")
    }
}

impl std::ops::Add<Color> for PieceType {
    type Output = Piece;
    fn add(self, rhs: Color) -> Self::Output {
        Piece::new(self, rhs)
    }
}
