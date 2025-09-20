use std::collections::VecDeque;

use crate::utils::{coor::Coor, fast_clear_array::FastClearArray2d};

pub struct GridPathFinder {
    h: usize,
    w: usize,
    q: VecDeque<Coor>,
    dist: FastClearArray2d<i32>,
    prev: FastClearArray2d<Option<Coor>>,
}

impl GridPathFinder {
    pub fn new(h: usize, w: usize) -> Self {
        Self {
            h,
            w,
            q: VecDeque::new(),
            dist: FastClearArray2d::new(h, w, i32::MAX),
            prev: FastClearArray2d::new(h, w, None),
        }
    }

    pub fn find_valid_path<F, G>(
        &mut self,
        start: &Coor,
        dest_cond: F,
        path_cond: G,
    ) -> Option<Vec<Coor>>
    where
        F: Fn(&Coor) -> bool,
        G: Fn(&Coor, &Coor) -> bool,
    {
        self.dist.clear();
        self.q.clear();
        self.prev.clear();

        self.dist.set(&start, 0);
        self.q.push_back(*start);

        while let Some(v) = self.q.pop_front() {
            if dest_cond(&v) {
                return Some(self.restore_path(start, &v));
            }

            let new_dist = self.dist.get(&v) + 1;
            for u in v.adj_iter(self.h, self.w).filter(|u| path_cond(u, &v)) {
                if self.dist.get(&u) <= new_dist {
                    continue;
                }
                self.dist.set(&u, new_dist);
                self.q.push_back(u);
                self.prev.set(&u, Some(v));
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
}
