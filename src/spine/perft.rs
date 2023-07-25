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
            board.do_move(state, m).unwrap();
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
mod perft_starting_position_tests {
    use std::sync::Once;

    use super::perft;
    use crate::bitboard::initialize_bitboards as bb_init;

    static INIT: Once = Once::new();

    fn init() {
        // Just make sure this happens ONE TIME. AUGH.
        INIT.call_once(|| bb_init())
    }

    mod shallow {
        use super::{init, perft};

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

    mod deepish {
        use super::{init, perft};

        #[test]
        fn depth_five() {
            init();
            assert_eq!(4_865_609, perft(5));
        }

        #[test]
        fn depth_six() {
            init();
            assert_eq!(119_060_324, perft(6));
        }
    }

    #[test]
    #[ignore = "Too long"]
    fn depth_seven() {
        init();
        assert_eq!(3_195_901_860, perft(7));
    }
}

#[cfg(test)]
mod perft_kiwipete {
    use std::sync::Once;

    use crate::bitboard::initialize_bitboards as bb_init;

    static INIT: Once = Once::new();

    static KIWI_FEN: &str = todo!();

    fn init() {
        // Just make sure this happens ONE TIME. AUGH.
        INIT.call_once(|| bb_init())
    }

    #[test]
    fn depth_one() {
        use super::perft_on;
        use crate::prelude::*;

        init();
        let mut s = State::new();
        let mut b = Board::new(KIWI_FEN, &mut s).unwrap();
        assert_eq!(perft_on(&mut b, &mut s, 1), 48);
    }
}
