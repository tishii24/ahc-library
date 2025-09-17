use crate::utils::coor::Coor;

pub struct Array2d<T>
where
    T: Clone + Copy,
{
    pub w: usize,
    values: Vec<T>,
}

impl<T> Array2d<T>
where
    T: Clone + Copy,
{
    /// expect: values[i].len() = const.
    pub fn new(values: Vec<Vec<T>>) -> Array2d<T> {
        let w = values[0].len();
        let values = values.into_iter().flatten().collect();
        Array2d { w, values }
    }

    pub fn get(&self, c: Coor) -> T {
        self.values[c.i * self.w + c.j]
    }

    pub fn set(&mut self, c: Coor, v: T) {
        self.values[c.i * self.w + c.j] = v;
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::coor::Coor;

    #[test]
    fn test_array2d() {
        let mut a = super::Array2d::new(vec![vec![0; 3]; 2]);
        assert_eq!(a.w, 3);
        assert_eq!(a.get(Coor::new(1, 2)), 0);
        a.set(Coor::new(1, 2), 5);
        assert_eq!(a.get(Coor::new(1, 2)), 5);
    }
}
