use super::{Bitboard, Square};
use super::{File, Rank};

use crate::macros::pext;

#[derive(Clone, Copy)]
struct Magic {
    mask: Bitboard,
    multiplier: u64,
    shift: i32,
    attack_index: usize,
}

struct RandU64(u64);

impl Magic {
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            mask: Bitboard::new(0),
            multiplier: 0,
            shift: 0,
            attack_index: 0
        }
    }
    #[inline]
    pub fn offset(&self, occupancy: Bitboard) -> usize {
        // TODO: Pext? -> attack_index + PEXT(occupancy, self.mask)
        (((self.mask & occupancy).as_u64() * self.multiplier) >> self.shift) as usize
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

const SEEDS: [u64; 8] = [8977, 44560, 54343, 38998, 5731, 95205, 104912, 17020];

pub(crate) fn initialize_magics() {
    unsafe {
        init_magics(false);
        init_magics(true);
    }
}

unsafe fn init_magics(is_rook: bool) {
    let mut tbl: &mut [Bitboard] = if is_rook { &mut ROOK_ATTACK_TABLE[..] } else { &mut BISHOP_ATTACK_TABLE[..] };
    let mut magics = if is_rook { ROOK_MAGICS } else { BISHOP_MAGICS };
    let mut occ: [Bitboard; 4096] = [Bitboard::new(0); 4096];
    let mut rfr: [Bitboard; 4096] = [Bitboard::new(0); 4096];
    let mut edges: Bitboard = Bitboard::new(0);
    let mut b: Bitboard = Bitboard::new(0);
    let mut epoch: [i32; 4096] = [0; 4096];
    let mut count: i32 = 0;
    let mut size: usize = 0;

    for i in 0..64 {
        println!("Looping for square: #{i}");
        let s = Square::new(i);
        edges = ((Rank::One.to_bitboard() | Rank::Eight.to_bitboard()) & !s.rank().to_bitboard())
            | ((File::A.to_bitboard() | File::H.to_bitboard()) &! s.file().to_bitboard());

        let last_att_index = if s.as_u8() == 0 { 0 } else { magics[s.as_u8() as usize - 1].attack_index + size };
        let mut m = &mut magics[s.as_u8() as usize];
        m.mask = sliding_attack(s, is_rook, Bitboard::new(0)) & !edges;
        m.shift = 64 - m.mask.popcount() as i32;
        m.attack_index = last_att_index;

        b ^= b;
        size = 0;
        loop {
            occ[size] = b;
            rfr[size] = sliding_attack(s, is_rook, b);

            if cfg!(feature = "pext") {
                tbl[m.attack_index + pext!(b, m.mask)] = rfr[size];
            }

            size += 1;
            b = b.carry_ripple(m.mask);

            if !b.gtz() {
                break;
            }
        } 

        #[cfg(feature = "pext")]
        continue;

        let mut rng = RandU64::new(SEEDS[s.rank().as_usize()]);

        let mut i: usize = 0;
        loop {
            m.multiplier = 0;
            'innermost: loop {
                if (m.mask.as_u64() * m.multiplier).count_ones() < 6 {
                    break 'innermost;
                }
                m.multiplier = rng.fetch_new_sparse();
            }

            count += 1;
            i = 0;
            loop  {
                let offset = m.offset(occ[i]);
                if epoch[offset] < count {
                    epoch[offset] = count;
                    tbl[m.attack_index + offset] = rfr[i];
                } else if tbl[m.attack_index + offset] != rfr[i] {
                    break;
                }

                i += 1;
                if i >= size {
                    break;
                }
            }

            if i >= size {
                break;
            }
        } 
    }
}

fn sliding_attack(square: Square, is_rook: bool, occupied: Bitboard) -> Bitboard {
    let mut rv = Bitboard::new(0);

    let shifts = if is_rook {
        [8, -8, 1, -1]
    } else {
        [7, 9, -7, -9]
    };

    for shift in shifts {
        let mut s = square;
        'inner: loop {
            if let Some(off) = s.offset(shift) {
                if s.distance(off) <= 2 && !(occupied & off.into()).gtz() {
                    s = off;
                    rv |= s.into();
                } else { break 'inner; }
            } else { break 'inner; }
        }
    }

    rv
}

pub(crate) fn get_magic_value(is_rook: bool, square: Square, occupancy: Bitboard) -> Bitboard {
    debug_assert!(square.is_ok());
    let magic = unsafe { if is_rook {
        ROOK_MAGICS[square.as_u8() as usize]
    } else {
        BISHOP_MAGICS[square.as_u8() as usize]
    }};
    let tbl = unsafe { if is_rook {
        &ROOK_ATTACK_TABLE[..]
    } else {
        &BISHOP_ATTACK_TABLE[..]
    }};

    tbl[magic.attack_index + magic.offset(occupancy)]
}

