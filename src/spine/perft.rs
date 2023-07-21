use crate::movegen;
use crate::prelude::*;

pub fn perft(depth: usize) -> usize {
    if depth == 0 {
        return 0;
    }
    let mut state = State::new();
    let mut board = Board::new(Board::STARTPOS, &mut state).unwrap();

    perft__::<true>(&mut board, &mut state, depth)
}

pub fn perft_on(board: &mut Board, state: &mut State, depth: usize) -> usize {
    assert!(depth > 0);
    perft__::<true>(board, state, depth)
}

fn perft__<const ROOT: bool>(board: &mut Board, state: &mut State, depth: usize) -> usize {
    let mut nodes = 0;
    let mut cur: usize;
    let leaf = depth == 2;

    let moves: Movelist = movegen::generate_legal(board, state);

    for &m in moves.iter() {
        if ROOT && depth <= 1 {
            cur = 1;
            nodes += 1;
        } else {
            board.do_move(state, m);
            cur = if leaf {
                movegen::generate_legal(board, state).len()
            } else {
                perft__::<false>(board, state, depth - 1)
            };
            nodes += cur;
            board.undo_move(state, m);
        }

        if ROOT && !cfg!(test) {
            println!("{}{}: {cur}", m.from_square(), m.to_square());
        }
    }

    nodes
}

#[cfg(test)]
mod tests {
    use std::sync::Once;

    use super::perft;
    use crate::bitboard::initialize_bitboards as bb_init;

    static INIT: Once = Once::new();

    fn init() {
        // Just make sure this happens ONE TIME. AUGH.
        INIT.call_once(|| bb_init())
    }

    #[test]
    fn depth_one() {
        init();
        assert_eq!(20, perft(1));
    }

    #[test]
    fn depth_two() {
        init();
        assert_eq!(400, perft(2));
    }

    #[test]
    fn depth_three() {
        init();
        assert_eq!(8902, perft(3));
    }

    #[test]
    fn depth_four() {
        init();
        assert_eq!(197_281, perft(4));
    }
}
