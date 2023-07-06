use super::{Board, State};
use super::{Move, MoveFlag};

use super::{Forward, Backward};

use super::{Color, PieceType, Bitboard};
use super::Rank;

#[derive(Debug)]
pub struct MoveList {
    moves: [Move; Self::MAX_MOVES],
    len: usize
}

#[derive(Debug, PartialEq, Eq)]
pub enum GenType {
    All,
    Quiets,
    Checks,
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
}

impl Iterator for MoveList {
    type Item = Move;
    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }

        self.len -= 1;
        let m = self.moves[self.len];
        Some(m)
    }
}

fn generate_pawn_moves_for(board: &Board, state: &State, color: Color, caps: bool, gt: GenType) -> MoveList {
    let mut ml = MoveList::new();

    let lrank: Bitboard = Rank::Seven.relative_to(color).into();

    let a = board.all();
    let enemy = board.color(!color);

    let pawns = board.spec(color, PieceType::Pawn);
    let to_promote = pawns & lrank;
    let other_pawns = pawns &! lrank;

    let fw = Forward(color);
    let bw = Backward(color);

    let oner = (other_pawns << fw) &! a;
    let twor = (oner << fw) &! a;

    for x in oner.into_iter() {
        let m = Move::new(x + bw, x, MoveFlag::Normal, PieceType::Pawn);
        debug_assert!(m.from_square().is_ok());
        ml.push(m);
    }

    ml
}

pub fn generate_type(board: &Board, state: &State, color: Color, gt: GenType) -> MoveList {
    let mut ml = MoveList::new(); 

    ml
}

pub fn generate_legal(board: &Board, state: &State) -> MoveList {
    let us = board.to_move();
    let t = if state.checkers().gtz() { GenType::Evasions } else {
        GenType::All
    };

    generate_type(board, state, us, t)
}
