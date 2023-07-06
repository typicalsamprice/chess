use super::{Board, State};
use super::movegen;

pub fn perft(depth: usize) -> usize {
    if depth == 0 { return 0; }
    let mut state = State::new();
    let mut board = Board::new(Board::STARTPOS, &mut state).unwrap();

    perft__::<true>(&mut board, &mut state, depth)
}

fn perft__<const root: bool>(board: &mut Board, state: &mut State, depth: usize) -> usize {
    let mut nodes = 0;
    let mut cur: usize;
    let leaf = depth == 2;
    
    let moves = movegen::generate_legal(board, state);

    for m in moves {
        if root && depth == 1 {
            cur = 1;
            nodes += 1;
        } else {
            board.do_move(state, m);
            cur = if leaf {
                // FIXME: Don't realloc, just alter `moves` in-place?
                movegen::generate_legal(board, state).len()
            } else { perft__::<false>(board, state, depth - 1) };
            nodes += cur;
            board.undo_move(state, m);
        }

        if root {
            println!("{}{}: {cur}", m.from_square(), m.to_square());
        }
    }


    nodes
}

#[cfg(test)]
mod tests {
    use super::perft;

    #[test]
    fn low_depth() {
        assert_eq!(20, perft(1));
        assert_eq!(400, perft(2));
        assert_eq!(8902, perft(3));
        assert_eq!(197_281, perft(4));
        assert_eq!(4_865_609, perft(5));
    }
}
