use super::{Bitboard, Square};
use super::{File, Rank};

use crate::macros::pext;

#[derive(Debug, Clone, Copy)]
struct Magic<'a> {
    mask: Bitboard,
    multiplier: u64,
    shift: i32,
    attacks: &'a [Bitboard],
    index: usize
}

struct RandU64(u64);

impl<'a> Magic<'a> {
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            mask: Bitboard::new(0),
            multiplier: 0,
            shift: 0,
            attacks: &[],
            index: 0
        }
    }

    #[inline]
    pub const fn offset(&self, occupancy: Bitboard) -> usize {
        let f1 = self.multiplier;
        let f2 = occupancy.as_u64();
        let f3 = self.mask.as_u64();

        #[cfg(feature = "pext")]
        return pext!(f2, f3);

        ((f2 & f3).wrapping_mul(f1) >> self.shift) as usize
    }
}

impl RandU64 {
    #[inline(always)]
    pub const fn new(val: u64) -> Self {
        debug_assert!(val > 0);
        Self(val)
    }

    pub fn reroll(&mut self) {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0 = self.0.wrapping_mul(2685821657736338717_u64);
    }

    pub fn fetch_new(&mut self) -> u64 {
        self.reroll();
        self.0
    }
    pub fn fetch_new_sparse(&mut self) -> u64 {
        let a = self.fetch_new();
        let b = self.fetch_new();
        let c = self.fetch_new();
        a & b & c
    }
}

static mut BISHOP_MAGICS: [Magic; 64] = [Magic::new(); 64];
static mut ROOK_MAGICS: [Magic; 64] = [Magic::new(); 64];

static mut BISHOP_ATTACK_TABLE: [Bitboard; 0x1480] = [Bitboard::new(0); 0x1480];
static mut ROOK_ATTACK_TABLE: [Bitboard; 0x19000] = [Bitboard::new(0); 0x19000];

const SEEDS: [u64; 8] = [728, 10316, 55013, 32803,
                         12281, 15100, 16645, 255];

pub(crate) fn initialize_magics() {
    unsafe {
        init_magics(false, &mut BISHOP_ATTACK_TABLE[..], &mut BISHOP_MAGICS[..]);
        init_magics(true, &mut ROOK_ATTACK_TABLE[..], &mut ROOK_MAGICS[..]);
    }
}

fn init_magics<'a>(is_rook: bool, table: &'a mut [Bitboard], magics: &'a mut [Magic<'a>]) {
    let mut occupancy = [Bitboard::new(0); 4096];
    let mut reference = [Bitboard::new(0); 4096];
    let mut edges: Bitboard;
    let mut b = Bitboard::new(0);
    let mut epoch = [0; 4096];
    let mut count = 0;
    let mut size = 0;

    for s in (0..64).map(|x| Square::new(x)) {
        let e_rank = (Rank::One.to_bitboard() | Rank::Eight.to_bitboard()) &! s.rank().to_bitboard();
        let e_file = (File::A.to_bitboard() | File::H.to_bitboard()) &! s.file().to_bitboard();
        edges = e_rank | e_file;

        // do this first to avoid reference sharing
        let new_index = if s.as_u8() == 0 { 0 } else { magics[s.as_u8() as usize - 1].index + size };

        let m = &mut magics[s.as_u8() as usize];
        m.mask = sliding_attack(s, is_rook, Bitboard::new(0)) & !edges;
        m.shift = 64 - m.mask.popcount() as i32;
        m.index = new_index;

        b ^= b;
        size = 0;

        loop {
            occupancy[size] = b;
            reference[size] = sliding_attack(s, is_rook, b);

            if cfg!(feature = "pext") {
                table[m.index + pext!(b.as_u64(), m.mask.as_u64())] = reference[size];
            }

            size += 1;
            b = b.carry_ripple(m.mask);

            if !b.gtz() { break; }
        }

        #[cfg(feature = "pext")]
        continue;

        let mut prng = RandU64::new(SEEDS[s.rank().as_usize()]);
        let mut i = 0;
        while i < size {
            m.multiplier = 0;
            while (m.multiplier.wrapping_mul(m.mask.as_u64()) >> 56).count_ones() < 6 {
                m.multiplier = prng.fetch_new_sparse();
            }

            count += 1;
            i = 0;
            while i < size {
                let idx = m.offset(occupancy[i]);

                if epoch[idx] < count {
                    epoch[idx] = count;
                    table[m.index + idx] = reference[i];
                } else if table[m.index + idx] != reference[i] {
                    break;
                }

                i += 1;
            }
        }
    }

    for m in magics {
        m.attacks = &table[m.index..];
    }
}


pub(crate) fn sliding_attack(square: Square, is_rook: bool, occupied: Bitboard) -> Bitboard {
    let mut rv = Bitboard::new(0);

    let shifts = if is_rook {
        [8, -8, 1, -1]
    } else {
        [7, 9, -7, -9]
    };

    for shift in shifts {
        let mut s = square;
        while let Some(o) = s.offset(shift) {
            if o.distance(s) <= 2 && !(occupied & s.into()).gtz() {
                rv |= o.into();
                s = o;
            } else { break; }
        }
    }

    rv
}

pub(crate) fn get_sliding_attack(is_rook: bool, square: Square, occupancy: Bitboard) -> Bitboard {
    debug_assert!(square.is_ok());
    let magic = unsafe { if is_rook {
        ROOK_MAGICS[square.as_u8() as usize]
    } else {
        BISHOP_MAGICS[square.as_u8() as usize]
    }};

    magic.attacks[magic.offset(occupancy)] 
}

