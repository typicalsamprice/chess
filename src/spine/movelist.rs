use crate::prelude::*;

#[derive(Debug, Clone)]
/// A structure containg a list of [`Move`]s, which can be traversed/filtered
pub struct Movelist {
    moves: [Move; Self::MAX_MOVES],
    count: usize,
}

impl Movelist {
    /// The maximum moves in any (theoretical) position is ~220
    pub const MAX_MOVES: usize = 256;

    /// Create an empty `Movelist`
    #[inline]
    pub const fn new() -> Self {
        Self {
            moves: [Move::NULL; Self::MAX_MOVES],
            count: 0,
        }
    }

    pub fn push_back(&mut self, mv: Move) {
        debug_assert_ne!(self.count, Self::MAX_MOVES);
        self.moves[self.count] = mv;
        self.count += 1;
    }

    #[inline]
    pub const fn as_slice(&self) -> &[Move] {
        &self.moves[0..self.count]
    }

    pub fn remove(&mut self, index: usize) -> Move {
        if index >= self.count {
            return Move::NULL;
        }

        let rem = self.moves[index];
        self.moves[index..self.count - 1].copy_from_slice(&self.moves[index + 1..self.count]);
        rem
    }

    /// Quickly swap out an element for the last element, returning the replaced
    /// [`Move`] or `Move::NULL` if there was none.
    ///
    /// Note that this is an `O(1)` operation, but does not preserve ordering
    /// like `Movelist::remove`
    pub fn swap_remove(&mut self, index: usize) -> Move {
        if index >= self.count {
            return Move::NULL;
        }

        let rem = self.moves[index];
        self.count -= 1;
        self.moves[index] = self.moves[self.count];
        rem
    }

    /// Filter a `Movelist` with the given predicate `f`, where it takes both a [`Board`]
    /// and a [`State`] along with each move, and if `f(board, state, move)` returns false, the [`Move`] is then
    /// swapped out (with `swap_remove`)
    pub fn filter<F>(&mut self, board: &Board, state: &State, f: F)
    where
        F: Fn(&Board, &State, Move) -> bool,
    {
        for i in 0..self.count {
            let m = self.moves[i];
            if !f(board, state, m) {
                self.swap_remove(i);
            }
        }
    }

    pub fn extend(&mut self, other: Self) -> Result<(), usize> {
        if self.count + other.count > Self::MAX_MOVES {
            return Err(self.count + other.count);
        }
        self.moves[self.count..self.count + other.count]
            .copy_from_slice(&other.moves[0..other.count]);
        Ok(())
    }

    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, Move> {
        self.as_slice().iter()
    }
}
