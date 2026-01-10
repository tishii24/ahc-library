use std::ops::{Index, IndexMut};

use super::v2::V2;

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

impl<T> Index<V2<usize>> for ArrayRef2d<T>
where
    T: Clone,
{
    type Output = T;

    #[inline]
    fn index(&self, v: V2<usize>) -> &Self::Output {
        &self.values[v.x * self.w + v.y]
    }
}

impl<T> IndexMut<V2<usize>> for ArrayRef2d<T>
where
    T: Clone,
{
    #[inline]
    fn index_mut(&mut self, v: V2<usize>) -> &mut Self::Output {
        &mut self.values[v.x * self.w + v.y]
    }
}

impl<T> Index<(usize, usize)> for ArrayRef2d<T>
where
    T: Clone,
{
    type Output = T;

    #[inline]
    fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
        &self.values[x * self.w + y]
    }
}

impl<T> IndexMut<(usize, usize)> for ArrayRef2d<T>
where
    T: Clone,
{
    #[inline]
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut Self::Output {
        &mut self.values[x * self.w + y]
    }
}

/// A 2D array stored in a 1D Vec.
/// The element type T must implement Clone and Copy.
///
/// # Example
/// ```
/// use ahc_library::utils::ndarray::Array2d;
/// let mut array = Array2d::new(vec![vec![1, 2, 3], vec![4, 5, 6]]);
/// assert_eq!(array[(0, 0)], 1);
/// assert_eq!(array[(1, 2)], 6);
/// array[(0, 1)] = 10;
/// assert_eq!(array[(0, 1)], 10);
/// ```
///
/// You can use V2<usize> as index as well:
/// ```
/// use ahc_library::utils::{ndarray::Array2d, v2::V2};
/// let mut array = Array2d::new(vec![vec![1, 2, 3], vec![4, 5, 6]]);
/// let v = V2::new(1, 0);
/// assert_eq!(array[v], 4);
/// array[v] = 20;
/// assert_eq!(array[v], 20);
/// ```
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

impl<T> Index<V2<usize>> for Array2d<T>
where
    T: Clone + Copy,
{
    type Output = T;

    #[inline]
    fn index(&self, v: V2<usize>) -> &Self::Output {
        &self.values[v.x * self.w + v.y]
    }
}

impl<T> IndexMut<V2<usize>> for Array2d<T>
where
    T: Clone + Copy,
{
    #[inline]
    fn index_mut(&mut self, v: V2<usize>) -> &mut Self::Output {
        &mut self.values[v.x * self.w + v.y]
    }
}

impl<T> Index<(usize, usize)> for Array2d<T>
where
    T: Clone + Copy,
{
    type Output = T;

    #[inline]
    fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
        &self.values[x * self.w + y]
    }
}

impl<T> IndexMut<(usize, usize)> for Array2d<T>
where
    T: Clone + Copy,
{
    #[inline]
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut Self::Output {
        &mut self.values[x * self.w + y]
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
}

impl<T> Index<(usize, usize, usize)> for Array3d<T>
where
    T: Clone + Copy,
{
    type Output = T;

    #[inline]
    fn index(&self, (x, y, z): (usize, usize, usize)) -> &Self::Output {
        &self.values[x * self.d1 * self.d2 + y * self.d2 + z]
    }
}

impl<T> IndexMut<(usize, usize, usize)> for Array3d<T>
where
    T: Clone + Copy,
{
    #[inline]
    fn index_mut(&mut self, (x, y, z): (usize, usize, usize)) -> &mut Self::Output {
        &mut self.values[x * self.d1 * self.d2 + y * self.d2 + z]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_array2d_index_v2() {
        let mut a = Array2d::new(vec![vec![0; 3]; 2]);
        let v = V2::new(1, 2);
        assert_eq!(a[v], 0);
        a[v] = 5;
        assert_eq!(a[v], 5);
    }

    #[test]
    fn test_array2d_index_tuple() {
        let mut a = Array2d::new(vec![vec![0; 3]; 2]);
        assert_eq!(a[(1, 2)], 0);
        a[(1, 2)] = 5;
        assert_eq!(a[(1, 2)], 5);
    }

    #[test]
    fn test_arrayref2d_index_v2() {
        let mut a = ArrayRef2d::init(3, 4, String::from("test"));
        let v = V2::new(1, 2);
        assert_eq!(a[v], "test");
        a[v] = String::from("modified");
        assert_eq!(a[v], "modified");
    }

    #[test]
    fn test_arrayref2d_index_tuple() {
        let mut a = ArrayRef2d::init(3, 4, String::from("test"));
        assert_eq!(a[(1, 2)], "test");
        a[(1, 2)] = String::from("modified");
        assert_eq!(a[(1, 2)], "modified");
    }
}
