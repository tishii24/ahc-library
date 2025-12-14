#[derive(Clone, Debug)]
pub struct ArrayRef2d<T>
where
    T: Clone,
{
    pub h: usize,
    pub w: usize,
    pub values: Vec<T>,
}

impl<T> ArrayRef2d<T>
where
    T: Clone,
{
    /// expect: values[i].len() = const.
    pub fn new(values: Vec<Vec<T>>) -> ArrayRef2d<T> {
        let h = values.len();
        let w = values[0].len();
        let values = values.into_iter().flatten().collect();
        ArrayRef2d { h, w, values }
    }

    pub fn init(h: usize, w: usize, init_value: T) -> ArrayRef2d<T> {
        let values = vec![init_value; h * w];
        ArrayRef2d { h, w, values }
    }

    #[inline]
    pub fn get_ref(&self, c: &(usize, usize)) -> &T {
        &self.values[c.0 * self.w + c.1]
    }

    #[inline]
    pub fn get_mut(&mut self, c: &(usize, usize)) -> &mut T {
        &mut self.values[c.0 * self.w + c.1]
    }

    #[inline]
    pub fn set(&mut self, c: &(usize, usize), v: T) {
        self.values[c.0 * self.w + c.1] = v;
    }

    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.values.iter()
    }
}

impl<T> From<Vec<Vec<T>>> for ArrayRef2d<T>
where
    T: Clone,
{
    fn from(values: Vec<Vec<T>>) -> Self {
        ArrayRef2d::new(values)
    }
}

#[derive(Clone, Debug)]
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

    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.values.iter()
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

#[derive(Clone, Debug)]
pub struct Array3d<T>
where
    T: Clone + Copy,
{
    pub d0: usize,
    pub d1: usize,
    pub d2: usize,
    values: Vec<T>,
}

impl<T> Array3d<T>
where
    T: Clone + Copy,
{
    pub fn init(d0: usize, d1: usize, d2: usize, init_value: T) -> Array3d<T> {
        let values = vec![init_value; d0 * d1 * d2];
        Array3d { d0, d1, d2, values }
    }

    #[inline]
    pub fn get(&self, c: &(usize, usize, usize)) -> T {
        self.values[c.0 * self.d1 * self.d2 + c.1 * self.d2 + c.2]
    }

    #[inline]
    pub fn set(&mut self, c: &(usize, usize, usize), v: T) {
        self.values[c.0 * self.d1 * self.d2 + c.1 * self.d2 + c.2] = v;
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
