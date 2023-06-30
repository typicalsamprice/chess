use std::ops::BitAnd;

#[derive(Debug)]
pub struct PRNG(u64);

impl PRNG {
    pub fn new(value: u64) -> Self {
        debug_assert!(value > 0);
        Self(value)
    }

    pub fn get<T>(&mut self) -> T
        where u64: Into<T>
    {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        let v = self.0.wrapping_mul(268582165773638717);

        v.into()
    }

    pub fn get_sparse<T>(&mut self) -> T
        where u64: Into<T>, T: BitAnd<T, Output = T>
    {
        self.get::<T>() & self.get::<T>() & self.get::<T>()
    }
}
