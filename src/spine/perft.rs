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

macro_rules! setup_perft {
    ($expected:literal, $depth:literal) => {
        setup_perft!($crate::prelude::Board::STARTPOS, $expected, $depth);
    };
    ($fen:expr, $expected:literal, $depth:literal) => {
        let mut s = $crate::prelude::State::new();
        let mut b = $crate::prelude::Board::new($fen, &mut s).unwrap();
        assert_eq!($crate::perft::perft_on(&mut b, &mut s, $depth), $expected);
    };
}

#[cfg(test)]
mod starting_position {
    use crate::bitboard::initialize_bitboards as bb_init;
    use std::sync::Once;

    static INIT: Once = Once::new();

    pub(super) fn init() {
        // Just make sure this happens ONE TIME. AUGH.
        INIT.call_once(|| bb_init())
    }

    mod shallow {
        use super::init;

        #[test]
        fn depth_one() {
            init();
            setup_perft!(20, 1);
        }

        #[test]
        fn depth_two() {
            init();
            setup_perft!(400, 2);
        }

        #[test]
        fn depth_three() {
            init();
            setup_perft!(8902, 3);
        }

        #[test]
        fn depth_four() {
            init();
            setup_perft!(197_281, 4);
        }
    }

    mod deepish {
        use super::init;

        #[test]
        fn depth_five() {
            init();
            setup_perft!(4_865_609, 5);
        }

        #[test]
        fn depth_six() {
            init();
            setup_perft!(119_060_324, 6);
        }

        #[test]
        #[ignore]
        fn depth_seven() {
            init();
            setup_perft!(3_195_901_860, 7);
        }
    }
}

#[cfg(test)]
mod kiwipete {
    use super::starting_position::init;

    mod shallow {
        use super::init;
        use crate::prelude::Board;

        #[test]
        fn depth_one() {
            init();
            setup_perft!(Board::KIWIPETE, 48, 1);
        }
        #[test]
        fn depth_two() {
            init();
            setup_perft!(Board::KIWIPETE, 2039, 2);
        }

        #[test]
        fn depth_three() {
            init();
            setup_perft!(Board::KIWIPETE, 97_862, 3);
        }

        #[test]
        fn depth_four() {
            init();
            setup_perft!(Board::KIWIPETE, 4_085_603, 4);
        }

        #[test]
        fn depth_five() {
            init();
            setup_perft!(Board::KIWIPETE, 193_690_690, 4);
        }
    }
}
