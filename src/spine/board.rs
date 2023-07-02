use crate::spine::{Bitboard, Color, File, Move, Piece, PieceType, Rank, Square};
use crate::bitboard;

use std::fmt;
use std::mem::transmute;

use super::piece_attacks;

macro_rules! ret_false_if {
    ($cond:expr) => {
        if $cond {
            return false;
        }
    }
}

#[derive(Debug)]
pub struct Board {
    color_bb: [Bitboard; Color::COUNT],
    piece_bb: [Bitboard; PieceType::COUNT],
    pieces: [Option<Piece>; Square::COUNT],
    piece_count: [i8; PieceType::COUNT * Color::COUNT],
    to_move: Color,
    ply: usize,
}

pub type StateP = Box<State>;
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

    prev: Option<StateP>
}

impl Board {
    #[inline(always)]
    pub const fn color(&self, color: Color) -> Bitboard {
        self.color_bb[color.as_usize()]
    }
    #[inline(always)]
    pub const fn all(&self) -> Bitboard {
        self.color(Color::White).const_or(self.color(Color::Black))
    }
    #[inline(always)]
    pub const fn piece_type(&self, pt: PieceType) -> Bitboard {
        self.piece_bb[pt.as_usize()]
    }
    #[inline(always)]
    pub fn piece_type2(&self, pt1: PieceType, pt2: PieceType) -> Bitboard {
        self.piece_type(pt1) | self.piece_type(pt2)
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

    #[inline(always)]
    pub fn king(&self, color: Color) -> Square {
        self.spec(color, PieceType::King).lsb()
    }

    pub fn attacks_to(&self, square: Square) -> Bitboard {
        let knights = piece_attacks::knight_attacks(square) & self.piece_type(PieceType::Knight);
        let wpawns = piece_attacks::pawn_attacks(square, Color::White) & self.spec(Color::Black, PieceType::Pawn);
        let bpawns = piece_attacks::pawn_attacks(square, Color::Black) & self.spec(Color::White, PieceType::Pawn);
        let rooks = piece_attacks::rook_attacks(square, self.all()) & self.piece_type2(PieceType::Rook, PieceType::Queen);
        let bishops = piece_attacks::bishop_attacks(square, self.all()) & self.piece_type2(PieceType::Bishop, PieceType::Queen);
        let kings = piece_attacks::king_attacks(square) & self.piece_type(PieceType::King);

        knights | wpawns | bpawns | rooks | bishops | kings
    }

    pub fn is_legal(&self, s: &State, mv: Move) -> bool { 
        ret_false_if!(!self.is_pseudo_legal(s, mv));

        true
    }
    
    pub fn is_pseudo_legal(&self, s: &State, mv: Move) -> bool {
        let f = mv.from_square();
        let t = mv.to_square();
        let us = self.to_move();
        let them = !us;

        ret_false_if!(f.distance(t) == 0);

        let Some(mp) = self.get_piece(f) else { return false; };
        ret_false_if!(mp.color() != us);

        let captured_piece = if let Some(cp) = self.get_piece(t) {
            ret_false_if!(cp.color() == us);
            Some(cp)
        } else { None };

        if mp.kind() == PieceType::Pawn {
            match f.distance(t) {
                0 => unreachable!(),
                1 => {
                    match us {
                        Color::White => ret_false_if!(f > t),
                        Color::Black => ret_false_if!(f < t),
                    }

                    if f.file() == t.file() {
                        ret_false_if!(captured_piece.is_some());
                    } else {
                        ret_false_if!(captured_piece.is_none());
                    }
                },
                2 => {
                    ret_false_if!(f.rank() != Rank::Two.relative_to(us));
                    ret_false_if!(f.file() != t.file());
                    let sum = f.as_u8() + t.as_u8();
                    let ibw_val = sum / 2;
                    let ibw = Square::new(ibw_val);
                    ret_false_if!(self.get_piece(ibw).is_some());
                },
                _ => return false,
            }
        }

        true
    }

    // use StateP.borrow_mut() for these types of things
    pub fn compute_state(&self, s: &mut State) {
        const Z: Bitboard = Bitboard::ZERO;
        let (us, them) = (self.to_move(), !self.to_move());

        // Reset it all
        s.checkers &= Z;
        s.blockers[us.as_usize()] &= Z;
        s.blockers[them.as_usize()] &= Z;
        s.pinners[us.as_usize()] &= Z;
        s.pinners[them.as_usize()] &= Z;
        s.check_squares[PieceType::King.as_usize()] = Z;

        debug_assert!((self.attacks_to(self.king(them)) & self.color(us)).gtz() == false);
        s.checkers = self.attacks_to(self.king(us)) & self.color(them);

        let rookqs = self.piece_type2(PieceType::Rook, PieceType::Queen);
        let bishqs = self.piece_type2(PieceType::Bishop, PieceType::Queen);

        let king = self.king(us);
        let mut bs = piece_attacks::bishop_attacks(king, Z) & bishqs;
        let mut rs = piece_attacks::rook_attacks(king, Z) & rookqs;

        for (x, isb) in [(bs, true), (rs, false)].iter_mut() {
            while let Some(slider) = x.pop_lsb() {
                let line = bitboard::between::<false>(king, slider);
                if line.gtz() && !line.more_than_one() {
                    if (self.color(us) & line).gtz() {
                        s.pinners[them.as_usize()] |= Into::<Bitboard>::into(slider);
                    }
                    s.blockers[us.as_usize()] |= line;
                }
            }
        }

        s.check_squares[PieceType::Pawn.as_usize()] = piece_attacks::pawn_attacks(king, us);
        s.check_squares[PieceType::Knight.as_usize()] = piece_attacks::knight_attacks(king);
        s.check_squares[PieceType::Bishop.as_usize()] = piece_attacks::bishop_attacks(king, self.all());
        s.check_squares[PieceType::Rook.as_usize()] = piece_attacks::rook_attacks(king, self.all());
        s.check_squares[PieceType::Queen.as_usize()] = s.check_squares[PieceType::Bishop.as_usize()]
            | s.check_squares[PieceType::Rook.as_usize()];
    }

    pub fn do_move(&mut self, s: &mut StateP, mv: Move) {
        let newstate = Box::new(State::new(Some(*s)));
        *s = newstate;
        s.plies_from_null -= 1;

        let f = mv.from_square();
        let t = mv.to_square();

        let Some(mp) = self.get_piece(f) else { return; }
        if mp.is_none() || mp.color() != us {

        }

    }
    pub fn undo_move(&mut self, s: &mut StateP, mv: Move) {
        *s = s.replace().unwrap();
    }

    pub fn new<S>(fen: S, state: &mut State) -> Result<Self, BoardCreationError>
        where S: Into<String>
    {
        let mut b = Self {
            color_bb: [Bitboard::ZERO; Color::COUNT],
            piece_bb: [Bitboard::ZERO; PieceType::COUNT],
            pieces: [None; Square::COUNT],
            piece_count: [0; PieceType::COUNT * Color::COUNT],
            to_move: Color::White,
            ply: 0
        };

        let fen: String = fen.into();
        let mut chars = fen.chars();

        if fen.len() == 0 {
            return Err(BoardCreationError::NoFenGiven);
        }

        let mut ri = 7;
        let mut fi = 0;
        for c in chars.by_ref() {
            if c == ' ' { break; }
            if c == '/' {
                ri -= 1;
                fi = 0;
                continue;
            }

            if ('1'..='8').contains(&c) {
                fi += c as u8 - b'0';
                continue;
            }

            if fi >= 8 || ri < 0 {
                return Err(BoardCreationError::BoardOverflow);
            }
            let f: File = unsafe { transmute(fi as u8) };
            let r: Rank = unsafe { transmute(ri as u8) };

            let s = Square::build(f, r);

            let color = match c {
                'A'..='Z' => Color::White,
                _ => Color::Black
            };
            let pt = match c.to_ascii_lowercase() {
                'p' => PieceType::Pawn,
                'n' => PieceType::Knight,
                'b' => PieceType::Bishop,
                'r' => PieceType::Rook,
                'q' => PieceType::Queen,
                'k' => PieceType::King,
                _ => return Err(BoardCreationError::InvalidPiece)
            };

            let p = Piece::new(pt, color);
            let sqb: Bitboard = s.into();

            b.color_bb[p.color().as_usize()] |= sqb;
            b.piece_bb[p.kind().as_usize()] |= sqb;
            b.pieces[s.as_usize()] = Some(p);

            fi += 1;
        }

        if let Some(c) = chars.next() {
            b.to_move = match c {
                'w' => Color::White,
                'b' => Color::Black,
                _ => return Err(BoardCreationError::InvalidColor),
            };
        } else {
            return Err(BoardCreationError::InvalidColor);
        }

        if let Some(c) = chars.next() {
            if c != ' ' {
                return Err(BoardCreationError::InvalidCastleRights);
            }
        } else {
            return Err(BoardCreationError::InvalidCastleRights);
        }

        for c in chars.by_ref() {
            if c == ' ' { break; }
            if c == '-' {
                if state.castle_rights.has_any_rights(CastleRights::WHITE | CastleRights::BLACK) {
                    return Err(BoardCreationError::InvalidCastleRights);
                }
                break;
            }

            state.castle_rights |= c.try_into().map_err(|_e| BoardCreationError::InvalidCastleRights)?;
        }

        if let Some(c) = chars.next() {
            if c == '-' {
                state.en_passant = None;
            } else {
                if let Some(nc) = chars.next() {
                    unsafe {
                        if !('a'..='h').contains(&c) || !('1'..='8').contains(&nc) {
                            return Err(BoardCreationError::InvalidEnPassant);
                        }
                        let f: File = transmute(c as u8 - b'a');
                        let r: Rank = transmute(nc as u8 - b'1');
                        let eps = Square::build(f, r);
                        state.en_passant = Some(eps);
                    }
                } else {
                    return Err(BoardCreationError::InvalidEnPassant);
                }
             }
        } else {
            return Err(BoardCreationError::InvalidEnPassant);
        }

        if let Some(c) = chars.next() {
            if c != ' ' { return Err(BoardCreationError::InvalidNumber); }
            let left_chars = chars.collect::<Vec<char>>();
            let two_nums = left_chars.split(|&c| c == ' ').collect::<Vec<&[char]>>();
            let fnum = two_nums[0];

            if let Ok(halfmoves) = fnum.iter().collect::<String>().parse::<usize>() {
                state.half_moves = halfmoves;
            } else {
                return Err(BoardCreationError::InvalidNumber);
            }

            if let Some(snum) = two_nums.get(1) {
                if let Ok(ply) = snum.iter().collect::<String>().parse::<usize>() {
                    if ply == 0 {
                        return Err(BoardCreationError::InvalidNumber);
                    }
                    b.ply = ply;
                } else {
                    return Err(BoardCreationError::InvalidNumber);
                }
            }
        }

        Ok(b)
    }
}

impl State {
    pub const fn new(prev: Option<StateP>) -> Self {
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
    pub fn replace(self) -> Option<StateP> {
        self.prev
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CastleRights(u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardCreationError {
    NoFenGiven,
    BoardOverflow,
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
    pub const WHITE: u8 = Self::W_OO | Self::W_OOO;
    pub const BLACK: u8 = Self::B_OO | Self::B_OOO;

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

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = String::with_capacity(1024);
        let sep = "+---+---+---+---+---+---+---+---+\n";

        for r_intermediate in 0..8 {
            s += sep;
            s += "| ";
            for f in 0..8 {
                let r = 7 - r_intermediate;
                let sq = Square::new(f + (r << 3));
                assert!(sq.is_ok());

                if let Some(p) = self.get_piece(sq) {
                    s += &p.to_string();
                } else { s += " "; };

                s += " | ";
            }
            s += &(8 - r_intermediate).to_string();
            s += "\n";
        }
        s += sep;

        write!(f, "{s}")
    }
}
