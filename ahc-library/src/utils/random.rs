#[derive(Debug, Clone, Copy)]
pub struct Rnd {
    state: u32,
}

impl Rnd {
    pub fn new(mut seed: u32) -> Self {
        if seed == 0 {
            seed = u32::MAX;
        }
        Self { state: seed }
    }

    #[inline(always)]
    pub fn next(&mut self) -> u32 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.state = x;
        x
    }

    #[inline(always)]
    pub fn nextf(&mut self) -> f64 {
        self.next() as f64 / ((1u64 << 32) as f64)
    }

    #[inline(always)]
    pub fn gen_index(&mut self, len: usize) -> usize {
        debug_assert!(len as u64 <= 1 << 32);
        ((len as u64 * self.next() as u64) >> 32) as usize
    }

    #[inline(always)]
    pub fn gen_range(&mut self, l: usize, r: usize) -> usize {
        debug_assert!(l < r);
        debug_assert!(r as u64 <= 1 << 32);
        l + (((r - l) as u64 * self.next() as u64) >> 32) as usize
    }

    #[inline(always)]
    pub fn gen_rangef(&mut self, l: f64, r: f64) -> f64 {
        debug_assert!(l <= r);
        l + self.next() as f64 * ((r - l) / ((1u64 << 32) as f64))
    }

    #[inline(always)]
    pub fn shuffle<T>(&mut self, v: &mut [T]) {
        let n = v.len();
        for i in (1..n).rev() {
            let j = self.gen_range(0, i + 1);
            v.swap(i, j);
        }
    }
}
