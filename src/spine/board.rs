use super::Bitboard;
use super::{Square, File, Rank};
use super::{Move, MoveFlag};
use super::{Piece, PieceType};
use super::Color;
use crate::bitboard;

use std::pin::Pin;

use std::fmt;
use std::mem::transmute;
use std::ptr::NonNull;

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

#[derive(Clone, Copy)]
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

    prev: Option<NonNull<Self>>
}

impl Board {
    pub const STARTPOS: &'static str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

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

    pub fn attacks_to(&self, square: Square) -> Bitboard { self.attacks_to_bits(square, self.all()) }
    pub fn attacks_to_bits(&self, square: Square, bits: Bitboard) -> Bitboard {
        let knights = piece_attacks::knight_attacks(square) & self.piece_type(PieceType::Knight);
        let wpawns = piece_attacks::pawn_attacks(square, Color::White) & self.spec(Color::Black, PieceType::Pawn);
        let bpawns = piece_attacks::pawn_attacks(square, Color::Black) & self.spec(Color::White, PieceType::Pawn);
        let rooks = piece_attacks::rook_attacks(square, bits) & self.piece_type2(PieceType::Rook, PieceType::Queen);
        let bishops = piece_attacks::bishop_attacks(square, bits) & self.piece_type2(PieceType::Bishop, PieceType::Queen);
        let kings = piece_attacks::king_attacks(square) & self.piece_type(PieceType::King);

        knights | wpawns | bpawns | rooks | bishops | kings
    }

    pub fn is_legal(&self, s: &State, mv: Move) -> bool { 
        ret_false_if!(!self.is_pseudo_legal(s, mv));

        let f = mv.from_square();
        let t = mv.to_square();
        let us = self.to_move();
        let them = !us;
        let k = self.king(self.to_move());

        if s.checkers().gtz() {
            ret_false_if!(s.checkers().more_than_one() && f != k);
            if s.checkers().popcount() == 1 {
                let b = bitboard::between::<true>(k, s.checkers().lsb());
                ret_false_if!(!(b & t).gtz());
            }
        }

        if (s.blockers[us.as_usize()] & f).gtz() {
            let mut pns = s.pinners[them.as_usize()];
            let mut p = None;
            while let Some(pinner) = pns.pop_lsb() {
                if k.in_line2(f, pinner) {
                    p = Some(pinner);
                    break;
                }
            };

            ret_false_if!(!p.expect("Didn't find pinner for pinned piece")
                            .in_line2(t, k));
        }

        if mv.flag() == MoveFlag::Castle {
            ret_false_if!(s.checkers().gtz());
            let mut ibw = bitboard::between::<true>(f, t);
            while let Some(sq) = ibw.pop_lsb() {
                ret_false_if!((self.attacks_to(sq) & self.color(them)).gtz());
            }
        }

        if mv.flag() == MoveFlag::EnPassant {
            ret_false_if!((self.attacks_to_bits(k, self.all() ^ s.en_passant().unwrap() ^ f ^ t)
                           & self.color(!self.to_move())).gtz());
        }

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
            Some(cp.kind())
        } else { None };

        if mp.kind() == PieceType::Pawn {
            if mv.flag() == MoveFlag::EnPassant {
                ret_false_if!(f.rank() != Rank::Five.relative_to(us));
                ret_false_if!(t.rank() != Rank::Six.relative_to(us));
                ret_false_if!(f.distance(t) != 1);
                ret_false_if!(mp.kind() != PieceType::Pawn);
                ret_false_if!(s.en_passant() != Some(t));
                ret_false_if!(self.get_piece(t).is_some());
                ret_false_if!(self.get_piece(Square::build(t.file(), f.rank()))
                              != Some(PieceType::Pawn + them));
            }

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

            ret_false_if!(mv.flag() == MoveFlag::Promotion && t.rank() != Rank::Eight.relative_to(us));
            if t.rank() == Rank::Eight.relative_to(us) {
                ret_false_if!(mv.flag() != MoveFlag::Promotion);
                ret_false_if!(mv.promotion_type() == PieceType::Pawn);
                ret_false_if!(mv.promotion_type() == PieceType::King);
            }
        }

        if mp.kind() == PieceType::Bishop || mp.kind() == PieceType::Queen {
            let possible_moves = piece_attacks::bishop_attacks(f, self.all());
            ret_false_if!(possible_moves & t != t.into());
        }
        if mp.kind() == PieceType::Rook || mp.kind() == PieceType::Queen {
            let possible_moves = piece_attacks::rook_attacks(f, self.all());
            ret_false_if!(possible_moves & t != t.into());
        }

        if mp.kind() == PieceType::Knight {
            ret_false_if!(piece_attacks::knight_attacks(f) & t != t.into());
        }

        if mp.kind() == PieceType::King {
            match f.distance(t) {
                0 => unreachable!(),
                1 => (),
                2 => {
                    ret_false_if!(mv.flag() != MoveFlag::Castle);
                    ret_false_if!(f.rank() != t.rank());
                    ret_false_if!(f.rank() != Rank::One.relative_to(us));
                    ret_false_if!(f != Square::E1.relative_to(us));
                    let ibs = (f.as_u8() + t.as_u8()) / 2;
                    let ibsq = Square::new(ibs);
                    ret_false_if!(self.get_piece(ibsq).is_some());
                    let (cr, rooksq) = match t.relative_to(us) {
                        Square::G1 => {
                            match us {
                                Color::White => (CastleRights::W_OO, Square::H1),
                                Color::Black => (CastleRights::B_OO, Square::H8),
                            } 
                        },
                        Square::C1 => {
                            match us {
                                Color::White => (CastleRights::W_OOO, Square::A1),
                                Color::Black => (CastleRights::B_OOO, Square::A8),
                            } 
                        },
                        _ => return false,
                    };

                    ret_false_if!(!s.castle_rights().has_exact_rights(cr));
                    ret_false_if!((self.all() & bitboard::between::<false>(f, rooksq)).gtz());
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
        s.checkers = Z;
        s.blockers[us.as_usize()] = Z;
        s.blockers[them.as_usize()] = Z;
        s.pinners[us.as_usize()] = Z;
        s.pinners[them.as_usize()] = Z;
        s.check_squares[PieceType::King.as_usize()] = Z;

        debug_assert!((self.attacks_to(self.king(them)) & self.color(us)).gtz() == false);
        s.checkers = self.attacks_to(self.king(us)) & self.color(them);

        let rookqs = self.piece_type2(PieceType::Rook, PieceType::Queen) & self.color(them);
        let bishqs = self.piece_type2(PieceType::Bishop, PieceType::Queen) & self.color(them);

        let king = self.king(us);

        let rks = piece_attacks::rook_attacks(king, Z) & rookqs;
        let bps = piece_attacks::bishop_attacks(king, Z) & bishqs;
        for pp in rks {
            let b = bitboard::between::<false>(king, pp);
            let overlap = b & self.all();
            if overlap.popcount() == 1 {
                let l = overlap.lsb();
                let p = self.get_piece(l).expect("compute-state: rook or queen not found but detected as blocker");
                s.blockers[p.color().as_usize()] |= l;
                if p.color() == us {
                    s.pinners[them.as_usize()] |= pp;
                }
            }
        }
        for pp in bps {
            let b = bitboard::between::<false>(king, pp);
            let overlap = b & self.all();
            if overlap.popcount() == 1 {
                let l = overlap.lsb();
                let p = self.get_piece(l).expect("compute-state: bishop or queen not found but detected as blocker");
                s.blockers[p.color().as_usize()] |= l;
                if p.color() == us {
                    s.pinners[them.as_usize()] |= pp;
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

    pub fn do_move(&mut self, s: &mut State, mv: Move) {
        if !self.is_legal(s, mv) {
            return;
        }

        unsafe { *s = s.make_own_child(); }

        self.ply += 1;
        s.half_moves += 1;
        s.plies_from_null += 1;

        let f = mv.from_square();
        let t = mv.to_square();
        let flag = mv.flag();
        let promt = mv.promotion_type();
        let us = self.to_move();
        let them = !us;

        let mov = self.get_piece(f).expect("Somehow tried to move nonexistent piece");

        let cap = self.get_piece(t).map(|p| p.kind());
        let cap = if flag == MoveFlag::EnPassant { debug_assert!(cap.is_none()); Some(PieceType::Pawn) } else { cap };

        let (short, long) = if us == Color::White { (CastleRights::W_OO, CastleRights::W_OOO) }
            else { (CastleRights::B_OO, CastleRights::B_OOO) };

        if let Some(pc) = cap {
            let cap_square = if flag == MoveFlag::EnPassant { Square::build(t.file(), Rank::Five.relative_to(us)) } else { t };
            self.remove_piece(cap_square);

            if pc == PieceType::Rook {
                match t.relative_to(us) {
                    Square::H8 => if s.castle_rights().has_exact_rights(short) {
                        s.castle_rights.remove_right(short);
                    },
                    Square::A8 => if s.castle_rights().has_exact_rights(long) {
                        s.castle_rights.remove_right(long);
                    },
                    _ => ()
                }
            }

            s.half_moves = 0;
        }

        debug_assert_eq!(mov.color(), us);

        if flag == MoveFlag::Castle {
            debug_assert_eq!(mov.kind(), PieceType::King);
            debug_assert_eq!(f.distance(t), 2);
            self.do_castle::<true>(s, us, f, t);
        } else {
            self.remove_piece(f);
            self.add_piece(t, mov);
        }

        if s.en_passant().is_some() {
            s.en_passant = None;
        }

        if mov.kind() == PieceType::Pawn {
            s.half_moves = 0;

            let pep = Square::build(f.file(), Rank::Three.relative_to(us));
            if f.distance(t) == 2 &&
                (piece_attacks::pawn_attacks_by_board(Bitboard::from(pep), us)
                 & self.spec(!us, PieceType::Pawn)).gtz()
            {
                s.en_passant = Some(pep);
            } else if flag == MoveFlag::Promotion {
                let promp = promt + us;
                debug_assert!(t.rank() == Rank::Eight.relative_to(us));
                self.remove_piece(t);
                self.add_piece(t, promp);
            }
        }

        if mov.kind() == PieceType::King {
            let _ = [short, long].into_iter().map(|cr| s.castle_rights.remove_right(cr));
        }

        s.captured_piece = cap;
        self.compute_state(s);

        self.to_move = them;
    }

    pub fn apply_moves(&mut self, s: &mut State, moves: &[Move]) -> Result<(), Move> {
        for &m in moves.iter() {
            if !self.is_legal(s, m) {
                return Err(m);
            }

            self.do_move(s, m);
        }

        Ok(())
    }

    pub fn undo_move(&mut self, s: &mut State, mv: Move) {
        self.to_move = !self.to_move;
        let us = self.to_move;

        let f = mv.from_square();
        let t = mv.to_square();
        let flag = mv.flag();

        let mov = self.get_piece(t).expect("undo-move: could not find moved piece");
        debug_assert!(self.get_piece(f).is_none());
        debug_assert!(s.captured_piece != Some(PieceType::King));

        if flag == MoveFlag::Promotion {
            debug_assert!(t.rank() == Rank::Eight.relative_to(us));
            debug_assert!(mov.kind() == mv.promotion_type());
            self.remove_piece(t);
            let p = PieceType::Pawn + us;
            self.add_piece(t, p);
        }
        if flag == MoveFlag::Castle {
            self.do_castle::<false>(s, us, f, t);
        } else {
            _ = self.remove_piece(t);
            self.add_piece(f, mov);
            if let Some(pt) = s.captured_piece {
                let csq = if flag == MoveFlag::EnPassant {
                    debug_assert!(f.rank() == Rank::Six.relative_to(us));
                    Square::build(t.file(), f.rank())
                } else { t };
                self.add_piece(csq, pt + !us);
            }
        }

        unsafe { *s = *s.collapse().unwrap() };
        self.ply -= 1;
    }

    pub fn add_piece(&mut self, s: Square, p: Piece) {
        self.color_bb[p.color().as_usize()] |= s;
        self.piece_bb[p.kind().as_usize()] |= s;
        self.pieces[s.as_usize()] = Some(p);
    }
    pub fn remove_piece(&mut self, s: Square) -> Option<Piece> {
        let popt = self.get_piece(s);
        if let Some(p) = popt {
            self.color_bb[p.color().as_usize()] ^= s;
            self.piece_bb[p.kind().as_usize()] ^= s;
            self.pieces[s.as_usize()] = None;
        }

        popt
    }

    fn do_castle<const APPLY: bool>(&mut self, s: &mut State, us: Color, from: Square, to: Square) {
        let is_ks = from < to;

        let rfrom = if is_ks { Square::H1.relative_to(us) } else { Square::A1.relative_to(us) };
        let rto = if is_ks { Square::F1.relative_to(us) } else { Square::D1.relative_to(us) };

        let rook_r = if APPLY { rfrom } else { rto };
        let rook_t = if APPLY { rto } else { rfrom };

        let kfr = if APPLY { from } else { to };
        let kto = if APPLY { to } else { from };

        let k = self.remove_piece(kfr).expect("No King Found in do_castle");
        let r = self.remove_piece(rook_r).expect("No Rook Found in do_castle");
        debug_assert!(k.kind() == PieceType::King);
        debug_assert!(r.kind() == PieceType::Rook);

        self.add_piece(kto, k);
        self.add_piece(rook_t, r);
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

            let p = pt + color;
            let sqb: Bitboard = s.into();

            b.add_piece(s, p);

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

            state.castle_rights.add_right(c.try_into().map_err(|_| BoardCreationError::InvalidCastleRights)?);
        }

        if state.castle_rights() == CastleRights(0) {
            if let Some(c) = chars.next() { // Consume the next char
                if c != ' ' {
                    return Err(BoardCreationError::InvalidEnPassant);
                }
            } else {
                return Err(BoardCreationError::InvalidEnPassant);
            }
        }

        if let Some(c) = chars.next() {
            if c == '-' {
                state.en_passant = None;
            } else if let Some(nc) = chars.next() {
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
        } else { return Err(BoardCreationError::InvalidEnPassant); }

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

        b.compute_state(state);
        Ok(b)
    }
}

impl State {
    pub const fn new() -> Self {
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
            prev: None,
        };
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

    unsafe fn make_own_child(self) -> Self {
        let mut s = self; // Copy!
        // Make a heap-allocated State variable
        let leaked_state: &'static mut Self = Box::leak(Box::new(self));
        let nn_ptr = NonNull::new_unchecked(leaked_state);
        s.prev = Some(nn_ptr);
        s
    }
    unsafe fn collapse(self) -> Option<Box<Self>> {
        // We use a Box<> here to not leak memory, I think?
        self.prev.map(|p| Box::from_raw(p.as_ptr()))
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

    pub fn remove_right(&mut self, right: u8) {
        self.0 &= !right;
    }
    pub fn add_right(&mut self, right: u8) {
        self.0 |= right;
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
        s += "  A   B   C   D   E   F   G   H\n";
        s += &format!("To Move: {:?}", self.to_move);

        write!(f, "{s}")
    }
}


pub struct BAS<'a>(pub &'a Board, pub &'a State);
impl<'a> fmt::Display for BAS<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = format!("{}", self.0);
        s += &format!("Checkers: {:#x?}", self.1.checkers.as_u64());
        write!(f, "{s}")
    }
}
