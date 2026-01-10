use rand_pcg::rand_core::{RngCore, SeedableRng};

pub trait Random {
    fn new(seed: u32) -> Self;
    fn _next(&mut self) -> u32;

    #[inline(always)]
    fn nextf(&mut self) -> f64 {
        self._next() as f64 / ((1u64 << 32) as f64)
    }

    #[inline(always)]
    fn choice<T: Clone + Copy>(&mut self, v: &[T]) -> T {
        let idx = self.gen_index(v.len());
        v[idx]
    }

    #[inline(always)]
    fn gen_index(&mut self, len: usize) -> usize {
        debug_assert!(len as u64 <= 1 << 32);
        ((len as u64 * self._next() as u64) >> 32) as usize
    }

    #[inline(always)]
    fn gen_range(&mut self, l: usize, r: usize) -> usize {
        debug_assert!(l < r);
        debug_assert!(r as u64 <= 1 << 32);
        l + (((r - l) as u64 * self._next() as u64) >> 32) as usize
    }

    #[inline(always)]
    fn gen_rangef(&mut self, l: f64, r: f64) -> f64 {
        debug_assert!(l <= r);
        l + self._next() as f64 * ((r - l) / ((1u64 << 32) as f64))
    }

    #[inline(always)]
    fn shuffle<T>(&mut self, v: &mut [T]) {
        let n = v.len();
        for i in (1..n).rev() {
            let j = self.gen_range(0, i + 1);
            v.swap(i, j);
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct XorShift32 {
    state: u32,
}

impl Random for XorShift32 {
    fn new(mut seed: u32) -> Self {
        if seed == 0 {
            seed = u32::MAX;
        }
        Self { state: seed }
    }

    #[inline(always)]
    fn _next(&mut self) -> u32 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.state = x;
        x
    }
}

#[derive(Debug, Clone)]
pub struct RandPcg64Mcg {
    inner: rand_pcg::Pcg64Mcg,
}

impl Random for RandPcg64Mcg {
    fn new(seed: u32) -> Self {
        Self {
            inner: rand_pcg::Pcg64Mcg::seed_from_u64(seed as u64),
        }
    }

    #[inline(always)]
    fn _next(&mut self) -> u32 {
        self.inner.next_u32()
    }
}

pub trait RandomSampler<T> {
    fn sample(&mut self) -> T;
}

pub struct DiscreteSampler<T, R> {
    buf: Vec<T>,
    rnd: R,
}

impl<T: Copy, R: Random> DiscreteSampler<T, R> {
    pub fn new(weight_values: &Vec<(usize, T)>) -> Self {
        let weight_sum = weight_values.iter().map(|(w, _)| *w).sum::<usize>();
        assert!(0 < weight_sum);
        assert!(weight_sum < 1_000_000);
        let mut buf = Vec::with_capacity(weight_sum);
        for &(w, val) in weight_values.iter() {
            buf.extend(std::iter::repeat(val).take(w));
        }
        Self {
            buf,
            rnd: R::new(24),
        }
    }
}

impl<T: Copy, R: Random> RandomSampler<T> for DiscreteSampler<T, R> {
    fn sample(&mut self) -> T {
        self.rnd.choice(&self.buf)
    }
}

pub struct ContinousSampler<R: Random> {
    buf: Vec<f64>,
    rnd: R,
}

impl<R: Random> ContinousSampler<R> {
    pub fn new<F>(f: F, x_min: f64, x_max: f64, size: usize) -> Self
    where
        F: Fn(f64) -> f64,
    {
        assert!(0 < size);
        assert!(size < 1_000_000);
        let mut buf = Vec::with_capacity(size);
        let step = (x_max - x_min) / (size as f64 - 1.);
        for i in 0..size {
            let x = x_min + step * (i as f64);
            buf.push(f(x));
        }
        Self {
            buf,
            rnd: R::new(24),
        }
    }
}

impl<R: Random> RandomSampler<f64> for ContinousSampler<R> {
    fn sample(&mut self) -> f64 {
        self.rnd.choice(&self.buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dicrete_sampler() {
        let weight_values = vec![(1, 'a'), (3, 'b'), (6, 'c')];
        let mut sampler = DiscreteSampler::<_, XorShift32>::new(&weight_values);
        let mut counts = std::collections::HashMap::new();
        for _ in 0..10000 {
            let v = sampler.sample();
            *counts.entry(v).or_insert(0) += 1;
        }
        assert!(counts[&'a'] < counts[&'b']);
        assert!(counts[&'b'] < counts[&'c']);
    }

    #[test]
    fn test_continous_sampler() {
        let f = |x: f64| x * x;
        let mut sampler = ContinousSampler::<XorShift32>::new(f, 0., 1., 100);
        let mut sum = 0.;
        for _ in 0..10000 {
            let v = sampler.sample();
            sum += v;
        }
        let mean = sum / 10000.;
        assert!((mean - 0.333).abs() < 0.01);
    }

    #[test]
    fn test_rand_pcg64_mcg() {
        let mut rnd = RandPcg64Mcg::new(42);
        let mut cnt = vec![0; 100];
        for _ in 0..10000 {
            cnt[rnd.gen_index(100)] += 1;
        }
        assert!(cnt.iter().all(|&c| 50 < c && c < 150));
    }
}
