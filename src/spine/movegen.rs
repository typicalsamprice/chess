use std::slice::Iter;

use crate::prelude::*;
use crate::bitboard;
use crate::piece_attacks;
use crate::spine::board::CastleRights;
use crate::macros::move_new;

use ShiftDir::*;

#[derive(Debug)]
/// A list that contains [`Move`]s generated within the `movegen` module.
///
/// The reason this is a fixed-size array rather than just a growable Vec<Move> is
/// that 1) the most moves in one position is roughly 220 and 2) this SHOULD be faster overall
/// than keeping the overhead of a Vec<T: Copy> when it's not needed
pub struct MoveList {
    moves: [Move; Self::MAX_MOVES],
    len: usize
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
/// The type of move to generate.
pub enum GenType {
    /// All moves
    All,
    /// Only captures
    Captures,
    /// Only moves that get a player out of check
    Evasions,
}

impl MoveList {
    /// The maximum moves to be generated at one time
    pub const MAX_MOVES: usize = 256;

    pub(crate) const fn new() -> Self {
        Self {
            moves: [Move::NULL; Self::MAX_MOVES],
            len: 0
        }
    }

    #[inline]
    /// Get the number of moves included.
    pub const fn len(&self) -> usize { self.len }
    #[inline]
    /// Get rid of all the moves (lazily)
    pub fn clear(&mut self) { self.len = 0; }

    #[inline]
    /// Get the move at `index`, returning `Some(Move)` if it exists,
    /// and `None` if there are not that many moves available.
    pub const fn get(&self, index: usize) -> Option<Move> {
        if index >= self.len {
            return None;
        }

        Some(self.moves[index])
    }

    /// Quickly removes a move by swapping in the last element to its place
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
    /// Adds a move to the list, returning `false` if it is full
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
    /// Returns an iterator over the moves
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

fn generate_promotions(us: Color, board: &Board, state: &State, from: Square, to: Square) -> MoveList {
    debug_assert_eq!(to.rank(), Rank::Eight.relative_to(us));

    let mut ml = MoveList::new();

    for pt in [PieceType::Knight, PieceType::Bishop, PieceType::Rook, PieceType::Queen] {
        let m = move_new!(from, to, MoveFlag::Promotion, pt);
        let b = ml.push(m);
        debug_assert!(b);
    }

    ml
}

fn generate_pawn_moves(board: &Board, state: &State, gen: GenType, targets: Bitboard) -> MoveList {
    let mut ml = MoveList::new();

    let us = board.to_move();
    let pawns = board.spec(us, PieceType::Pawn);
    let to_promote = pawns & Rank::Seven.relative_to(us);
    let rest = pawns ^ to_promote;

    let enemies = board.color(!us);
    let empty = !board.all();

    for p in to_promote {

    }

    ml
}
