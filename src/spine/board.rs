use crate::spine::{Bitboard, Color, File, Move, Piece, PieceType, Rank, Square};
use std::rc::Rc;

#[derive(Debug)]
pub struct Board {
    color_bb: [Bitboard; Color::COUNT],
    piece_bb: [Bitboard; PieceType::COUNT],
    pieces: [Option<Piece>; Square::COUNT],
    piece_count: [i8; PieceType::COUNT * Color::COUNT],
    to_move: Color,
    ply: usize,
}

pub type HState = Box<State>;
pub struct State {
    castle_rights: CastleRights,
    en_passant: Option<Square>,
    half_moves: usize,
    plies_from_null: usize,

    checkers: Bitboard,
    check_squares: [Bitboard; PieceType::COUNT],
    blockers: [Bitboard; Color::COUNT],
    pinners: [Bitboard; Color::COUNT],
    captured_piece: Option<PieceType>,

    prev: Option<HState>
}

impl Board {
    #[inline(always)]
    pub const fn color(&self, color: Color) -> Bitboard {
        self.color_bb[color.as_usize()]
    }
    #[inline(always)]
    pub const fn piece_type(&self, pt: PieceType) -> Bitboard {
        self.piece_bb[pt.as_usize()]
    }
    #[inline(always)]
    pub fn spec(&self, color: Color, pt: PieceType) -> Bitboard {
        self.color(color) & self.piece_type(pt)
    }
    #[inline(always)]
    pub const fn get_piece(&self, square: Square) -> Option<Piece> {
        debug_assert!(square.is_ok());
        self.pieces[square.as_usize()]
    }
    #[inline(always)]
    pub const fn piece_count(&self, color: Color, pt: PieceType) -> i8 {
        self.piece_count[PieceType::COUNT * color.as_usize() + pt.as_usize()]
    }
    #[inline(always)]
    pub const fn to_move(&self) -> Color {
        self.to_move
    }
    #[inline(always)]
    pub const fn ply(&self) -> usize {
        self.ply
    }

    pub fn king(&self, color: Color) -> Square {
        self.spec(color, PieceType::King).lsb()
    }

    pub fn is_legal(&self, s: &State, mv: Move) -> bool { todo!() }
    pub fn is_pseudo_legal(&self, s: &State, mv: Move) -> bool { todo!() }

    // use HState.borrow_mut() for these types of things
    pub fn compute_state(&self, s: &mut State) {}

    pub fn do_move(&mut self, s: &mut State, mv: Move) {}
    pub fn undo_move(&mut self, s: &mut State, mv: Move) {}
}

impl State {
    pub const fn new(prev: Option<HState>) -> Self {
        let mut s = Self {
            castle_rights: CastleRights::new(0),
            en_passant: None,
            half_moves: 0,
            plies_from_null: 0,

            checkers: Bitboard::ZERO,
            check_squares: [Bitboard::ZERO; PieceType::COUNT],
            blockers: [Bitboard::ZERO; Color::COUNT],
            pinners: [Bitboard::ZERO; Color::COUNT],
            captured_piece: None,
            prev,
        };

        if let Some(st) = s.prev.as_ref() {
            s.en_passant = st.en_passant;
            s.castle_rights = st.castle_rights;
            s.half_moves = st.half_moves;
            s.plies_from_null = st.plies_from_null;
        }

        s
    }

    #[inline(always)]
    pub const fn castle_rights(&self) -> CastleRights {
        self.castle_rights
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
    pub const fn checkers(&self) -> Bitboard { self.checkers }
    #[inline(always)]
    pub const fn blockers(&self, color: Color) -> Bitboard { self.blockers[color.as_usize()] }
    #[inline(always)]
    pub const fn pinners(&self, color: Color) -> Bitboard { self.pinners[color.as_usize()] }

    #[inline(always)]
    pub fn prev(&self) -> Option<&Self> {
        self.prev.as_deref()
    }

    #[inline(always)]
    pub fn destroy_and_get_prev(self) -> Option<HState> {
        self.prev
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CastleRights(u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardCreationError {
    NoFenGiven,
    InvalidPiece,
    InvalidColor,
    InvalidCastleRights,
    InvalidEnPassant,
    InvalidNumber
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

pub struct InvalidCastleRightsChar;
impl TryFrom<char> for CastleRights {
    type Error = InvalidCastleRightsChar;
    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'K' => Ok(Self(Self::W_OO)),
            'Q' => Ok(Self(Self::W_OOO)),
            'k' => Ok(Self(Self::B_OO)),
            'q' => Ok(Self(Self::B_OOO)),
            '-' => Ok(Self(0)),
            _ => Err(InvalidCastleRightsChar)
        } 
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
