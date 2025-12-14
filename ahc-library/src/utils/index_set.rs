use crate::utils::random::Rnd;

#[derive(Debug, Clone)]
pub struct IndexSet {
    pub que: Vec<usize>,
    pub pos: Vec<usize>,
}

impl IndexSet {
    pub fn empty(n: usize) -> Self {
        IndexSet {
            que: Vec::with_capacity(n),
            pos: vec![!0; n],
        }
    }

    pub fn full(n: usize) -> Self {
        IndexSet {
            que: (0..n).collect(),
            pos: (0..n).collect(),
        }
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
        self.pos[v] = !0;
    }

    pub fn contains(&self, v: usize) -> bool {
        self.pos[v] != !0
    }

    pub fn size(&self) -> usize {
        self.que.len()
    }

    pub fn get_first(&self) -> Option<usize> {
        self.que.get(0).copied()
    }

    pub fn get_random(&self, rnd: &mut Rnd) -> Option<usize> {
        self.que.get(rnd.gen_index(self.que.len())).copied()
    }

    pub fn iter(&self) -> impl Iterator<Item = &usize> {
        self.que.iter()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_index_set() {
        let mut s = super::IndexSet::empty(5);
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
        let mut s = super::IndexSet::full(5);
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
}
