#[derive(Clone)]
pub struct Array2d<T>
where
    T: Clone + Copy,
{
    pub h: usize,
    pub w: usize,
    values: Vec<T>,
}

impl<T> Array2d<T>
where
    T: Clone + Copy,
{
    /// expect: values[i].len() = const.
    pub fn new(values: Vec<Vec<T>>) -> Array2d<T> {
        let h = values.len();
        let w = values[0].len();
        let values = values.into_iter().flatten().collect();
        Array2d { h, w, values }
    }

    pub fn init(h: usize, w: usize, init_value: T) -> Array2d<T> {
        let values = vec![init_value; h * w];
        Array2d { h, w, values }
    }

    #[inline]
    pub fn get(&self, c: &(usize, usize)) -> T {
        self.values[c.0 * self.w + c.1]
    }

    #[inline]
    pub fn set(&mut self, c: &(usize, usize), v: T) {
        self.values[c.0 * self.w + c.1] = v;
    }
}

impl<T> From<Vec<Vec<T>>> for Array2d<T>
where
    T: Clone + Copy,
{
    fn from(values: Vec<Vec<T>>) -> Self {
        Array2d::new(values)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_array2d() {
        let mut a = super::Array2d::new(vec![vec![0; 3]; 2]);
        assert_eq!(a.w, 3);
        assert_eq!(a.get(&(1, 2)), 0);
        a.set(&(1, 2), 5);
        assert_eq!(a.get(&(1, 2)), 5);
    }
}
