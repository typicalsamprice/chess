use crate::bitboard;
use crate::macros::move_new;
use crate::piece_attacks::{
    self, bishop_attacks, knight_attacks, pawn_attacks_by_board, rook_attacks,
};
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

fn generate_pawn_promotions(list: &mut Movelist, from: Square, to: Square) {
    for pt in [Knight, Bishop, Rook, Queen] {
        list.push_back(Move::new(from, to, MoveFlag::Promotion, pt));
    }
}

fn generate_pawn_moves(
    list: &mut Movelist,
    board: &Board,
    state: &State,
    targets: Bitboard,
    _gen: GenType,
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
        list.push_back(move_new!(x + Backward(us), x));
    }
    for x in push_twice {
        list.push_back(move_new!(x + Backward(us) + Backward(us), x));
    }

    let t = targets & enemies;
    let rightup = (rest << Forward(us) << 1).and_not(File::A) & t;
    let leftup = (rest << Forward(us) >> 1).and_not(File::H) & t;
    for x in rightup {
        let orig = (x + Backward(us)).offset(-1).unwrap();
        list.push_back(move_new!(orig, x));
    }
    for x in leftup {
        let orig = (x + Backward(us)).offset(1).unwrap();
        list.push_back(move_new!(orig, x));
    }

    if let Some(ep) = state.en_passant() {
        let pawns = pawn_attacks(ep, them) & rest;
        for p in pawns {
            list.push_back(move_new!(p, ep, MoveFlag::EnPassant));
            dbg!(ep.to_string());
        }
    }
}

fn generate_king_moves(
    list: &mut Movelist,
    board: &Board,
    state: &State,
    _targets: Bitboard,
    gen: GenType,
) {
    let us = board.to_move();
    let king = board.king(us);

    let basic_moves = king_attacks(king) & !board.color(us);
    for x in basic_moves {
        list.push_back(move_new!(king, x));
    }

    if gen != GenType::All {
        return;
    }

    for right in state.castle_rights().rights_for(us) {
        if let Some(ct) = right {
            debug_assert_eq!(king, ct.king_from);
            if board.unblocked_castle(ct) {
                // Safety: We can unwrap() here because `unblocked_castle` would
                // return false if the variant was `None`.
                list.push_back(move_new!(ct.king_from, ct.king_to, MoveFlag::Castle));
            }
        }
    }
}

fn generate_piece_moves(
    list: &mut Movelist,
    board: &Board,
    _state: &State,
    targets: Bitboard,
    _gen: GenType,
) {
    let us = board.to_move();

    let knights = board.spec(us, Knight);
    for knight in knights {
        let atts = knight_attacks(knight) & targets;
        for a in atts {
            list.push_back(move_new!(knight, a));
        }
    }

    let b_queens = board.piece_type2(Bishop, Queen) & board.color(us);
    let r_queens = board.piece_type2(Rook, Queen) & board.color(us);

    for bq in b_queens {
        let attacks = bishop_attacks(bq, board.all()) & targets;
        for a in attacks {
            list.push_back(move_new!(bq, a));
        }
    }
    for rq in r_queens {
        let attacks = rook_attacks(rq, board.all()) & targets;
        for a in attacks {
            list.push_back(move_new!(rq, a));
        }
    }
}

/// Generates all legal moves in a position
pub fn generate_legal(board: &Board, state: &State) -> Movelist {
    let mut list = Movelist::new();

    let us = board.to_move();
    let king = board.king(us);
    let targets = match state.checkers().popcount() {
        1 => bitboard::between::<true>(king, state.checkers().lsb()),
        0 => !board.color(us),
        2 => Bitboard::ZERO, // We can only move the king!
        _ => unreachable!(),
    };

    let gt = if state.checkers().gtz() {
        GenType::Evasions
    } else {
        GenType::All
    };

    generate_pawn_moves(&mut list, board, state, targets, gt);

    generate_king_moves(&mut list, board, state, targets, gt);

    generate_piece_moves(&mut list, board, state, targets, gt);

    list.retain(|&m| {
        if m.from_square() == king
            || (state.blockers(us) & m.from_square()).gtz()
            || Some(m.to_square()) == state.en_passant()
        {
            board.is_legal(state, m)
        } else {
            true
        }
    });

    list
}
