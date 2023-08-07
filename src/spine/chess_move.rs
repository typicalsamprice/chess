use crate::prelude::*;
use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq)]
/// A struct that holds the bit pattern for a chess move
pub struct Move(u32);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Flag that is used to signify a special property of a [`Move`]
pub enum MoveFlag {
    /// No special property
    Normal,
    /// En passant, the special pawn move described [here](https://en.wikipedia.org/wiki/En_passant)
    EnPassant,
    /// Castling the king
    Castle,
    /// Promotion of a pawn to a greater piece
    ///
    /// Note that if this flag is present in a [`Move`], there is the requirement
    /// for the "promotion type" bits to be a valid piece pattern
    Promotion,
}

impl Move {
    /// The bit pattern of sixteen zeros, an invalid move
    pub const NULL: Self = Self(0x0000);

    /// Create a new [`Move`] by passing in all the bits and pieces manually
    pub fn new(from: Square, to: Square, flag: MoveFlag, promotion_type: PieceType) -> Self {
        let frombits = from.to_u8() as u32;
        let tobits = (to.to_u8() as u32) << 6;
        let flagbits = flag.as_u32();
        let promo_bits = (promotion_type.to_usize() as u32) << 14;

        Self(frombits | tobits | flagbits | promo_bits)
    }

    /// Get the [`Square`] the piece is moving from
    pub const fn from_square(self) -> Square {
        Square::new((self.0 & 0x3f) as u8)
    }

    /// Get the [`Square`] the piece is moving to
    pub const fn to_square(self) -> Square {
        Square::new(((self.0 >> 6) & 0x3f) as u8)
    }

    /// The [`MoveFlag`] that signifies a special property of the [`Move`]
    pub const fn flag(self) -> MoveFlag {
        unsafe { std::mem::transmute(((self.0 >> 12) & 3) as u8) }
    }

    /// The [`PieceType`] that the pawn is promoting to.
    /// Note that if the flag of the [`Move`] is not [`MoveFlag::Promotion`],
    /// this **must** equal `PieceType::Pawn`, and can not equal the `Pawn` or `King`
    /// variants of `PieceType` when the flag *is* `MoveFlag::Promotion`
    pub const fn promotion_type(self) -> PieceType {
        unsafe { std::mem::transmute((self.0 >> 14) as u8) }
    }
}

impl MoveFlag {
    const fn as_u32(self) -> u32 {
        (self as u32) << 12
    }
}

impl fmt::Debug for Move {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Move {{ from: {}, to: {}, flag: {:?}, promotion_type: {:?} }}",
            self.from_square(),
            self.to_square(),
            self.flag(),
            self.promotion_type()
        )
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let map_promo = |t: PieceType| match t {
            PieceType::Pawn | PieceType::King => '\x00',
            PieceType::Knight => 'n',
            PieceType::Bishop => 'b',
            PieceType::Rook => 'r',
            PieceType::Queen => 'q',
        };

        write!(
            f,
            "{}{}{}",
            self.from_square(),
            self.to_square(),
            map_promo(self.promotion_type())
        )
    }
}
