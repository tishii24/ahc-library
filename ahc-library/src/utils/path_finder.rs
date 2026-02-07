use std::collections::VecDeque;

use crate::utils::{fast_clear_array::FastClearArray2d, ndarray::Array2d, random::Random, v2::V2};

pub trait PathFindState {
    fn trans(&self, u: &V2<usize>, path: &[V2<usize>]) -> Self;
}

#[derive(Clone, Copy)]
pub struct DummyPathFindState;

impl PathFindState for DummyPathFindState {
    fn trans(&self, _: &V2<usize>, _: &[V2<usize>]) -> Self {
        *self
    }
}

pub struct BfsGridPathFinder<R: Random> {
    h: usize,
    w: usize,
    q: VecDeque<V2<usize>>,
    dist: FastClearArray2d<i32>,
    prev: FastClearArray2d<Option<V2<usize>>>,
    seen: Vec<V2<usize>>,
    rnd: R,
}

impl<R: Random> BfsGridPathFinder<R> {
    pub fn new(h: usize, w: usize, rnd: R) -> Self {
        Self {
            h,
            w,
            q: VecDeque::new(),
            dist: FastClearArray2d::new(h, w, i32::MAX),
            prev: FastClearArray2d::new(h, w, None),
            seen: Vec::with_capacity(h * w),
            rnd,
        }
    }

    pub fn get_reachable_coors<T, D>(
        &mut self,
        start: &V2<usize>,
        trans_cond: T,
        priority_d: D,
    ) -> Vec<V2<usize>>
    where
        T: Fn(&V2<usize>, &V2<usize>) -> bool,
        D: Fn(usize, &V2<usize>, &mut R) -> V2<usize>,
    {
        self._bfs(start, |_| false, trans_cond, priority_d);
        self.seen.clone()
    }

    pub fn get_reachable_size<T, D>(
        &mut self,
        start: &V2<usize>,
        trans_cond: T,
        priority_d: D,
    ) -> usize
    where
        T: Fn(&V2<usize>, &V2<usize>) -> bool,
        D: Fn(usize, &V2<usize>, &mut R) -> V2<usize>,
    {
        self._bfs(start, |_| false, trans_cond, priority_d);
        self.seen.len()
    }

    /// 両端点を含む
    pub fn find_path<C, T, D>(
        &mut self,
        start: &V2<usize>,
        complete_cond: C,
        trans_cond: T,
        priority_d: D,
    ) -> Option<Vec<V2<usize>>>
    where
        C: Fn(&V2<usize>) -> bool,
        T: Fn(&V2<usize>, &V2<usize>) -> bool,
        D: Fn(usize, &V2<usize>, &mut R) -> V2<usize>,
    {
        let v = self._bfs(start, complete_cond, trans_cond, priority_d)?;
        Some(self.restore_path(start, &v))
    }

    fn _bfs<C, T, D>(
        &mut self,
        start: &V2<usize>,
        complete_cond: C,
        trans_cond: T,
        priority_d: D,
    ) -> Option<V2<usize>>
    where
        C: Fn(&V2<usize>) -> bool,
        T: Fn(&V2<usize>, &V2<usize>) -> bool,
        D: Fn(usize, &V2<usize>, &mut R) -> V2<usize>,
    {
        self.reset();

        self.dist.set(&(start.x, start.y), 0);
        self.q.push_back(*start);
        self.seen.push(*start);

        while let Some(v) = self.q.pop_front() {
            if complete_cond(&v) {
                return Some(v);
            }

            let new_dist = self.dist.get(&(v.x, v.y)) + 1;
            for i in 0..4 {
                let d = priority_d(i, &v, &mut self.rnd);
                let u = v + d;
                if u.x < self.h
                    && u.y < self.w
                    && (trans_cond)(&u, &v)
                    && new_dist < self.dist.get(&(u.x, u.y))
                {
                    self.q.push_back(u);

                    self.dist.set(&(u.x, u.y), new_dist);
                    self.prev.set(&(u.x, u.y), Some(v));
                    self.seen.push(u);
                }
            }
        }

        None
    }

    pub fn restore_path(&mut self, start: &V2<usize>, end: &V2<usize>) -> Vec<V2<usize>> {
        let mut path = vec![*end];
        let mut cur = *end;
        while let Some(p) = self.prev.get(&(cur.x, cur.y)) {
            cur = p;
            path.push(cur);
        }
        path.reverse();

        assert_eq!(&cur, start);

        path
    }

    pub fn get_dist_table(&mut self) -> Array2d<usize> {
        let mut array2d = Array2d::init(self.h, self.w, 0);
        for i in 0..self.h {
            for j in 0..self.w {
                array2d[(i, j)] = self.dist.get(&(i, j)) as usize;
            }
        }
        array2d
    }

    fn reset(&mut self) {
        self.dist.clear();
        self.q.clear();
        self.prev.clear();
        self.seen.clear();
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::{path_finder::BfsGridPathFinder, random::XorShift32, v2::*};

    #[test]
    fn test_bfs() {
        let (h, w) = (5, 8);
        let mut path_finder = BfsGridPathFinder::new(h, w, XorShift32::new(24));
        let size = path_finder.get_reachable_size(&V2::new(0, 0), |_, _| true, |i, _, _| D4[i]);
        assert_eq!(size, h * w);
    }

    #[test]
    fn test_bfs_with_wall() {
        let n = 5;
        let mut path_finder = BfsGridPathFinder::new(n, n, XorShift32::new(24));
        let size = path_finder.get_reachable_size(&V2::new(0, 0), |u, _| u.y != 2, |i, _, _| D4[i]);
        assert_eq!(size, n * 2);
    }

    #[test]
    fn test_find_path() {
        let n = 5;
        let mut path_finder = BfsGridPathFinder::new(n, n, XorShift32::new(24));
        let d = [D_DOWN, D_UP, D_RIGHT, D_LEFT];
        let t = V2::new(n - 1, n - 1);
        let path = path_finder.find_path(&V2::new(0, 0), |c| c == &t, |_, _| true, |i, _, _| d[i]);
        assert_eq!(
            path.unwrap(),
            vec![
                V2::new(0, 0),
                V2::new(1, 0),
                V2::new(2, 0),
                V2::new(3, 0),
                V2::new(4, 0),
                V2::new(4, 1),
                V2::new(4, 2),
                V2::new(4, 3),
                V2::new(4, 4),
            ]
        );
    }
}
