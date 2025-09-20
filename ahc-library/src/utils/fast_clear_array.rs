use crate::utils::{array2d::Array2d, coor::Coor};

pub struct FastClearArray<T: Clone + Copy> {
    pub version: usize,
    pub values: Vec<(usize, T)>,
    pub init_value: T,
}

impl<T: Clone + Copy> FastClearArray<T> {
    pub fn new(n: usize, init_value: T) -> FastClearArray<T> {
        FastClearArray {
            version: 0,
            values: vec![(!0, init_value); n],
            init_value: init_value,
        }
    }

    #[inline]
    pub fn get(&mut self, i: usize) -> T {
        if self.values[i].0 != self.version {
            self.values[i] = (self.version, self.init_value);
        }
        self.values[i].1
    }

    #[inline]
    pub fn set(&mut self, i: usize, new_value: T) {
        self.values[i] = (self.version, new_value);
    }

    pub fn clear(&mut self) {
        self.version += 1;
    }
}

pub struct FastClearArray2d<T: Clone + Copy> {
    pub version: usize,
    pub values: Array2d<(usize, T)>,
    pub init_value: T,
}

impl<T: Clone + Copy> FastClearArray2d<T> {
    pub fn new(h: usize, w: usize, init_value: T) -> FastClearArray2d<T> {
        FastClearArray2d {
            version: 0,
            values: Array2d::new(vec![vec![(!0, init_value); w]; h]),
            init_value,
        }
    }

    #[inline]
    pub fn get(&mut self, c: &Coor) -> T {
        if self.values.get(c).0 != self.version {
            self.values.set(c, (self.version, self.init_value));
        }
        self.values.get(c).1
    }

    #[inline]
    pub fn set(&mut self, c: &Coor, new_value: T) {
        self.values.set(c, (self.version, new_value));
    }

    pub fn clear(&mut self) {
        self.version += 1;
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::coor::Coor;

    #[test]
    fn test_fast_clear_array() {
        let mut arr = super::FastClearArray::new(10, 0);
        assert_eq!(arr.get(3), 0);
        arr.set(3, 5);
        assert_eq!(arr.get(3), 5);
        arr.clear();
        assert_eq!(arr.get(3), 0);
        arr.set(3, 7);
        assert_eq!(arr.get(3), 7);
    }

    #[test]
    fn test_fast_clear_array_2d() {
        let mut arr = super::FastClearArray2d::new(3, 4, 0);
        assert_eq!(arr.get(&Coor::new(1, 2)), 0);
        arr.set(&Coor::new(1, 2), 5);
        assert_eq!(arr.get(&Coor::new(1, 2)), 5);
        arr.clear();
        assert_eq!(arr.get(&Coor::new(1, 2)), 0);
        arr.set(&Coor::new(1, 2), 7);
        assert_eq!(arr.get(&Coor::new(1, 2)), 7);
    }
}
