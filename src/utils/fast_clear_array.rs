#[derive(Debug)]
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
            self.values[i].0 = self.version;
            self.values[i].1 = self.init_value;
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
