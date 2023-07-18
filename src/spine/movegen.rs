use std::cmp::Ordering;
use crate::prelude::*;

#[derive(Debug)]
pub struct RankedMove {
    inner: Move,
    rank: f32
}

impl RankedMove {
    #[inline]
    pub const fn new(m: Move) -> Self {
        Self {
            inner: m,
            rank: 0f32
        }
    }

    #[inline]
    pub const fn unwrap(self) -> Move {
        self.inner
    }
}

// These only test for the `rank` field being equal/ordered
// so that they can be easily sorted (we can unwrap if we want the `inner` move)
impl PartialEq for RankedMove {
    fn eq(&self, other: &Self) -> bool {
        self.rank == other.rank
    }
}
impl PartialOrd for RankedMove {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.rank.partial_cmp(&other.rank)
    }
}

#[inline]
pub fn sort_ranked_moves(rankedmove_list: &mut [RankedMove]) {
    rankedmove_list.sort_by(|a, b| b.partial_cmp(a).unwrap())
}

pub fn movelist_to_rankedlist(movelist: Movelist) -> Vec<RankedMove> {
    movelist.as_slice().into_iter().map(|m| RankedMove::new(*m)).collect()
}
