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
    rnd: Rnd,
}

impl BfsGridPathFinder {
    pub fn new(h: usize, w: usize) -> Self {
        Self {
            h,
            w,
            q: VecDeque::new(),
            dist: FastClearArray2d::new(h, w, i32::MAX),
            prev: FastClearArray2d::new(h, w, None),
            rnd: Rnd::new(24),
        }
    }

    pub fn get_reachable_coors<T>(&mut self, start: &Coor, trans_cond: T) -> Vec<Coor>
    where
        T: Fn(&Coor, &Coor) -> bool,
    {
        self.reset();

        self.dist.set(&start, 0);
        self.q.push_back(*start);

        let mut coors = vec![*start];

        while let Some(v) = self.q.pop_front() {
            let new_dist = self.dist.get(&v) + 1;
            for u in v.adj_iter(self.h, self.w).filter(|u| trans_cond(u, &v)) {
                if self.dist.get(&u) <= new_dist {
                    continue;
                }
                self.dist.set(&u, new_dist);
                self.q.push_back(u);
                self.prev.set(&u, Some(v));

                coors.push(u);
            }
        }

        coors
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
        self.reset();

        self.dist.set(&start, 0);
        self.q.push_back(*start);

        while let Some(v) = self.q.pop_front() {
            if complete_cond(&v) {
                return Some(self.restore_path(start, &v));
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
                    self.dist.set(&u, new_dist);
                    self.q.push_back(u);
                    self.prev.set(&u, Some(v));
                }
            }
        }

        None
    }

    fn restore_path(&mut self, start: &Coor, end: &Coor) -> Vec<Coor> {
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
    }
}
