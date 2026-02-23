use std::collections::VecDeque;

use crate::utils::{
    fast_clear_array::FastClearArray2d, ndarray::Array2d, random::Random, v2::Coor,
};

pub trait PathFindState {
    fn trans(&self, u: &Coor<usize>, path: &[Coor<usize>]) -> Self;
}

#[derive(Clone, Copy)]
pub struct DummyPathFindState;

impl PathFindState for DummyPathFindState {
    fn trans(&self, _: &Coor<usize>, _: &[Coor<usize>]) -> Self {
        *self
    }
}

pub struct BfsGridPathFinder<R: Random> {
    h: usize,
    w: usize,
    q: VecDeque<Coor<usize>>,
    dist: FastClearArray2d<i32>,
    prev: FastClearArray2d<Option<Coor<usize>>>,
    seen: Vec<Coor<usize>>,
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
        start: &Coor<usize>,
        trans_cond: T,
        priority_d: D,
    ) -> Vec<Coor<usize>>
    where
        T: Fn(&Coor<usize>, &Coor<usize>) -> bool,
        D: Fn(usize, &Coor<usize>, &mut R) -> Coor<usize>,
    {
        self._bfs(start, |_| false, trans_cond, priority_d);
        self.seen.clone()
    }

    pub fn get_reachable_size<T, D>(
        &mut self,
        start: &Coor<usize>,
        trans_cond: T,
        priority_d: D,
    ) -> usize
    where
        T: Fn(&Coor<usize>, &Coor<usize>) -> bool,
        D: Fn(usize, &Coor<usize>, &mut R) -> Coor<usize>,
    {
        self._bfs(start, |_| false, trans_cond, priority_d);
        self.seen.len()
    }

    /// 両端点を含む
    pub fn find_path<C, T, D>(
        &mut self,
        start: &Coor<usize>,
        complete_cond: C,
        trans_cond: T,
        priority_d: D,
    ) -> Option<Vec<Coor<usize>>>
    where
        C: Fn(&Coor<usize>) -> bool,
        T: Fn(&Coor<usize>, &Coor<usize>) -> bool,
        D: Fn(usize, &Coor<usize>, &mut R) -> Coor<usize>,
    {
        let v = self._bfs(start, complete_cond, trans_cond, priority_d)?;
        Some(self.restore_path(start, &v))
    }

    fn _bfs<C, T, D>(
        &mut self,
        start: &Coor<usize>,
        complete_cond: C,
        trans_cond: T,
        priority_d: D,
    ) -> Option<Coor<usize>>
    where
        C: Fn(&Coor<usize>) -> bool,
        T: Fn(&Coor<usize>, &Coor<usize>) -> bool,
        D: Fn(usize, &Coor<usize>, &mut R) -> Coor<usize>,
    {
        self.reset();

        self.dist.set(&(start.i, start.j), 0);
        self.q.push_back(*start);
        self.seen.push(*start);

        while let Some(v) = self.q.pop_front() {
            if complete_cond(&v) {
                return Some(v);
            }

            let new_dist = self.dist.get(&(v.i, v.j)) + 1;
            for i in 0..4 {
                let d = priority_d(i, &v, &mut self.rnd);
                let u = v + d;
                if u.i < self.h
                    && u.j < self.w
                    && (trans_cond)(&u, &v)
                    && new_dist < self.dist.get(&(u.i, u.j))
                {
                    self.q.push_back(u);

                    self.dist.set(&(u.i, u.j), new_dist);
                    self.prev.set(&(u.i, u.j), Some(v));
                    self.seen.push(u);
                }
            }
        }

        None
    }

    pub fn restore_path(&mut self, start: &Coor<usize>, end: &Coor<usize>) -> Vec<Coor<usize>> {
        let mut path = vec![*end];
        let mut cur = *end;
        while let Some(p) = self.prev.get(&(cur.i, cur.j)) {
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
    use super::*;
    use crate::utils::{
        random::XorShift32,
        v2::{Coor, D4, D_DOWN, D_LEFT, D_RIGHT, D_UP},
    };

    #[test]
    fn test_bfs() {
        let (h, w) = (5, 8);
        let mut path_finder = BfsGridPathFinder::new(h, w, XorShift32::new(24));
        let size = path_finder.get_reachable_size(&Coor::new(0, 0), |_, _| true, |i, _, _| D4[i]);
        assert_eq!(size, h * w);
    }

    #[test]
    fn test_bfs_with_wall() {
        let n = 5;
        let mut path_finder = BfsGridPathFinder::new(n, n, XorShift32::new(24));
        let size =
            path_finder.get_reachable_size(&Coor::new(0, 0), |u, _| u.j != 2, |i, _, _| D4[i]);
        assert_eq!(size, n * 2);
    }

    #[test]
    fn test_find_path() {
        let n = 5;
        let mut path_finder = BfsGridPathFinder::new(n, n, XorShift32::new(24));
        let d = [D_DOWN, D_UP, D_RIGHT, D_LEFT];
        let t = Coor::new(n - 1, n - 1);
        let path =
            path_finder.find_path(&Coor::new(0, 0), |c| c == &t, |_, _| true, |i, _, _| d[i]);
        assert_eq!(
            path.unwrap(),
            vec![
                Coor::new(0, 0),
                Coor::new(1, 0),
                Coor::new(2, 0),
                Coor::new(3, 0),
                Coor::new(4, 0),
                Coor::new(4, 1),
                Coor::new(4, 2),
                Coor::new(4, 3),
                Coor::new(4, 4),
            ]
        );
    }
}
