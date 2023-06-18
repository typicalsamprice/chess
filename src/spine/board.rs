use crate::spine::{Bitboard, Color, File, Move, Piece, PieceType, Rank, Square};
use std::rc::Rc;

#[derive(Clone)]
pub struct Board {
    color_bb: [Bitboard; Color::COUNT],
    piece_bb: [Bitboard; PieceType::COUNT],
    pieces: [Option<Piece>; Square::COUNT],
    piece_count: [i8; PieceType::COUNT * Color::COUNT],
    to_move: Color,
    ply: usize,

    state: Rc<State>
}

pub struct State {
    castle_rights: CastleRights,
    en_passant: Option<Square>,
    half_moves: usize,
    plies_from_null: usize,

    checkers: Bitboard,
    check_squares: [Bitboard; PieceType::COUNT],
    blockers: [Bitboard; Color::COUNT],
    pinners: [Bitboard; Color::COUNT],
    captured_piece: Option<PieceType>
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct CastleRights(u8);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BoardCreationError {
    NoFenGiven,
    InvalidPiece,
    InvalidColor,
    InvalidCastleRights,
    InvalidEnPassant,
    InvalidNumber
}

impl Board {
    pub fn from_fen(fen: &str) -> Result<Self, BoardCreationError> {
        let mut b = Self {
            color_bb: [Bitboard::new(0); Color::COUNT],
            piece_bb: [Bitboard::new(0); PieceType::COUNT],
            pieces: [None; Square::COUNT],
            piece_count: [0; PieceType::COUNT * Color::COUNT],
            to_move: Color::White,
            ply: 0,
            state: Rc::new(State::new())
        };
        Err(BoardCreationError::NoFenGiven)
    }

    #[inline(always)] 
    pub const fn color(&self, color: Color) -> Bitboard { self.color_bb[color.as_usize()] }
    #[inline(always)] 
    pub const fn piece_type(&self, piece_type: PieceType) -> Bitboard { self.piece_bb[piece_type.as_usize()] }
    #[inline]
    pub fn spec(&self, piece: Piece) -> Bitboard {
        self.color(piece.color()) & self.piece_type(piece.kind())
    }
    #[inline(always)] 
    pub const fn to_move(&self) -> Color { self.to_move }
    #[inline(always)]
    pub const fn piece_on(&self, square: Square) -> Option<Piece> {
        debug_assert!(square.is_ok());
        self.pieces[square.as_u8() as usize]
    }
    #[inline(always)]
    pub const fn is_empty(&self, square: Square) -> bool {
        self.piece_on(square).is_none()
    }
    #[inline(always)]
    pub fn king(&self, color: Color) -> Square {
        Square::new(self.spec(Piece::new(PieceType::King, color)).ctz() as u8)
    }
    #[inline(always)]
    pub const fn piece_count(&self, piece: Piece) -> i8 {
        self.piece_count[piece.as_usize()]
    } 
    pub const fn total_piece_count(&self, piece_type: PieceType) -> i8 {
        self.piece_count(Piece::new(piece_type, Color::White))
        + self.piece_count(Piece::new(piece_type, Color::Black))
    }

    #[inline(always)]
    pub fn state(&self) -> Rc<State> {
        Rc::clone(&self.state)
    }

    pub fn is_ok(&self) -> bool {
        let wk = self.king(Color::White);
        let bk = self.king(Color::Black);
        if !(wk.is_ok() && bk.is_ok()) {
            return false;
        }
        todo!()
    }

    pub fn is_legal(&self, m: Move) -> bool { todo!() }
    pub fn is_pseudo_legal(&self, m: Move) -> bool { todo!() }

    pub fn play_move(&mut self, m: Move) { todo!() }
    pub fn undo_move(&mut self, m: Move) { todo!() }

    pub fn attacks_to_square(&self, square: Square) -> Bitboard { todo!() }
    pub fn attacks_to_square_from(&self, square: Square, color: Color) -> Bitboard {
        self.attacks_to_square(square) & self.color(color)
    }
}

impl State {
    pub fn new() -> Self { todo!() }

    #[inline(always)]
    pub const fn checkers(&self) -> Bitboard {
        self.checkers
    }
    #[inline(always)]
    pub const fn blockers(&self, color: Color) -> Bitboard {
        self.blockers[color.as_usize()]
    }
    #[inline(always)]
    pub const fn pinners(&self, color: Color) -> Bitboard {
        self.pinners[color.as_usize()]
    }
    #[inline(always)]
    pub const fn check_squares(&self, piece_type: PieceType) -> Bitboard {
        self.check_squares[piece_type.as_usize()]
    }
    #[inline(always)]
    pub const fn en_passant(&self) -> Option<Square> {
        self.en_passant
    }
    #[inline(always)]
    pub const fn plies_from_null(&self) -> usize {
        self.plies_from_null
    }
    #[inline(always)]
    pub const fn half_moves(&self) -> usize {
        self.half_moves
    }

    pub fn clone(&self) -> Self {
        Self {
            castle_rights: self.castle_rights,
            en_passant: self.en_passant,
            half_moves: self.half_moves,
            plies_from_null: self.plies_from_null,

            checkers: Bitboard::new(0),
            check_squares: [Bitboard::new(0); PieceType::COUNT],
            blockers: [Bitboard::new(0); Color::COUNT],
            pinners: [Bitboard::new(0); Color::COUNT],
            captured_piece: None
        }
    }
}

impl CastleRights {
    pub const W_OO: u8 = 0b0001;
    pub const W_OOO: u8 = 0b0010;
    pub const B_OO: u8 = 0b0100;
    pub const B_OOO: u8 = 0b1000;

    #[inline(always)]
    pub const fn new(value: u8) -> Self {
        Self(value)
    }

    #[inline(always)]
    pub const fn has_exact_rights(self, rights: u8) -> bool {
        self.0 & rights == rights
    }

    #[inline(always)]
    pub const fn has_any_rights(self, rights: u8) -> bool {
        self.0 & rights > 0
    }
}

impl std::ops::BitXorAssign<u8> for CastleRights {
    fn bitxor_assign(&mut self, right: u8) {
        debug_assert!(right.count_zeros() == 1);
        self.0 ^= right;
    }
}
impl std::ops::BitOrAssign<u8> for CastleRights {
    fn bitor_assign(&mut self, right: u8) {
        self.0 |= right;
    }
}
