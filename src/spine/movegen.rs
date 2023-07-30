use crate::bitboard;
use crate::macros::move_new;
use crate::piece_attacks::{self, pawn_attacks_by_board};
use crate::piece_attacks::{king_attacks, pawn_attacks};
use crate::prelude::*;

use PieceType::*;
use ShiftDir::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenType {
    All,
    Legal,
    Captures,
    Evasions,
}

#[inline]
fn generate_pawn_promotions(list: &mut Movelist, from: Square, to: Square) {
    for pt in [Knight, Bishop, Rook, Queen] {
        list.push_back(move_new!(from, to, MoveFlag::Promotion, pt));
    }
}

fn generate_pawn_moves(
    list: &mut Movelist,
    board: &Board,
    state: &State,
    targets: Bitboard,
    gen: GenType,
) {
    let us = board.to_move();
    let them = !us;
    let pawns = board.spec(us, Pawn);
    let seventh_rank = pawns & Rank::Seven.relative_to(us);
    let rest = pawns ^ seventh_rank;

    let enemies = board.color(them);
    let empty = !board.all();

    for p in seventh_rank {
        let attacks = pawn_attacks(p, us) & enemies & targets;
        let push = Bitboard::from(p) << Forward(us) & empty & targets;

        for a in attacks {
            generate_pawn_promotions(list, p, a);
        }
        for x in push {
            generate_pawn_promotions(list, p, x);
        }
    }

    let push_once = rest << Forward(us) & empty;
    let push_twice = (push_once & Rank::Three.relative_to(us)) << Forward(us) & empty & targets;
    // INFO: The reason this has to be shadowed is so that
    // the movegen doesn't stop double-pushes from being used
    // to block checks.
    let push_once = push_once & targets;

    for x in push_once {
        list.push_back(move_new!(x, x + Backward(us)));
    }
    for x in push_twice {
        list.push_back(move_new!(x, x + Backward(us) + Backward(us)));
    }

    let t = targets & (enemies | state.en_passant());
    let rightup = (rest << Forward(us) << 1).and_not(File::A) & t;
    let leftup = (rest << Forward(us) >> 1).and_not(File::H) & t;
    for x in rightup {
        let orig = (x + Backward(us)).offset(-1).unwrap();
        if Some(x) == state.en_passant() {
            list.push_back(move_new!(orig, x, MoveFlag::EnPassant));
        } else {
            list.push_back(move_new!(orig, x));
        }
    }
    for x in leftup {
        let orig = (x + Backward(us)).offset(1).unwrap();
        if Some(x) == state.en_passant() {
            list.push_back(move_new!(orig, x, MoveFlag::EnPassant));
        } else {
            list.push_back(move_new!(orig, x));
        }
    }
}

fn generate_king_moves(
    list: &mut Movelist,
    board: &Board,
    state: &State,
    targets: Bitboard,
    gen: GenType,
) {
    let us = board.to_move();
    let king = board.king(us);

    let basic_moves = king_attacks(king) & targets;
    for x in basic_moves {
        list.push_back(move_new!(king, x));
    }
    let castles = CastleRights::rights_for(us);
    for ct in castles {
        if state.castle_rights().has_right(ct) {
            if board.unblocked_castle(state, ct) {
                // Safety: We can unwrap() here because `unblocked_castle` would
                // return false if the variant was `None`.
                let (_, to) = state.castle_rights().get(ct).unwrap();
                list.push_back(move_new!(king, to, MoveFlag::Castle));
            }
        }
    }
}

fn generate_piece_moves(
    list: &mut Movelist,
    board: &Board,
    state: &State,
    targets: Bitboard,
    gen: GenType,
) {
}
