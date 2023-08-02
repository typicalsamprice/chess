use crate::prelude::*;

#[derive(Debug, Clone)]
/// A structure containg a list of [`Move`]s, which can be traversed/filtered
pub struct Movelist {
    moves: Vec<Move>,
}

impl Movelist {
    /// The maximum moves in any (theoretical) position is ~220
    const MAX_MOVES: usize = 256;

    /// Create an empty `Movelist`

    pub fn new() -> Self {
        Self {
            moves: Vec::with_capacity(Self::MAX_MOVES),
        }
    }

    pub fn len(&self) -> usize {
        self.moves.len()
    }

    pub fn get(&self, index: usize) -> Option<&Move> {
        self.moves.get(index)
    }

    pub fn last(&self) -> Option<&Move> {
        self.moves.last()
    }

    pub fn push_back(&mut self, mv: Move) {
        // This is just to make sure we aren't reallocating unnecessarily.
        debug_assert_ne!(self.len(), Self::MAX_MOVES);
        self.moves.push(mv);
    }

    /// Get the Movelist as a slice of [`Move`]s

    pub fn as_slice(&self) -> &[Move] {
        &self.moves
    }

    /// Quickly swap out an element for the last element, returning the replaced
    /// [`Move`] or `Move::NULL` if there was none.
    ///
    /// Note that this is an `O(1)` operation, but does not preserve ordering
    /// like `Movelist::remove`
    pub fn swap_remove(&mut self, index: usize) -> Move {
        self.moves.swap_remove(index)
    }

    /// Filter a `Movelist` with the given predicate `f`, where it takes both a [`Board`]
    /// and a [`State`] along with each move, and if `f(board, state, move)` returns false, the [`Move`] is then
    /// swapped out (with `swap_remove`)
    pub fn retain<F>(&mut self, f: F)
    where
        F: Fn(&Move) -> bool,
    {
        self.moves.retain(f);
    }

    pub fn extend(&mut self, other: Self) {
        self.moves.extend_from_slice(other.as_slice());
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Move> {
        self.moves.iter()
    }
}
