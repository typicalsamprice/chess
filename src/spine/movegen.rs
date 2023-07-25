use crate::bitboard;
use crate::macros::move_new;
use crate::piece_attacks;
use crate::prelude::*;

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum GenType {
    /// Generate all legal moves in a position
    Legal,
    /// Generate all (pseudo-legal) moves in a position
    #[default]
    All,
    /// Generate only captures
    Captures,
    /// Generate only moves that get the king out of check
    Evasions,
}

use GenType::*;
use PieceType::*;

fn generate_pawn_promotions(list: &mut Movelist, from: Square, to: Square) {
    for pt in [Bishop, Rook, Knight, Queen] {
        list.push_back(Move::new(from, to, MoveFlag::Promotion, pt));
    }
}

fn generate_pawn_moves(
    list: &mut Movelist,
    board: &Board,
    state: &State,
    gen: GenType,
    targets: Bitboard,
) {
    let us = board.to_move();
    let pawns = board.spec(us, Pawn);

    let enemies = board.color(!us);
    let empty = !board.all();

    let up = ShiftDir::Forward(us);
    let down = ShiftDir::Backward(us);

    let seventh_rank_pawns = pawns & Rank::Seven.relative_to(us);
    let non_seventh_rank_pawns = pawns ^ seventh_rank_pawns;

    for pawn in seventh_rank_pawns {
        let promo_forward = (Bitboard::from(pawn) << up) & targets;
        let attacks = piece_attacks::pawn_attacks(pawn, us) & targets;

        if gen != Captures {
            if (targets & promo_forward).gtz() {
                generate_pawn_promotions(list, pawn, promo_forward.lsb());
            }
        }

        for attack in attacks {
            generate_pawn_promotions(list, pawn, attack);
        }
    }

    for pawn in non_seventh_rank_pawns {
        let atts = piece_attacks::pawn_attacks(pawn, us) & targets & (enemies | state.en_passant());
        for att in atts {
            let mf = if Some(att) == state.en_passant() {
                MoveFlag::EnPassant
            } else {
                MoveFlag::Normal
            };
            list.push_back(move_new!(pawn, att, mf));
        }
    }

    if gen == Captures {
        return;
    }

    let pawns_up_one = non_seventh_rank_pawns << up & empty;
    let pawns_up_two = (pawns_up_one & Rank::Three.relative_to(us)) << up & targets & empty;
    // We shadow this one so that we don't wipe any potential two-rank moves
    // that block a check.
    //
    // See: 1. c2c3 d7d6 2. d1a4 (b7b5 missed as response)
    let pawns_up_one = pawns_up_one & targets;

    for pawn in pawns_up_one {
        list.push_back(move_new!(pawn + down, pawn));
    }
    for pawn in pawns_up_two {
        list.push_back(move_new!(pawn + down + down, pawn));
    }
}

fn generate_piece_moves(
    list: &mut Movelist,
    board: &Board,
    _state: &State,
    gen: GenType,
    targets: Bitboard,
) {
    let us = board.to_move();
    let knights = board.spec(us, Knight);
    let bishops_queens = board.piece_type2(Bishop, Queen) & board.color(us);
    let rooks_queens = board.piece_type2(Rook, Queen) & board.color(us);
    let king = board.king(us);

    let pcs = board.all();

    for knight in knights {
        let atts = piece_attacks::knight_attacks(knight) & targets;
        for att in atts {
            list.push_back(move_new!(knight, att));
        }
    }

    for bq in bishops_queens {
        let atts = piece_attacks::bishop_attacks(bq, pcs) & targets;
        for att in atts {
            list.push_back(move_new!(bq, att));
        }
    }

    for rq in rooks_queens {
        let atts = piece_attacks::rook_attacks(rq, pcs) & targets;
        for att in atts {
            list.push_back(move_new!(rq, att));
        }
    }

    // We only care about using a "real" targets mask for king moves
    // when the goal is to generate captures
    let targets = if gen == Evasions { Bitboard::MAX } else { targets };
    let katts = piece_attacks::king_attacks(king) & targets;
    for att in katts {
        list.push_back(move_new!(king, att));
    }

    if gen == Captures { return; }

    let rights = CastleRights::rights_for(us);
    for right in rights {
        if let Some((from, to)) = _state.castle_rights().get(right) {
            if !board.pseudo_legal_castle(from, to) {
                continue;
            }
            debug_assert!(from == king);
            list.push_back(move_new!(from, to, MoveFlag::Castle));
        }
    }
}

pub fn generate_legal(board: &Board, state: &State) -> Movelist {
    let us = board.to_move();
    let gen = if state.checkers().gtz() {
        Evasions
    } else {
        All
    };
    let targets = match gen {
        Evasions => bitboard::between::<true>(board.king(us), state.checkers().lsb()),
        All => !board.color(us),
        _ => unreachable!(),
    };
    let mut ml = Movelist::new();

    generate_pawn_moves(&mut ml, board, state, gen, targets);
    generate_piece_moves(&mut ml, board, state, gen, targets);

    ml.retain(|&m| {
        let f = m.from_square();
        let us = board.to_move();
        let pinned = state.blockers(us);
        let k = board.king(us);

        if !((pinned & f).gtz() || f == k || m.flag() == MoveFlag::EnPassant) {
            return true;
        }

        board.is_legal(state, m)
    });

    ml
}
