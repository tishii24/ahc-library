use std::collections::VecDeque;

use crate::utils::{coor::Coor, fast_clear_array::FastClearArray2d, random::Rnd};

pub trait PathFindState {
    fn trans(&self, u: &Coor, path: &Vec<Coor>) -> Self;
}

#[derive(Clone, Copy)]
pub struct DummyPathFindState;

impl PathFindState for DummyPathFindState {
    fn trans(&self, _: &Coor, _: &Vec<Coor>) -> Self {
        *self
    }
}

pub struct BfsGridPathFinder {
    h: usize,
    w: usize,
    q: VecDeque<Coor>,
    dist: FastClearArray2d<i32>,
    prev: FastClearArray2d<Option<Coor>>,
    seen: Vec<Coor>,
    rnd: Rnd,
}

impl BfsGridPathFinder {
    pub fn new(h: usize, w: usize, seed: u32) -> Self {
        Self {
            h,
            w,
            q: VecDeque::new(),
            dist: FastClearArray2d::new(h, w, i32::MAX),
            prev: FastClearArray2d::new(h, w, None),
            seen: Vec::with_capacity(h * w),
            rnd: Rnd::new(seed),
        }
    }

    pub fn get_reachable_coors<T, D>(
        &mut self,
        start: &Coor,
        trans_cond: T,
        priority_d: D,
    ) -> Vec<Coor>
    where
        T: Fn(&Coor, &Coor) -> bool,
        D: Fn(usize, &Coor, &mut Rnd) -> Coor,
    {
        self._bfs(start, |_| false, trans_cond, priority_d);
        self.seen.clone()
    }

    pub fn get_reachable_size<T, D>(&mut self, start: &Coor, trans_cond: T, priority_d: D) -> usize
    where
        T: Fn(&Coor, &Coor) -> bool,
        D: Fn(usize, &Coor, &mut Rnd) -> Coor,
    {
        self._bfs(start, |_| false, trans_cond, priority_d);
        self.seen.len()
    }

    /// 両端点を含む
    pub fn find_path<C, T, D>(
        &mut self,
        start: &Coor,
        complete_cond: C,
        trans_cond: T,
        priority_d: D,
    ) -> Option<Vec<Coor>>
    where
        C: Fn(&Coor) -> bool,
        T: Fn(&Coor, &Coor) -> bool,
        D: Fn(usize, &Coor, &mut Rnd) -> Coor,
    {
        let v = self._bfs(start, complete_cond, trans_cond, priority_d)?;
        Some(self.restore_path(start, &v))
    }

    fn _bfs<C, T, D>(
        &mut self,
        start: &Coor,
        complete_cond: C,
        trans_cond: T,
        priority_d: D,
    ) -> Option<Coor>
    where
        C: Fn(&Coor) -> bool,
        T: Fn(&Coor, &Coor) -> bool,
        D: Fn(usize, &Coor, &mut Rnd) -> Coor,
    {
        self.reset();

        self.dist.set(&start, 0);
        self.q.push_back(*start);
        self.seen.push(*start);

        while let Some(v) = self.q.pop_front() {
            if complete_cond(&v) {
                return Some(v);
            }

            let new_dist = self.dist.get(&v) + 1;
            for i in 0..4 {
                let d = priority_d(i, &v, &mut self.rnd);
                let u = v.add(&d);
                if u.i < self.h
                    && u.j < self.w
                    && (trans_cond)(&u, &v)
                    && new_dist < self.dist.get(&u)
                {
                    self.q.push_back(u);

                    self.dist.set(&u, new_dist);
                    self.prev.set(&u, Some(v));
                    self.seen.push(u);
                }
            }
        }

        None
    }

    pub fn restore_path(&mut self, start: &Coor, end: &Coor) -> Vec<Coor> {
        let mut path = vec![*end];
        let mut cur = *end;
        while let Some(p) = self.prev.get(&cur) {
            cur = p;
            path.push(cur);
        }
        path.reverse();

        assert_eq!(&cur, start);

        path
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
    use crate::utils::{coor::*, path_finder::BfsGridPathFinder};

    #[test]
    fn test_bfs() {
        let (h, w) = (5, 8);
        let mut path_finder = BfsGridPathFinder::new(h, w, 0);
        let size = path_finder.get_reachable_size(&Coor::new(0, 0), |_, _| true, |i, _, _| D4[i]);
        assert_eq!(size, h * w);
    }

    #[test]
    fn test_bfs_with_wall() {
        let n = 5;
        let mut path_finder = BfsGridPathFinder::new(n, n, 0);
        let size =
            path_finder.get_reachable_size(&Coor::new(0, 0), |u, _| u.j != 2, |i, _, _| D4[i]);
        assert_eq!(size, n * 2);
    }

    #[test]
    fn test_find_path() {
        let n = 5;
        let mut path_finder = BfsGridPathFinder::new(n, n, 0);
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
