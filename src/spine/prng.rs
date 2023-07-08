use std::ops::BitAnd;

#[derive(Debug)]
pub(crate) struct PRNG(u64);

impl PRNG {
    pub(crate) fn new(value: u64) -> Self {
        debug_assert!(value > 0);
        Self(value)
    }

    pub(crate) fn get<T>(&mut self) -> T
        where T: From<u64>
    {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        let v = self.0.wrapping_mul(268582165773638717);

        T::from(v)
    }

    pub(crate) fn get_sparse<T>(&mut self) -> T
        where T: From<u64>
    {
        T::from(self.get::<u64>()
            & self.get::<u64>()
            & self.get::<u64>())
    }
}
