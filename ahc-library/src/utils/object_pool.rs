use crate::utils::index_set::IndexSet;

/// プール可能なオブジェクトのトレイト
pub trait Poolable: Clone {
    fn new() -> Self;
    fn reset(&mut self);
}

#[derive(Clone, Debug)]
pub struct ObjectPool<T: Poolable> {
    objects: Vec<Option<T>>,
    can_use_slots: IndexSet,
    empty_slots: IndexSet,
}

impl<T: Poolable> ObjectPool<T> {
    pub fn new(pool_size: usize) -> ObjectPool<T> {
        ObjectPool {
            objects: vec![None; pool_size],
            can_use_slots: IndexSet::full(pool_size),
            empty_slots: IndexSet::empty(pool_size),
        }
    }

    /// オブジェクトをプールに戻す
    pub fn pool(&mut self, obj: T) -> Option<usize> {
        if let Some(idx) = self.empty_slots.first() {
            self.objects[idx] = Some(obj);
            self.empty_slots.remove(idx);
            self.can_use_slots.add(idx);
            Some(idx)
        } else {
            None
        }
    }

    /// プールからオブジェクトを取得する
    pub fn get_new(&mut self) -> Option<T> {
        if let Some(idx) = self.can_use_slots.first() {
            self.can_use_slots.remove(idx);
            self.empty_slots.add(idx);
            let mut obj = self.objects[idx].take().unwrap_or(T::new());
            obj.reset();
            Some(obj)
        } else {
            None
        }
    }

    pub fn n_remain(&self) -> usize {
        self.can_use_slots.size()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug)]
    struct TestObj {
        val: i32,
    }
    impl Poolable for TestObj {
        fn new() -> Self {
            TestObj { val: 0 }
        }
        fn reset(&mut self) {
            self.val = 0;
        }
    }

    #[test]
    fn test_object_pool() {
        let mut pool: ObjectPool<TestObj> = ObjectPool::new(2);
        assert_eq!(pool.n_remain(), 2);
        let mut obj1 = pool.get_new().unwrap();
        assert_eq!(pool.n_remain(), 1);
        let _ = pool.get_new().unwrap();
        assert_eq!(pool.n_remain(), 0);
        assert!(pool.get_new().is_none());
        obj1.val = 42;
        pool.pool(obj1);
        assert_eq!(pool.n_remain(), 1);
        let obj3 = pool.get_new().unwrap();
        assert_eq!(obj3.val, 0);
        assert_eq!(pool.n_remain(), 0);
    }
}
