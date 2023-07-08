use std::slice::Iter;

use crate::prelude::*;
use crate::bitboard;
use crate::piece_attacks;
use crate::spine::board::CastleRights;
use crate::macros::move_new;

use ShiftDir::*;

#[derive(Debug)]
pub struct MoveList {
    moves: [Move; Self::MAX_MOVES],
    len: usize
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum GenType {
    All,
    Captures,
    Evasions,
}

impl MoveList {
    pub const MAX_MOVES: usize = 256;

    pub const fn new() -> Self {
        Self {
            moves: [Move::NULL; Self::MAX_MOVES],
            len: 0
        }
    }

    #[inline]
    pub const fn len(&self) -> usize { self.len }
    #[inline]
    pub fn clear(&mut self) { self.len = 0; }

    #[inline]
    pub const fn get(&self, index: usize) -> Option<Move> {
        if index >= self.len {
            return None;
        }

        Some(self.moves[index])
    }

    pub fn swap_remove(&mut self, index: usize) -> Option<Move> {
        if index >= self.len {
            None
        } else {
            let m = self.moves[index];
            let l = self.moves[self.len - 1];
            self.moves[index] = l;
            self.moves[self.len - 1] = Move::NULL;
            self.len -= 1;
            Some(m)
        }
    }

    #[inline]
    pub fn push(&mut self, m: Move) -> bool {
        if self.len == Self::MAX_MOVES {
            return false;
        }

        self.moves[self.len] = m;
        self.len += 1;
        true
    }

    fn extend(&mut self, other: Self) -> Result<(), usize> {
        if self.len + other.len >= Self::MAX_MOVES {
            // How many moves greater than maximum are we?
            return Err(self.len + other.len - Self::MAX_MOVES + 1);
        }

        self.moves[self.len..(self.len + other.len)].copy_from_slice(&other.moves[..other.len]);
        self.len += other.len;

        Ok(())
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, Move> {
        self.moves[0..self.len].iter()
    }
}

impl<'a> IntoIterator for &'a MoveList {
    type IntoIter = Iter<'a, Move>;
    type Item = &'a Move;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}


fn make_promotions(ml: &mut MoveList, from: Square, to: Square) {
    for &pt in [PieceType::Knight, PieceType::Bishop,
                PieceType::Rook, PieceType::Queen].iter() {
        ml.push(Move::new(from, to, MoveFlag::Promotion, pt));
    }
}
fn generate_pawn_moves_for(board: &Board, state: &State, color: Color, caps: bool, gt: GenType) -> MoveList {
    let mut ml = MoveList::new();

    let lrank: Bitboard = Rank::Seven.relative_to(color).into();
    let trank: Bitboard = Rank::Three.relative_to(color).into();

    let a = board.all();
    let enemy = board.color(!color);

    let pawns = board.spec(color, PieceType::Pawn);
    let to_promote = pawns & lrank;
    let other_pawns = pawns &! lrank;

    let fw = Forward(color);
    let bw = Backward(color);

    let oner = (other_pawns << fw) &! a;
    let twor = ((oner & trank) << fw) &! a;

    for x in oner.into_iter() {
        let m = move_new!(x + bw, x);
        ml.push(m);
    }

    for x in twor.into_iter() {
        let m = move_new!(x + bw + bw, x);
        ml.push(m);
    }

    if caps {
        let leftup = (other_pawns << fw >> 1) &! Bitboard::from(File::H);
        let rightup = (other_pawns << fw << 1) &! Bitboard::from(File::A);
        let lru = leftup & enemy;
        let rru = rightup & enemy;

        let ep_bb = state.en_passant().map_or(Bitboard::ZERO, Bitboard::from);
        let epc = (leftup | rightup) & ep_bb;

        if epc.gtz() {
            let pawns_capping =
                piece_attacks::pawn_attacks_by_board(epc, !color) & other_pawns;
            for x in pawns_capping {
                let e = epc.lsb();
                let m = Move::new(x, e, MoveFlag::EnPassant, PieceType::Pawn);
                ml.push(m);
            }
        }

        for x in lru {
            let s = (x + bw).offset(1).unwrap();
            let m = Move::new(s, x, MoveFlag::Normal, PieceType::Pawn);
            ml.push(m);
        }

        for x in rru {
            let s = (x + bw).offset(-1).unwrap();
            let m = Move::new(s, x, MoveFlag::Normal, PieceType::Pawn);
            ml.push(m);
        }
    }

    for p in to_promote {
        let up = p + fw;
        let upl = (p + fw).offset(-1);
        let upr = (p + fw).offset(1);
        if !(board.all() & up).gtz() {
            make_promotions(&mut ml, p, up);
        }

        if !caps {
            continue;
        }

        if let Some(upl) = upl {
            if (enemy & upl).gtz() {
                make_promotions(&mut ml, p, upl);
            }
        }

        if let Some(upr) = upr {
            if (enemy & upr).gtz() {
                make_promotions(&mut ml, p, upr);
            }
        }
    }

    ml
}

fn generate_king_moves(board: &Board, state: &State, color: Color, caps: bool) -> MoveList {
    let mut ml = MoveList::new();

    let ks = board.king(color);
    let enemies = board.color(!color);
    let all_minus_ks = board.all() ^ ks;

    let mut evasions = piece_attacks::king_attacks(ks) &! board.color(color);
    if caps {
        evasions &= enemies;
    }
    for x in evasions {
        if (board.attacks_to_bits(x, all_minus_ks) & enemies).gtz() {
            continue;
        }

        ml.push(move_new!(ks, x));
    }

    if state.checkers().gtz() {
        return ml;
    }

    let (ksc, qsc) = if color == Color::White {
        (CastleRights::W_OO, CastleRights::W_OOO)
    } else {
        (CastleRights::B_OO, CastleRights::B_OOO)
    };

    let castle_tuples = [(ksc, Square::G1),
                         (qsc, Square::C1)];

    'outer: for &(cr, tosq) in castle_tuples.iter() {
        let betw = bitboard::between::<true>(ks, tosq);
        if !state.castle_rights().has_exact_rights(cr) { continue; }
        if (betw & board.all()).gtz() { continue; }
        for x in betw {
            if (board.attacks_to_bits(x, all_minus_ks) & enemies).gtz() { continue 'outer; }
        }

        ml.push(move_new!(ks, tosq, MoveFlag::Castle));
    }

    ml
}

fn generate_piece_moves_for(
        board: &Board, state: &State, color: Color,
        caps: bool, gt: GenType, pt: PieceType
    ) -> MoveList {
    let mut ml = MoveList::new();

    let pcs = board.spec(color, pt);
    let all = board.all();
    for p in pcs {
        let mut atts = match pt {
            PieceType::Knight => piece_attacks::knight_attacks(p),
            PieceType::Bishop => piece_attacks::bishop_attacks(p, all),
            PieceType::Rook => piece_attacks::rook_attacks(p, all),
            PieceType::Queen => piece_attacks::queen_attacks(p, all),
            _ => unreachable!(),
        } &! board.color(color);

        if gt == GenType::Captures {
            atts &= board.color(!color);
        } else if gt == GenType::Evasions {
            atts &= bitboard::between::<true>(board.king(color), state.checkers().lsb());
        }

        if !caps {
            atts &= !board.color(!color);
        }

        for t in atts {
            ml.push(move_new!(p, t));
        }
    }

    ml
}

fn generate_piece_moves(board: &Board, state: &State, color: Color, caps: bool, gt: GenType) -> MoveList {
    let mut ml = MoveList::new();

    let pts = [PieceType::Knight, PieceType::Bishop, PieceType::Rook, PieceType::Queen];

    let _ = ml.extend(generate_king_moves(board, state, color, caps));

    if state.checkers().more_than_one() {
        return ml;
    }

    for &pt in pts.iter() {
        let _ = ml.extend(generate_piece_moves_for(board, state, color, caps, gt, pt));
    }

    ml 
}

pub fn generate_legal(board: &Board, state: &State) -> MoveList {
    let color = board.to_move();
    let gt = if state.checkers().gtz() {
        GenType::Evasions
    } else {
        GenType::All
    };
    let mut ml = generate_pawn_moves_for(board, state, color, true, gt);
    let _ = ml.extend(generate_piece_moves(board, state, color, true, gt));

    ml
}
