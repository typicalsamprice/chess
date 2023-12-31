use crate::piece_attacks;
use crate::prelude::*;

use std::fmt;
use std::mem::transmute;
use std::ptr::NonNull;

macro_rules! ret_false_if {
    ($cond:expr) => {
        if $cond {
            return false;
        }
    };
}

#[derive(Clone, Debug)]
pub struct Board {
    color_bb: [Bitboard; Color::COUNT],
    piece_bb: [Bitboard; PieceType::COUNT],
    pieces: [Option<Piece>; Square::COUNT],
    piece_count: [i8; PieceType::COUNT * Color::COUNT],
    to_move: Color,
    ply: usize,

    history: Vec<Move>,

    is960: bool,
}

// Safety: State can be send because the NonNull<State> inside of it
// is never aliased and impossible to access for others.
//
// This is important because of multithreading in the future, without weird
// (de)serializing.
unsafe impl Send for State {}

#[derive(Clone, Copy, Debug)]
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

    prev: Option<NonNull<Self>>,
}

impl Board {
    pub const STARTPOS: &'static str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    pub const KIWIPETE: &'static str =
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -";

    #[inline(always)]
    pub const fn color(&self, color: Color) -> Bitboard {
        self.color_bb[color.to_usize()]
    }
    #[inline(always)]
    pub const fn all(&self) -> Bitboard {
        self.color(Color::White).const_or(self.color(Color::Black))
    }
    #[inline(always)]
    pub const fn piece_type(&self, pt: PieceType) -> Bitboard {
        self.piece_bb[pt.to_usize()]
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
        self.pieces[square.to_usize()]
    }
    #[inline(always)]
    pub const fn piece_count(&self, color: Color, pt: PieceType) -> i8 {
        self.piece_count[PieceType::COUNT * color.to_usize() + pt.to_usize()]
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
        debug_assert!(square.is_ok());
        self.attacks_to_bits(square, self.all())
    }
    pub fn attacks_to_bits(&self, square: Square, bits: Bitboard) -> Bitboard {
        debug_assert!(square.is_ok());
        let knights = piece_attacks::knight_attacks(square) & self.piece_type(PieceType::Knight);
        let wpawns = piece_attacks::pawn_attacks(square, Color::White)
            & self.spec(Color::Black, PieceType::Pawn);
        let bpawns = piece_attacks::pawn_attacks(square, Color::Black)
            & self.spec(Color::White, PieceType::Pawn);
        let kings = piece_attacks::king_attacks(square) & self.piece_type(PieceType::King);

        knights | wpawns | bpawns | kings | self.sliders_to(square, bits)
    }

    fn sliders_to(&self, square: Square, bits: Bitboard) -> Bitboard {
        let rooks = piece_attacks::rook_attacks(square, bits)
            & self.piece_type2(PieceType::Rook, PieceType::Queen);
        let bishops = piece_attacks::bishop_attacks(square, bits)
            & self.piece_type2(PieceType::Bishop, PieceType::Queen);

        rooks | bishops
    }

    fn str_history(&self) -> String {
        self.history
            .iter()
            .map(|m| format!("{m}"))
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub fn unblocked_castle(&self, right: CastleRight) -> bool {
        let CastleRight {
            king_from,
            king_to,
            rook_from,
            rook_to,
        } = right;

        let btw_kf_and_rf = Bitboard::between::<false>(king_from, rook_from);
        if (self.all() & btw_kf_and_rf).gtz() {
            return false;
        }

        let king_travel = Bitboard::between::<true>(king_from, king_to) & self.all();
        if king_travel.gtz() && (king_travel ^ rook_from).gtz() {
            return false;
        }

        true
    }

    /// Check whether a [`Move`] is legal given a position.
    pub fn is_legal(&self, s: &State, mv: Move) -> bool {
        #[cfg(debug_assertions)]
        ret_false_if!(!self.is_pseudo_legal(s, mv));

        let f = mv.from_square();
        let t = mv.to_square();
        let us = self.to_move();
        let them = !us;
        let k = self.king(us);

        match s.checkers().popcount() {
            0 => (),
            1 => {
                let to_checker = Bitboard::between::<true>(k, s.checkers().lsb());
                if f != k {
                    // TODO: Make this less hacky
                    if let Some(ep) = s.en_passant() {
                        ret_false_if!((s.checkers() & (ep + ShiftDir::Forward(them))).inner() == 0);
                    } else {
                        // Must capture or interpose
                        ret_false_if!(!(to_checker & t).gtz());
                    }
                }
            }
            2 => {
                ret_false_if!(f != k)
            }
            3.. => unreachable!(),
        }

        // King cannot walk into check
        if f == k {
            let rk = if mv.flag() == MoveFlag::Castle {
                // TODO: Allow C960 castling in the future.
                let mv_rook = Bitboard::between::<true>(f, t);
                mv_rook
            } else {
                Bitboard::ZERO
            };
            ret_false_if!((self.attacks_to_bits(t, self.all() ^ f ^ rk) & self.color(them)).gtz());
        }

        if (s.blockers(us) & f).gtz() {
            let pinners = s.pinners(them);
            // Possible pinners, up to 2. This makes the iterator iterate less
            // PERF: Is this mask worth it?
            let pinners = Bitboard::line(f, k) & pinners;

            let mut pinner = f;
            for pin in pinners {
                if (Bitboard::between::<false>(pin, k) & f).gtz() {
                    pinner = pin;
                    break;
                }
            }

            debug_assert!(pinner != f);
            // The pinned piece must move on the line, or take the piece
            ret_false_if!(!(Bitboard::between::<true>(k, pinner) & t).gtz());
        }

        if mv.flag() == MoveFlag::EnPassant {
            let caps = Square::build(t.file(), f.rank());
            let wo_extras = self.all() ^ f ^ caps ^ t;
            // We don't want to count attacks from the taken pawn
            let atts = self.attacks_to_bits(k, wo_extras) & !Bitboard::from(caps);
            ret_false_if!((atts & self.color(them)).gtz());
        } else if mv.flag() == MoveFlag::Castle {
            // Cannot castle through check
            let squares = Bitboard::between::<true>(f, t);
            let bits = self.all() ^ f;
            let thbits = self.color(them);
            for x in squares {
                ret_false_if!((self.attacks_to_bits(x, bits) & thbits).gtz());
            }
        }

        true
    }

    pub fn is_pseudo_legal(&self, s: &State, mv: Move) -> bool {
        use MoveFlag::*;
        use PieceType::*;

        let f = mv.from_square();
        let t = mv.to_square();
        let flag = mv.flag();
        let us = self.to_move();
        let them = !us;
        let k = self.king(us);

        ret_false_if!(f == t);

        let mov = self.get_piece(f);
        ret_false_if!(mov.is_none());
        let mov = mov.unwrap();

        let capsq = if flag == EnPassant {
            Square::build(t.file(), f.rank())
        } else {
            t
        };
        let cap = self.get_piece(capsq);

        if let Some(cap) = cap {
            ret_false_if!(cap.color() == us);

            // EP
            if capsq != t {
                ret_false_if!(cap != Piece::new(Pawn, them));
                ret_false_if!(mov != Piece::new(Pawn, us));
            }
        }

        // The 960 check is because you could castle "through" your own rook
        if !self.is960 || self.get_piece(t) != Some(Piece::new(Rook, us)) {
            let between_to_from = Bitboard::between::<false>(f, t);
            // Can't move THROUGH a piece.
            ret_false_if!((between_to_from & self.all()).gtz());
        }

        // TODO: Test this logic
        if flag == Castle {
            let Some(right) = s.castle_rights().find(f, t) else {
                return false;
            };

            let btw_rook_and_king =
                Bitboard::between::<false>(right.king_from, right.rook_from) & self.all();
            ret_false_if!(btw_rook_and_king.gtz());

            let king_travels =
                Bitboard::between::<true>(right.king_from, right.king_to) & self.all();
            if king_travels.gtz()
                && !(self.is960
                    && right.king_to == right.rook_from
                    && !(king_travels ^ right.rook_from).gtz())
            {
                return false;
            }
        }

        true
    }

    fn compute_state(&self, s: &mut State) {
        const Z: Bitboard = Bitboard::ZERO;
        let us = self.to_move();
        let them = !us;

        debug_assert_eq!(self.attacks_to(self.king(them)) & self.color(us), Z);

        s.checkers = Z;
        s.blockers[us.to_usize()] = Z;
        s.blockers[them.to_usize()] = Z;
        s.check_squares[PieceType::King.to_usize()] = Z;

        let king = self.king(us);
        debug_assert!(king.is_ok());

        s.checkers = self.attacks_to(king) & self.color(them);

        s.blockers[Color::White.to_usize()] =
            self.pinner_blocker(&mut s.pinners[Color::Black.to_usize()], Color::White);
        s.blockers[Color::Black.to_usize()] =
            self.pinner_blocker(&mut s.pinners[Color::White.to_usize()], Color::Black);

        s.check_squares[PieceType::Pawn.to_usize()] = piece_attacks::pawn_attacks(king, us);
        s.check_squares[PieceType::Knight.to_usize()] = piece_attacks::knight_attacks(king);
        s.check_squares[PieceType::Bishop.to_usize()] =
            piece_attacks::bishop_attacks(king, self.all());
        s.check_squares[PieceType::Rook.to_usize()] = piece_attacks::rook_attacks(king, self.all());
        s.check_squares[PieceType::Queen.to_usize()] = s.check_squares
            [PieceType::Bishop.to_usize()]
            | s.check_squares[PieceType::Rook.to_usize()];
    }

    fn pinner_blocker(&self, pinners: &mut Bitboard, color: Color) -> Bitboard {
        let mut blockers = Bitboard::ZERO;
        *pinners = Bitboard::ZERO;

        let king = self.king(color);
        let thematts = self.sliders_to(king, Bitboard::ZERO) & self.color(!color);

        for slider in thematts {
            let line_tk = Bitboard::between::<false>(slider, king) & self.all();
            if line_tk.gtz() {
                if line_tk.more_than_one() {
                    continue;
                }
                blockers |= line_tk; // That's a blocker!
                *pinners |= slider; // That's a pinner
            }
        }

        blockers
    }

    pub fn do_move(&mut self, s: &mut State, mv: Move) -> Result<(), Move> {
        if !self.is_legal(s, mv) {
            return Err(mv);
        }

        unsafe {
            *s = s.make_own_child();
        }

        self.ply += 1;
        s.half_moves += 1;
        s.plies_from_null += 1;

        let f = mv.from_square();
        let t = mv.to_square();
        let flag = mv.flag();
        let promt = mv.promotion_type();
        let us = self.to_move();
        let them = !us;

        let mov = self
            .get_piece(f)
            .expect("Somehow tried to move nonexistent piece");
        debug_assert_eq!(mov.color(), us);

        let cap = self.get_piece(t).map(|p| p.kind());
        let cap = if flag == MoveFlag::EnPassant {
            debug_assert!(cap.is_none());
            Some(PieceType::Pawn)
        } else {
            cap
        };

        if let Some(pc) = cap {
            let cap_square = if flag == MoveFlag::EnPassant {
                Square::build(t.file(), Rank::Five.relative_to(us))
            } else {
                t
            };

            _ = self.remove_piece(cap_square).map(|x| x.kind());

            if pc == PieceType::Rook {
                s.castle_rights.mut_rights_for(them).map(|right| {
                    if let Some(r) = right {
                        if r.rook_from == cap_square {
                            *right = None;
                        }
                    }
                });
            }

            s.half_moves = 0;
        }

        if flag == MoveFlag::Castle {
            debug_assert_eq!(mov.kind(), PieceType::King);
            debug_assert_eq!(f.distance(t), 2);
            self.do_castle::<true>(s, us, f, t);
        } else {
            let _ = self.remove_piece(f);
            self.add_piece(t, mov);
        }

        // OPT: Is this faster just to skip the check?
        if s.en_passant().is_some() {
            s.en_passant = None;
        }

        if mov.kind() == PieceType::Pawn {
            s.half_moves = 0;

            let pep = Square::build(f.file(), Rank::Three.relative_to(us));
            // OPT: Should we bother with the extra check? Might be faster just to assign.
            if f.distance(t) == 2
                && (piece_attacks::pawn_attacks_by_board(Bitboard::from(pep), us)
                    & self.spec(!us, PieceType::Pawn))
                .gtz()
            {
                s.en_passant = Some(pep);
            } else if flag == MoveFlag::Promotion {
                let promp = promt + us;
                debug_assert!(t.rank() == Rank::Eight.relative_to(us));
                let pr = self.remove_piece(t);
                debug_assert_eq!(pr, Some(Piece::new(PieceType::Pawn, us)));
                self.add_piece(t, promp);
            }
        }

        if mov.kind() == PieceType::King {
            s.castle_rights
                .mut_rights_for(us)
                .map(|right| *right = None);
        } else if mov.kind() == PieceType::Rook {
            for x in s.castle_rights.mut_rights_for(us) {
                if let Some(right) = x {
                    if right.rook_from == f {
                        *x = None;
                    }
                }
            }
        }

        s.captured_piece = cap;
        self.to_move = them;
        self.history.push(mv);
        self.compute_state(s);

        Ok(())
    }

    pub fn apply_moves(&mut self, s: &mut State, moves: &[Move]) -> Result<(), Move> {
        for &m in moves.iter() {
            self.do_move(s, m)?;
        }

        Ok(())
    }

    pub fn undo_move(&mut self, s: &mut State, mv: Move) {
        self.to_move = !self.to_move;
        let us = self.to_move;

        let f = mv.from_square();
        let t = mv.to_square();
        let flag = mv.flag();

        let mov = self
            .get_piece(t)
            .expect("undo-move: could not find moved piece");

        debug_assert!(self.get_piece(f).is_none());
        debug_assert!(s.captured_piece != Some(PieceType::King));

        let prev_mov = self.history.pop();
        debug_assert_eq!(Some(mv), prev_mov);

        if flag == MoveFlag::Promotion {
            debug_assert!(t.rank() == Rank::Eight.relative_to(us));
            if mov.kind() != mv.promotion_type() {
                println!("Expected: {:?} Got: {:?}", mv.promotion_type(), mov.kind());
                println!("Last move: {}. Given: {mv}", prev_mov.unwrap());
                println!("{}", self.str_history());
                panic!()
            }
            debug_assert!(mov.kind() == mv.promotion_type());
            _ = self.remove_piece(t);
            let p = PieceType::Pawn + us;
            self.add_piece(t, p);
        }

        if flag == MoveFlag::Castle {
            self.do_castle::<false>(s, us, f, t);
        } else {
            let x = self.remove_piece(t); // We have corrected the type for promos
            self.add_piece(f, x.unwrap());

            if let Some(pt) = s.captured_piece {
                let csq = if flag == MoveFlag::EnPassant {
                    debug_assert!(f.rank() == Rank::Five.relative_to(us));
                    Square::build(t.file(), f.rank())
                } else {
                    t
                };
                self.add_piece(csq, pt + !us);
            }
        }

        // Safety: FIXME: Not sure
        unsafe { *s = *s.collapse().unwrap() };
        self.ply -= 1;
    }

    pub fn add_piece(&mut self, s: Square, p: Piece) {
        self.color_bb[p.color().to_usize()] |= s;
        self.piece_bb[p.kind().to_usize()] |= s;
        self.pieces[s.to_usize()] = Some(p);
    }
    pub fn remove_piece(&mut self, s: Square) -> Option<Piece> {
        let popt = self.get_piece(s);
        if let Some(p) = popt {
            self.color_bb[p.color().to_usize()] ^= s;
            self.piece_bb[p.kind().to_usize()] ^= s;
            self.pieces[s.to_usize()] = None;
        }

        popt
    }

    fn do_castle<const APPLY: bool>(&mut self, s: &mut State, us: Color, from: Square, to: Square) {
        let is_ks = from < to;

        let rfrom = if is_ks {
            Square::H1.relative_to(us)
        } else {
            Square::A1.relative_to(us)
        };
        let rto = if is_ks {
            Square::F1.relative_to(us)
        } else {
            Square::D1.relative_to(us)
        };

        let rook_r = if APPLY { rfrom } else { rto };
        let rook_t = if APPLY { rto } else { rfrom };

        let kfr = if APPLY { from } else { to };
        let kto = if APPLY { to } else { from };

        let k = self.remove_piece(kfr).expect("No King Found in do_castle");
        let r = self
            .remove_piece(rook_r)
            .expect("No Rook Found in do_castle");
        debug_assert!(k.kind() == PieceType::King);
        debug_assert!(r.kind() == PieceType::Rook);

        self.add_piece(kto, k);
        self.add_piece(rook_t, r);
    }

    /// Create a new [`Board`] and set up a proper [`State`] for
    /// the board given some [FEN](https://en.wikipedia.org/wiki/Forsyth-Edwards_Notation)
    pub fn new<S>(fen: S, state: &mut State) -> Result<Self, BoardCreationError>
    where
        S: Into<String>,
    {
        let mut b = Self {
            color_bb: [Bitboard::ZERO; Color::COUNT],
            piece_bb: [Bitboard::ZERO; PieceType::COUNT],
            pieces: [None; Square::COUNT],
            piece_count: [0; PieceType::COUNT * Color::COUNT],
            to_move: Color::White,
            ply: 0,

            history: Vec::with_capacity(2usize.pow(11)),
            is960: false,
        };

        let fen: String = fen.into();
        let mut chars = fen.chars();

        if fen.len() == 0 {
            return Err(BoardCreationError::NoFenGiven);
        }

        let mut ri = 7;
        let mut fi = 0;
        for c in chars.by_ref() {
            if c == ' ' {
                break;
            }
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
            let f: File = unsafe { transmute(fi) };
            let r: Rank = unsafe { transmute(ri as u8) };

            let s = Square::build(f, r);

            let color = match c {
                'A'..='Z' => Color::White,
                _ => Color::Black,
            };
            let pt = match c.to_ascii_lowercase() {
                'p' => PieceType::Pawn,
                'n' => PieceType::Knight,
                'b' => PieceType::Bishop,
                'r' => PieceType::Rook,
                'q' => PieceType::Queen,
                'k' => PieceType::King,
                _ => return Err(BoardCreationError::InvalidPiece),
            };

            let p = pt + color;

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

        let mut last: char = 0 as char;
        for c in chars.by_ref() {
            last = c;
            if c == ' ' {
                break;
            }
            if c == '-' {
                if !state.castle_rights.all_none() {
                    return Err(BoardCreationError::InvalidCastleRights);
                }
                break;
            }

            match c {
                'K' => {
                    state.castle_rights.white_short = Some(CastleRight {
                        king_from: Square::E1,
                        king_to: Square::G1,
                        rook_from: Square::H1,
                        rook_to: Square::F1,
                    })
                }
                'Q' => {
                    state.castle_rights.white_long = Some(CastleRight {
                        king_from: Square::E1,
                        king_to: Square::C1,
                        rook_from: Square::A1,
                        rook_to: Square::D1,
                    })
                }
                'k' => {
                    state.castle_rights.black_short = Some(CastleRight {
                        king_from: Square::E8,
                        king_to: Square::G8,
                        rook_from: Square::H8,
                        rook_to: Square::F8,
                    })
                }
                'q' => {
                    state.castle_rights.black_long = Some(CastleRight {
                        king_from: Square::E8,
                        king_to: Square::C8,
                        rook_from: Square::A8,
                        rook_to: Square::D8,
                    })
                }
                _ => return Err(BoardCreationError::InvalidCastleRights),
            }
        }

        if last == 0 as char {
            return Err(BoardCreationError::InvalidCastleRights);
        }

        if last != ' ' {
            if let Some(c) = chars.next() {
                // Consume the next char
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
        } else {
            return Err(BoardCreationError::InvalidEnPassant);
        }

        if let Some(c) = chars.next() {
            if c != ' ' {
                return Err(BoardCreationError::InvalidNumber);
            }
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

    pub fn clone(&self, state: &State) -> (Self, State) {
        let mut s = state.clone();
        s.prev = None;
        let board = <Self as Clone>::clone(self);

        (board, s)
    }
}

impl State {
    /// Create a new blank [`State`] with no parent
    pub const fn new() -> Self {
        Self {
            castle_rights: CastleRights::new(),
            en_passant: None,
            half_moves: 0,
            plies_from_null: 0,

            checkers: Bitboard::ZERO,
            check_squares: [Bitboard::ZERO; PieceType::COUNT],
            blockers: [Bitboard::ZERO; Color::COUNT],
            pinners: [Bitboard::ZERO; Color::COUNT],
            captured_piece: None,
            prev: None,
        }
    }

    /// Get the current castling permissions in the form of a [`CastleRights`]
    pub const fn castle_rights(&self) -> CastleRights {
        self.castle_rights
    }

    /// Get the current en passant square (the one that the capture takes place on)
    pub const fn en_passant(&self) -> Option<Square> {
        self.en_passant
    }

    /// Get the current count of plies from null (null being the start)
    pub const fn plies_from_null(&self) -> usize {
        self.plies_from_null
    }

    /// Get the [`Bitboard`] of the pieces delivering check currently
    pub const fn checkers(&self) -> Bitboard {
        self.checkers
    }

    /// Get the [`Bitboard`] of the pieces being pinned to their own king
    pub const fn blockers(&self, color: Color) -> Bitboard {
        self.blockers[color.to_usize()]
    }

    /// Get the [`Bitboard`] of the pieces of [`Color`] pinning against the opposite king
    pub const fn pinners(&self, color: Color) -> Bitboard {
        self.pinners[color.to_usize()]
    }

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
        // TODO: Use miri?
        self.prev.map(|p| Box::from_raw(p.as_ptr()))
    }
}

#[derive(Debug, Clone, Copy)]
/// A wrapper that denotes certain castling privileges
pub struct CastleRights {
    pub white_short: Option<CastleRight>,
    pub white_long: Option<CastleRight>,
    pub black_short: Option<CastleRight>,
    pub black_long: Option<CastleRight>,
}

#[derive(Debug, Clone, Copy)]
pub struct CastleRight {
    pub king_from: Square,
    pub king_to: Square,
    pub rook_from: Square,
    pub rook_to: Square,
}

impl CastleRights {
    pub const fn new() -> Self {
        Self {
            white_short: None,
            white_long: None,
            black_short: None,
            black_long: None,
        }
    }

    fn each(&self) -> [Option<CastleRight>; 4] {
        [
            self.white_short,
            self.white_long,
            self.black_short,
            self.black_long,
        ]
    }

    pub const fn all_none(&self) -> bool {
        self.white_short.is_none()
            && self.white_long.is_none()
            && self.black_short.is_none()
            && self.black_long.is_none()
    }

    pub const fn rights_for(&self, color: Color) -> [Option<CastleRight>; 2] {
        match color {
            Color::White => [self.white_short, self.white_long],
            Color::Black => [self.black_short, self.black_long],
        }
    }

    pub fn mut_rights_for(&mut self, color: Color) -> [&mut Option<CastleRight>; 2] {
        match color {
            Color::White => [&mut self.white_short, &mut self.white_long],
            Color::Black => [&mut self.black_short, &mut self.black_long],
        }
    }

    pub fn find(&self, king_from: Square, king_to: Square) -> Option<CastleRight> {
        for x in self.each() {
            if let Some(x) = x {
                if x.king_from == king_from && x.king_to == king_to {
                    return Some(x);
                }
            }
        }

        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardCreationError {
    NoFenGiven,
    BoardOverflow,
    InvalidPiece,
    InvalidColor,
    InvalidCastleRights,
    InvalidEnPassant,
    InvalidNumber,
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
                } else {
                    s += " ";
                };

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
