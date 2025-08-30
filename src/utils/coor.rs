#![allow(overflowing_literals)]

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Coor {
    pub i: usize,
    pub j: usize,
}

impl Coor {
    pub fn new(i: usize, j: usize) -> Coor {
        Coor { i, j }
    }

    #[inline]
    pub fn to_index(&self, w: usize) -> usize {
        self.i * w + self.j
    }

    #[inline]
    pub fn from_index(index: usize, w: usize) -> Coor {
        Coor {
            i: index / w,
            j: index % w,
        }
    }

    #[inline]
    pub fn add(&self, d: Coor) -> Coor {
        Coor {
            i: self.i + d.i,
            j: self.j + d.j,
        }
    }

    #[inline]
    pub fn sub(&self, d: Coor) -> Coor {
        Coor {
            i: self.i - d.i,
            j: self.j - d.j,
        }
    }

    pub fn adj_iter(&self, h: usize, w: usize) -> impl Iterator<Item = Coor> {
        let directions = [
            Coor::new(1, 0),
            Coor::new(0, 1),
            Coor::new(!0, 0),
            Coor::new(0, !0),
        ];
        directions
            .into_iter()
            .map(|d| self.add(d))
            .filter(move |c| c.i < h && c.j < w)
    }

    pub fn shuffled_adj_iter(
        &self,
        h: usize,
        w: usize,
        rnd: &mut crate::utils::rnd::Rnd,
    ) -> impl Iterator<Item = Coor> {
        let mut directions = [
            Coor::new(1, 0),
            Coor::new(0, 1),
            Coor::new(!0, 0),
            Coor::new(0, !0),
        ];
        rnd.shuffle(&mut directions);

        directions
            .into_iter()
            .map(|d| self.add(d))
            .filter(move |c| c.i < h && c.j < w)
    }

    pub fn manhattan_distance(a: Coor, b: Coor) -> usize {
        a.i.abs_diff(b.i) + a.j.abs_diff(b.j)
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::coor::Coor;

    #[test]
    fn test_adj_iter() {
        let coor = Coor::new(1, 1);
        let adj: Vec<_> = coor.adj_iter(3, 3).collect();
        assert_eq!(adj.len(), 4);
        assert!(adj.contains(&Coor::new(2, 1)));
        assert!(adj.contains(&Coor::new(1, 2)));
        assert!(adj.contains(&Coor::new(0, 1)));
        assert!(adj.contains(&Coor::new(1, 0)));
    }

    #[test]
    fn test_adj_iter_edge() {
        let coor = Coor::new(0, 0);
        let adj: Vec<_> = coor.adj_iter(3, 3).collect();
        assert_eq!(adj.len(), 2);
        assert!(adj.contains(&Coor::new(1, 0)));
        assert!(adj.contains(&Coor::new(0, 1)));
    }
}
