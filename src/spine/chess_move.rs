use crate::spine::Square;
use crate::spine::PieceType;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Move(u16);
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MoveFlag {
    Normal,
    EnPassant,
    Castle,
    Promotion
}

impl Move {
    pub const NULL: Self = Self(0);
    
   pub const fn new(from: Square, to: Square,
                     flag: MoveFlag, promotion_type: PieceType) -> Self {
        let frombits = from.as_u8() as u16; 
        let tobits = (to.as_u8() as u16) << 6; 
        let flagbits = flag.as_u16();
        let promo_bits = (promotion_type.as_usize() as u16) << 14;

        Self(frombits | tobits | flagbits | promo_bits)
    }

    #[inline(always)]
    pub const fn from_square(self) -> Square {
        Square::new((self.0 & 0x3f) as u8)
    }
    #[inline(always)]
    pub const fn to_square(self) -> Square {
        Square::new(((self.0 >> 6) & 0x3f) as u8)
    }
    #[inline(always)]
    pub const fn flag(self) -> MoveFlag {
        unsafe { std::mem::transmute(((self.0 >> 12) & 3) as u8) }
    }
    #[inline(always)]
    pub const fn promotion_type(self) -> PieceType {
        unsafe { std::mem::transmute((self.0 >> 14) as u8) }
    }

    pub fn is_ok(self, fast: bool) -> bool { // FIXME: Maybe check this properly for fast = true?
        if fast {
            return self != Self::NULL;
        }

        let fr = self.from_square();
        let to = self.to_square();
        let fl = self.flag();
        let prom = self.promotion_type();

        if fr == to {
            return false;
        }

        if prom != PieceType::Pawn && fl != MoveFlag::Promotion {
            return false;
        }

        if fl == MoveFlag::Castle && !(fr.distance(to) == 2 && fr.rank() == to.rank()) {
            return false;
        }

        if prom == PieceType::Pawn || prom == PieceType::King {
            if fl != MoveFlag::Promotion {
                return false;
            }
        }

        true
    }
}

impl MoveFlag {
    #[inline(always)]
    pub const fn as_u16(self) -> u16 {
        (self as u16) << 12
    }
}
