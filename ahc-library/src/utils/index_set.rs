use crate::utils::random::Random;

const NONE_POS: usize = !0;

#[derive(Debug, Clone)]
pub struct IndexSet {
    pub que: Vec<usize>,
    pub pos: Vec<usize>,
}

impl IndexSet {
    pub fn empty(n: usize) -> Self {
        IndexSet {
            que: Vec::with_capacity(n),
            pos: vec![NONE_POS; n],
        }
    }

    pub fn full(n: usize) -> Self {
        IndexSet {
            que: (0..n).collect(),
            pos: (0..n).collect(),
        }
    }

    pub fn clear(&mut self) {
        for &v in &self.que {
            self.pos[v] = NONE_POS;
        }
        self.que.clear();
    }

    pub fn add(&mut self, v: usize) {
        if self.contains(v) {
            return;
        }
        self.pos[v] = self.que.len();
        self.que.push(v);
    }

    pub fn remove(&mut self, v: usize) {
        if !self.contains(v) {
            return;
        }

        let p = self.pos[v];
        let b = self.que[self.que.len() - 1];
        self.que.swap_remove(p);
        self.pos[b] = p;
        self.pos[v] = NONE_POS;
    }

    pub fn contains(&self, v: usize) -> bool {
        self.pos[v] != NONE_POS
    }

    pub fn size(&self) -> usize {
        self.que.len()
    }

    pub fn get_first(&self) -> Option<usize> {
        self.que.get(0).copied()
    }

    pub fn get_random(&self, rnd: &mut impl Random) -> Option<usize> {
        self.que.get(rnd.gen_index(self.que.len())).copied()
    }

    pub fn iter(&self) -> impl Iterator<Item = &usize> {
        self.que.iter()
    }
}

#[derive(Debug, Clone)]
pub struct IndexMap<T> {
    set: IndexSet,
    vals: Vec<Option<T>>,
}

impl<T> IndexMap<T>
where
    T: Clone + Copy + Default,
{
    pub fn new(n: usize) -> Self {
        IndexMap {
            set: IndexSet::empty(n),
            vals: vec![None; n],
        }
    }

    pub fn add(&mut self, idx: usize, val: T) {
        if !self.set.contains(idx) {
            self.set.add(idx);
        }
        self.vals[idx] = Some(val);
    }

    pub fn remove(&mut self, idx: usize) {
        if self.set.contains(idx) {
            self.set.remove(idx);
            self.vals[idx] = None;
        }
    }

    pub fn get(&self, idx: usize) -> Option<T> {
        self.vals[idx]
    }

    pub fn iter(&self) -> impl Iterator<Item = (usize, T)> + '_ {
        self.set
            .iter()
            .map(move |&idx| (idx, self.vals[idx].expect("Somehow IndexMap is invalid")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_set() {
        let mut s = IndexSet::empty(5);
        assert_eq!(s.size(), 0);
        assert!(!s.contains(3));
        s.remove(3);
        assert_eq!(s.size(), 0);
        s.add(3);
        assert_eq!(s.size(), 1);
        assert!(s.contains(3));
        s.add(3);
        assert_eq!(s.size(), 1);
        assert!(s.contains(3));
        s.add(1);
        assert_eq!(s.size(), 2);
        assert!(s.contains(1));
        s.remove(3);
        assert_eq!(s.size(), 1);
        assert!(!s.contains(3));
        s.remove(3);
        assert_eq!(s.size(), 1);
        assert!(!s.contains(3));
        s.remove(1);
        assert_eq!(s.size(), 0);
        assert!(!s.contains(1));
    }

    #[test]
    fn test_index_set_full() {
        let mut s = IndexSet::full(5);
        assert_eq!(s.size(), 5);
        assert!(s.contains(3));
        s.add(3);
        assert_eq!(s.size(), 5);
        s.remove(3);
        assert_eq!(s.size(), 4);
        assert!(!s.contains(3));
        s.remove(3);
        assert_eq!(s.size(), 4);
        assert!(!s.contains(3));
        s.remove(1);
        assert_eq!(s.size(), 3);
        assert!(!s.contains(1));
        s.add(3);
        assert_eq!(s.size(), 4);
        assert!(s.contains(3));
        s.add(3);
        assert_eq!(s.size(), 4);
        assert!(s.contains(3));
        s.add(1);
        assert_eq!(s.size(), 5);
        assert!(s.contains(1));
    }

    #[test]
    fn test_index_map() {
        let mut m = IndexMap::new(5);
        assert_eq!(m.get(2), None);
        m.add(2, 10);
        assert_eq!(m.get(2), Some(10));
        m.add(2, 20);
        assert_eq!(m.get(2), Some(20));
        m.add(3, 30);
        assert_eq!(m.get(3), Some(30));
        m.remove(2);
        assert_eq!(m.get(2), None);
        m.remove(2);
        assert_eq!(m.get(2), None);
        m.add(2, 40);
        assert_eq!(m.get(2), Some(40));

        let mut v = m.iter().collect::<Vec<(usize, usize)>>();
        v.sort_by_key(|&(k, _)| k);
        assert_eq!(v, vec![(2, 40), (3, 30)]);
    }

    #[test]
    fn test_index_set_clear() {
        let mut s = IndexSet::empty(5);
        s.add(1);
        s.add(3);
        assert_eq!(s.size(), 2);
        s.clear();
        assert_eq!(s.size(), 0);
        assert!(!s.contains(1));
        assert!(!s.contains(3));
        s.add(2);
        assert_eq!(s.size(), 1);
        assert!(s.contains(2));
        s.add(1);
        assert_eq!(s.size(), 2);
        assert!(s.contains(1));
    }
}
