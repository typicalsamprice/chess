use bitintr::Pext;

#[macro_export]
macro_rules! move_new {
    ("O-O"; $b:ident, $s:ident) => {{
        let state_: &State = &$s;
        let board_: &Board = &$b;
        let r = state_.castle_rights().rights_for(board_.to_move())[0].unwrap();
        move_new!(r.king_from, r.king_to, MoveFlag::Castle)
    }};

    ("O-O-O"; $b:ident, $s:ident) => {{
        let state_: &State = &$s;
        let board_: &Board = &$b;
        let r = state_.castle_rights().rights_for(board_.to_move())[1].unwrap();
        move_new!(r.king_from, r.king_to, MoveFlag::Castle)
    }};

    ($lit:literal) => {{
        let b = $lit.as_bytes();
        let f1 = b[0] - b'a';
        let r1 = b[1] - b'1';
        let f2 = b[2] - b'a';
        let r2 = b[3] - b'1';
        let p = match b.iter().nth(4).unwrap_or(&b'\x00') {
            b'n' => PieceType::Pawn,
            b'b' => PieceType::Bishop,
            b'r' => PieceType::Rook,
            b'q' => PieceType::Queen,
            _ => PieceType::Pawn,
        };
        let f = match p {
            PieceType::Pawn => MoveFlag::Normal,
            _ => MoveFlag::Promotion,
        };

        let s1 = f1 + 8 * r1;
        let s2 = f2 + 8 * r2;
        Move::new(Square::new(s1), Square::new(s2), f, p)
    }};

    ($from:expr, $to:expr) => {
        move_new!($from, $to, MoveFlag::Normal, PieceType::Pawn)
    };
    ($from:expr, $to:expr, $ty:expr) => {
        move_new!($from, $to, $ty, PieceType::Pawn)
    };
    ($from:expr, $to:expr, $ty:expr, $promt:expr) => {
        Move::new($from, $to, $ty, $promt)
    };
}

pub(crate) fn pext_u64(a: u64, b: u64) -> u64 {
    a.pext(b)
}

pub use move_new;
