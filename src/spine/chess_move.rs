use crate::prelude::{Square, PieceType};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// A struct that holds the bit pattern for a chess move
pub struct Move(u16);
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
    Promotion
}

impl Move {
    /// The bit pattern of sixteen zeros, an invalid move
    pub const NULL: Self = Self(0);
    
    /// Create a new [`Move`] by passing in all the bits and pieces manually
    pub const fn new(from: Square, to: Square,
                     flag: MoveFlag, promotion_type: PieceType) -> Self {
        let frombits = from.as_u8() as u16; 
        let tobits = (to.as_u8() as u16) << 6; 
        let flagbits = flag.as_u16();
        let promo_bits = (promotion_type.as_usize() as u16) << 14;

        Self(frombits | tobits | flagbits | promo_bits)
    }

    #[inline]
    /// Get the [`Square`] the piece is moving from
    pub const fn from_square(self) -> Square {
        Square::new((self.0 & 0x3f) as u8)
    }

    #[inline]
    /// Get the [`Square`] the piece is moving to
    pub const fn to_square(self) -> Square {
        Square::new(((self.0 >> 6) & 0x3f) as u8)
    }

    #[inline]
    /// The [`MoveFlag`] that signifies a special property of the [`Move`]
    pub const fn flag(self) -> MoveFlag {
        unsafe { std::mem::transmute(((self.0 >> 12) & 3) as u8) }
    }

    #[inline]
    /// The [`PieceType`] that the pawn is promoting to.
    /// Note that if the flag of the [`Move`] is not [`MoveFlag::Promotion`],
    /// this **must** equal `PieceType::Pawn`, and can not equal the `Pawn` or `King`
    /// variants of `PieceType` when the flag *is* `MoveFlag::Promotion`
    pub const fn promotion_type(self) -> PieceType {
        unsafe { std::mem::transmute((self.0 >> 14) as u8) }
    }

    #[inline]
    /// Check if the move is (sufficiently) valid. This does not check legality
    /// based on the type of the piece being moved (as this isn't tracked within a [`Move`]),
    /// but does do checks on the validity of the bit pattern contained. If the const generic
    /// `FAST == true`, then it only checks that `self` is not equal to [`Move::NULL`]
    pub fn is_ok<const FAST: bool>(self) -> bool {
        if FAST {  
            return self.0 > Self::NULL.0;
        }
        if self.to_square() == self.from_square() {
            return false;
        }

        if FAST {
            return true;
        }

        let pt = self.promotion_type();
        let fl = self.flag();

        if fl == MoveFlag::Castle || fl == MoveFlag::Promotion {
            if self.to_square().rank() != self.from_square().rank() {
                return false;
            }

            if fl == MoveFlag::Promotion {
                if pt == PieceType::Pawn || pt == PieceType::King {
                    return false;
                }
            } else if pt != PieceType::Pawn {
                return false;
            }
        }

        if fl == MoveFlag::EnPassant && self.to_square().distance(self.from_square()) != 1 {
            return false;
        }

        true
    }
}

impl MoveFlag {
    #[inline]
    const fn as_u16(self) -> u16 {
        (self as u16) << 12
    }
}
