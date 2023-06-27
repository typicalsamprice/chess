use super::Bitboard;
use super::Square;
use super::{File, Rank};
use super::{Color, PieceType};
use super::get_sliding_attack;

use super::bitboard::PSEUDO_ATTACKS;
use super::bitboard::PAWN_ATTACKS;

pub fn pawn_attacks_by_board(pawns: Bitboard, color: Color) -> Bitboard {
    match color {
        Color::White => ((pawns << 7) &! File::H.to_bitboard()) | ((pawns << 9) & !File::A.to_bitboard()),
        Color::Black => ((pawns >> 7) &! File::A.to_bitboard()) | ((pawns >> 9) & !File::H.to_bitboard())
    }
}

pub(crate) fn king_attacks(square: Square) -> Bitboard {
    let mut rv = Bitboard::new(0);
    for shift in [1, 7, 8, 9, -1, -7, -8, -9] {
        if let Some(off) = square.offset(shift) {
            if square.distance(off) <= 2 {
                rv |= off.into();
            }
        }
    }

    rv
}

pub fn knight_attacks(square: Square) -> Bitboard {
    unsafe { PSEUDO_ATTACKS[0][square.as_u8() as usize] }
}

pub fn knight_attacks_by_board(knights: Bitboard) -> Bitboard {
    let mut rv = Bitboard::new(0);

    rv |= ((knights << 15) | (knights >> 17)) & !File::A.to_bitboard();
    rv |= ((knights >> 15) | (knights << 17)) & !File::H.to_bitboard();
    rv |= ((knights << 10) | (knights >> 6)) &! (File::A.to_bitboard() | File::B.to_bitboard());
    rv |= ((knights >> 10) | (knights >> 6)) &! (File::G.to_bitboard() | File::H.to_bitboard());

    rv
}

pub fn bishop_attacks(square: Square, occupancy: Bitboard) -> Bitboard {
    get_sliding_attack(false, square, occupancy)
}
pub fn rook_attacks(square: Square, occupancy: Bitboard) -> Bitboard {
    get_sliding_attack(true, square, occupancy)
}
pub fn queen_attacks(square: Square, occupancy: Bitboard) -> Bitboard {
    bishop_attacks(square, occupancy) | rook_attacks(square, occupancy)
}

pub fn piece_attack(square: Square, occupancy: Bitboard, piece_type: PieceType, color: Color) -> Bitboard {
    match piece_type {
        PieceType::Pawn => unsafe { PAWN_ATTACKS[color.as_usize()][square.as_u8() as usize] }
        PieceType::Knight => unsafe { PSEUDO_ATTACKS[0][square.as_u8() as usize] },
        PieceType::Bishop => bishop_attacks(square, occupancy),
        PieceType::Rook => rook_attacks(square, occupancy),
        PieceType::Queen => queen_attacks(square, occupancy),
        PieceType::King => unsafe { PSEUDO_ATTACKS[1][square.as_u8() as usize] }
    }
}
