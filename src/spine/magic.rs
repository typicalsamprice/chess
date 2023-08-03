use crate::macros::pext_u64;
use crate::prelude::*;
use crate::spine::prng::PRNG;

#[derive(Debug, Clone, Copy)]
struct Magic {
    magic: Bitboard,
    mask: Bitboard,
    shift: u32,
    attacks: &'static [Bitboard],
    width: usize,
    ptr: usize,
}

impl Magic {
    #[inline(always)]
    pub(crate) const fn new() -> Self {
        Self {
            magic: Bitboard::ZERO,
            mask: Bitboard::ZERO,
            shift: 0,
            attacks: &[],
            width: 0,
            ptr: 0,
        }
    }

    #[cfg(feature = "pext")]
    fn offset(&self, occupied: Bitboard) -> usize {
        pext_u64(occupied.inner(), self.mask.inner()) as usize
    }

    #[cfg(not(feature = "pext"))]
    fn offset(&self, occupied: Bitboard) -> usize {
        let masked = occupied & self.mask;
        let v = masked * self.magic;
        v.inner() as usize >> self.shift
    }
}

static mut ROOK_MAGICS: [Magic; 64] = [Magic::new(); 64];
static mut BISHOP_MAGICS: [Magic; 64] = [Magic::new(); 64];

static mut ROOK_TABLE: [Bitboard; 0x19000] = [Bitboard::ZERO; 0x19000];
static mut BISHOP_TABLE: [Bitboard; 0x1480] = [Bitboard::ZERO; 0x1480];

const SEEDS: [u64; 8] = [728, 10316, 55013, 32803, 12281, 15100, 16645, 255];

pub(crate) fn initialize_magics() {
    unsafe {
        init_magics::<false>();
        init_magics::<true>();
    }
}

unsafe fn init_magics<const IS_ROOK: bool>() {
    let magics = if IS_ROOK {
        &mut ROOK_MAGICS
    } else {
        &mut BISHOP_MAGICS
    };
    let table = if IS_ROOK {
        &mut ROOK_TABLE[..]
    } else {
        &mut BISHOP_TABLE[..]
    };

    let mut b: Bitboard;
    let mut edges: Bitboard;

    let mut occs = [Bitboard::ZERO; 4096];
    let mut atts = [Bitboard::ZERO; 4096];

    let mut count = 0;
    let mut size = 0;
    let mut epoch = [0; 4096];

    for s in (0..64).map(|sq_idx| Square::new(sq_idx)) {
        edges =
            (Bitboard::from(Rank::One) | Bitboard::from(Rank::Eight)) & !Bitboard::from(s.rank());
        edges |= (Bitboard::from(File::A) | Bitboard::from(File::H)) & !Bitboard::from(s.file());

        let ptr = if s == Square::A1 {
            0
        } else {
            magics[s.to_usize() - 1].ptr
        };
        let m = &mut magics[s.to_usize()];
        m.mask = sliding_attack(s, IS_ROOK, Bitboard::ZERO) & !edges;
        m.shift = 64 - m.mask.popcount();
        m.ptr = ptr + size;

        size = 0;
        b = Bitboard::ZERO;

        while b.gtz() || size == 0 {
            occs[size] = b;
            atts[size] = sliding_attack(s, IS_ROOK, b);

            if cfg!(feature = "pext") {
                table[m.ptr + pext_u64(b.inner(), m.mask.inner()) as usize] = atts[size];
            }

            size += 1;
            b = (b - m.mask) & m.mask;
        }

        m.width = size;

        if cfg!(feature = "pext") {
            continue;
        }

        let mut prng = PRNG::new(SEEDS[s.rank().to_usize()]);

        let mut i = 0;
        while i < size {
            m.magic = Bitboard::ZERO;
            while ((m.magic * m.mask) >> 56).popcount() < 6 {
                m.magic = prng.get_sparse::<Bitboard>();
            }

            count += 1;
            i = 0;
            'thisone: while i < size {
                let idx = m.offset(occs[i]);

                if epoch[idx] < count {
                    epoch[idx] = count;
                    table[m.ptr + idx] = atts[i];
                } else if table[m.ptr + idx] != atts[i] {
                    break 'thisone;
                }

                i += 1;
            }
        }
    }

    for m in magics {
        m.attacks = &table[m.ptr..m.ptr + m.width];
    }
}

fn sliding_attack(square: Square, is_rook: bool, occupied_squares: Bitboard) -> Bitboard {
    let mut rv = Bitboard::ZERO;

    let shift_amounts: [i32; 4] = if is_rook {
        [1, -1, 8, -8]
    } else {
        [7, -7, 9, -9]
    };

    let fa = Bitboard::from(File::A);
    let fh = Bitboard::from(File::H);
    let r1 = Bitboard::from(Rank::One);
    let r8 = Bitboard::from(Rank::Eight);

    for shift in shift_amounts {
        let mut sb: Bitboard = square.into();
        let mut curshift = Bitboard::ZERO;
        while sb.gtz() & !(curshift & occupied_squares).gtz() {
            sb = match shift {
                8 => (sb << 8) & !r1,
                1 => (sb << 1) & !fa,
                -1 => (sb >> 1) & !fh,
                -8 => (sb >> 8) & !r8,
                9 => (sb << 9) & !(fa | r1),
                -7 => (sb >> 7) & !(fa | r8),
                -9 => (sb >> 9) & !(fh | r8),
                7 => (sb << 7) & !(fh | r1),
                _ => unreachable!(),
            };

            curshift |= sb;
        }
        rv |= curshift;
    }

    rv
}

pub(crate) fn magic_lookup<const IS_ROOK: bool>(square: Square, occupied: Bitboard) -> Bitboard {
    debug_assert!(square.is_ok());
    let magics = unsafe {
        if IS_ROOK {
            ROOK_MAGICS
        } else {
            BISHOP_MAGICS
        }
    };
    let m = magics[square.to_usize()];

    let i = m.offset(occupied);
    m.attacks[i]
}
