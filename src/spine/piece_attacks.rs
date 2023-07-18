use crate::prelude::*;
use crate::spine::magic::magic_lookup;

use crate::spine::bitboard::PAWN_ATTACKS;
use crate::spine::bitboard::PSEUDO_ATTACKS;

pub fn pawn_attacks_by_board(pawns: Bitboard, color: Color) -> Bitboard {
    match color {
        Color::White => {
            ((pawns << 7) & !Bitboard::from(File::H)) | ((pawns << 9) & !Bitboard::from(File::A))
        }
        Color::Black => {
            ((pawns >> 7) & !Bitboard::from(File::A)) | ((pawns >> 9) & !Bitboard::from(File::H))
        }
    }
}
pub fn pawn_attacks(square: Square, color: Color) -> Bitboard {
    debug_assert!(square.is_ok());
    unsafe { PAWN_ATTACKS[color.as_usize()][square.as_usize()] }
}

pub fn king_attacks(square: Square) -> Bitboard {
    unsafe { PSEUDO_ATTACKS[1][square.as_usize()] }
}
pub(crate) fn king_attacks_comp(square: Square) -> Bitboard {
    let mut rv = Bitboard::ZERO;
    for shift in [1, 7, 8, 9, -1, -7, -8, -9] {
        if let Some(off) = square.offset(shift) {
            if square.distance(off) <= 2 {
                rv |= Into::<Bitboard>::into(off);
            }
        }
    }

    rv
}

pub fn knight_attacks(square: Square) -> Bitboard {
    unsafe { PSEUDO_ATTACKS[0][square.as_usize()] }
}

pub fn knight_attacks_by_board(knights: Bitboard) -> Bitboard {
    let mut rv = Bitboard::ZERO;

    rv |= ((knights >> 15) | (knights << 17)) & !Bitboard::from(File::A);
    rv |= ((knights << 15) | (knights >> 17)) & !Bitboard::from(File::H);
    rv |= ((knights << 10) | (knights >> 6)) & !(Bitboard::from(File::A) | Bitboard::from(File::B));
    rv |= ((knights >> 10) | (knights << 6)) & !(Bitboard::from(File::G) | Bitboard::from(File::H));

    rv
}

pub fn bishop_attacks(square: Square, occupancy: Bitboard) -> Bitboard {
    magic_lookup(false, square, occupancy)
}
pub fn rook_attacks(square: Square, occupancy: Bitboard) -> Bitboard {
    magic_lookup(true, square, occupancy)
}
pub fn queen_attacks(square: Square, occupancy: Bitboard) -> Bitboard {
    bishop_attacks(square, occupancy) | rook_attacks(square, occupancy)
}
