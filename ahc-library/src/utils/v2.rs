use num_traits::{Num, Zero};
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

pub const D_DOWN: V2<usize> = V2 { x: 1, y: 0 };
pub const D_UP: V2<usize> = V2 { x: !0, y: 0 };
pub const D_LEFT: V2<usize> = V2 { x: 0, y: !0 };
pub const D_RIGHT: V2<usize> = V2 { x: 0, y: 1 };

pub const D4: [V2<usize>; 4] = [D_UP, D_DOWN, D_LEFT, D_RIGHT];

/// 2D Vector that supports basic arithmetic operations.
/// # Examples
/// ```
/// use ahc_library::utils::v2::V2;
/// let v1 = V2::new(1, 2);
/// let v2 = V2::new(3, 4);
/// let v3 = v1 + v2;
/// assert_eq!(v3.x, 4);
/// assert_eq!(v3.y, 6);
///
/// let mut v4 = V2::new(5, 6);
/// v4 += 10;
/// assert_eq!(v4.x, 15);
/// assert_eq!(v4.y, 16);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct V2<T>
where
    T: Num,
{
    pub x: T,
    pub y: T,
}

macro_rules! impl_bin_op {
    ($Trait:ident, $method:ident, $op:tt) => {
        impl<T> $Trait for V2<T>
        where
            T: Num + $Trait<Output = T>,
        {
            type Output = Self;
            fn $method(self, other: Self) -> Self {
                Self {
                    x: self.x $op other.x,
                    y: self.y $op other.y,
                }
            }
        }

        impl<T> $Trait<T> for V2<T>
        where
            T: Num + Copy + Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Div<Output = T>,
        {
            type Output = Self;
            fn $method(self, factor: T) -> Self {
                Self {
                    x: self.x $op factor,
                    y: self.y $op factor,
                }
            }
        }
    };
}

macro_rules! impl_assign_op {
    ($Trait:ident, $method:ident, $op:tt) => {
        impl<T> $Trait for V2<T>
        where
            T: Num + $Trait,
        {
            fn $method(&mut self, other: Self) {
                self.x $op other.x;
                self.y $op other.y;
            }
        }

        impl<T> $Trait<T> for V2<T>
        where
            T: Num + $Trait + Copy,
        {
            fn $method(&mut self, factor: T) {
                self.x $op factor;
                self.y $op factor;
            }
        }
    };
}

impl_bin_op!(Add, add, +);
impl_bin_op!(Sub, sub, -);
impl_bin_op!(Mul, mul, *);
impl_bin_op!(Div, div, /);

impl_assign_op!(AddAssign, add_assign, +=);
impl_assign_op!(SubAssign, sub_assign, -=);
impl_assign_op!(MulAssign, mul_assign, *=);
impl_assign_op!(DivAssign, div_assign, /=);

impl<T> std::fmt::Display for V2<T>
where
    T: Num + std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl<T> From<(T, T)> for V2<T>
where
    T: Num,
{
    fn from(tuple: (T, T)) -> Self {
        Self {
            x: tuple.0,
            y: tuple.1,
        }
    }
}

impl<T> From<&(T, T)> for V2<T>
where
    T: Num + Copy,
{
    fn from(tuple: &(T, T)) -> Self {
        Self {
            x: tuple.0,
            y: tuple.1,
        }
    }
}

impl<T> Into<(T, T)> for V2<T>
where
    T: Num + Copy,
{
    fn into(self) -> (T, T) {
        (self.x, self.y)
    }
}

impl<T> Zero for V2<T>
where
    T: Num,
{
    fn zero() -> Self {
        Self {
            x: T::zero(),
            y: T::zero(),
        }
    }

    fn is_zero(&self) -> bool {
        self.x.is_zero() && self.y.is_zero()
    }
}

impl<T> V2<T>
where
    T: Num,
{
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

#[test]
fn test_a() {
    let v1 = V2::new(1, 2);
    let v2 = V2::new(3, 4);
    let v3 = v1 + v2;
    assert_eq!(v3.x, 4);
    assert_eq!(v3.y, 6);

    let mut v4 = V2::new(5, 6);
    v4 += 10;
    assert_eq!(v4.x, 15);
    assert_eq!(v4.y, 16);
}
