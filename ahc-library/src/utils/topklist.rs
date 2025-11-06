/// https://atcoder.jp/contests/ahc052/submissions/68703195 より拝借
use std::collections::BinaryHeap;

#[derive(Clone, Debug)]
struct Entry<K, V> {
    k: K,
    v: V,
}

impl<K: PartialOrd, V> Ord for Entry<K, V> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.k.partial_cmp(&other.k).unwrap()
    }
}

impl<K: PartialOrd, V> PartialOrd for Entry<K, V> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.k.partial_cmp(&other.k)
    }
}

impl<K: PartialEq, V> PartialEq for Entry<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.k.eq(&other.k)
    }
}

impl<K: PartialEq, V> Eq for Entry<K, V> {}

#[derive(Clone, Debug)]
pub struct BoundedSortedList<K: PartialOrd + Copy, V: Clone> {
    que: BinaryHeap<Entry<K, V>>,
    size: usize,
}

impl<K: PartialOrd + Copy, V: Clone> BoundedSortedList<K, V> {
    pub fn new(size: usize) -> Self {
        Self {
            que: BinaryHeap::with_capacity(size),
            size,
        }
    }

    pub fn can_insert(&self, k: K) -> bool {
        self.que.len() < self.size || self.que.peek().unwrap().k > k
    }

    pub fn insert(&mut self, k: K, v: V) {
        if self.que.len() < self.size {
            self.que.push(Entry { k, v });
        } else if let Some(mut top) = self.que.peek_mut() {
            if top.k > k {
                top.k = k;
                top.v = v;
            }
        }
    }

    pub fn to_list(self) -> Vec<(K, V)> {
        let v = self.que.into_sorted_vec();
        v.into_iter().map(|e| (e.k, e.v)).collect()
    }

    pub fn len(&self) -> usize {
        self.que.len()
    }

    pub fn is_empty(&self) -> bool {
        self.que.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::topklist::BoundedSortedList;

    #[test]
    fn test_bounded_sorted_list() {
        let mut cands = BoundedSortedList::new(3);
        cands.insert(5, "a");
        cands.insert(3, "b");
        cands.insert(4, "c");
        assert_eq!(cands.len(), 3);
        assert!(!cands.can_insert(6));
        assert!(cands.can_insert(2));
        cands.insert(2, "d");
        let list = cands.to_list();
        assert_eq!(list, vec![(2, "d"), (3, "b"), (4, "c")]);
    }
}
