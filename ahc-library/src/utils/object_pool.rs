use crate::utils::{index_set::IndexSet, random::Random};

#[derive(Clone, Debug)]
pub struct ObjectPool<T: Clone> {
    objects: Vec<Option<T>>,
    is_active: IndexSet,
    is_inactive: IndexSet,
}

impl<T: Clone> ObjectPool<T> {
    pub fn new(objects: Vec<T>, pool_size: usize) -> ObjectPool<T> {
        let mut pool = ObjectPool {
            is_active: IndexSet::empty(pool_size),
            is_inactive: IndexSet::full(pool_size),
            objects: vec![None; pool_size],
        };
        for obj in objects {
            pool.add(obj);
        }
        pool
    }

    pub fn get_ref(&self, idx: usize) -> Option<&T> {
        self.objects[idx].as_ref()
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut T> {
        self.objects[idx].as_mut()
    }

    pub fn add(&mut self, group: T) -> Option<usize> {
        let idx = self.is_inactive.get_first()?;
        self.objects[idx] = Some(group);
        self.is_inactive.remove(idx);
        self.is_active.add(idx);
        Some(idx)
    }

    pub fn remove(&mut self, idx: usize) -> Option<T> {
        self.is_active.remove(idx);
        self.is_inactive.add(idx);
        self.objects[idx].take()
    }

    pub fn size(&self) -> usize {
        self.is_active.size()
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.is_active
            .iter()
            .map(move |&idx| self.get_ref(idx).expect("Somehow is_active is invalid"))
    }

    pub fn iter_idx(&self) -> impl Iterator<Item = &usize> {
        self.is_active.iter()
    }

    pub fn get_random_active_index(&self, rnd: &mut impl Random) -> Option<usize> {
        self.is_active.get_random(rnd)
    }
}
