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
macro_rules! define_vec2 {
    (
        $name:ident,
        $f1:ident,
        $f2:ident
    ) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
        pub struct $name<T>
        where
            T: num_traits::Num,
        {
            pub $f1: T,
            pub $f2: T,
        }

        /* ---------- binary ops ---------- */
        macro_rules! impl_bin_op {
            ($Trait:ident, $method:ident, $op:tt) => {
                impl<T> std::ops::$Trait for $name<T>
                where
                    T: num_traits::Num + std::ops::$Trait<Output = T>,
                {
                    type Output = Self;
                    fn $method(self, other: Self) -> Self {
                        Self {
                            $f1: self.$f1 $op other.$f1,
                            $f2: self.$f2 $op other.$f2,
                        }
                    }
                }

                impl<T> std::ops::$Trait<T> for $name<T>
                where
                    T: num_traits::Num
                        + Copy
                        + std::ops::Add<Output = T>
                        + std::ops::Sub<Output = T>
                        + std::ops::Mul<Output = T>
                        + std::ops::Div<Output = T>,
                {
                    type Output = Self;
                    fn $method(self, factor: T) -> Self {
                        Self {
                            $f1: self.$f1 $op factor,
                            $f2: self.$f2 $op factor,
                        }
                    }
                }
            };
        }

        macro_rules! impl_assign_op {
            ($Trait:ident, $method:ident, $op:tt) => {
                impl<T> std::ops::$Trait for $name<T>
                where
                    T: num_traits::Num + std::ops::$Trait,
                {
                    fn $method(&mut self, other: Self) {
                        self.$f1 $op other.$f1;
                        self.$f2 $op other.$f2;
                    }
                }

                impl<T> std::ops::$Trait<T> for $name<T>
                where
                    T: num_traits::Num + std::ops::$Trait + Copy,
                {
                    fn $method(&mut self, factor: T) {
                        self.$f1 $op factor;
                        self.$f2 $op factor;
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

        /* ---------- misc ---------- */
        impl<T> std::fmt::Display for $name<T>
        where
            T: num_traits::Num + std::fmt::Display,
        {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "({}, {})", self.$f1, self.$f2)
            }
        }

        impl<T> num_traits::Zero for $name<T>
        where
            T: num_traits::Num,
        {
            fn zero() -> Self {
                Self {
                    $f1: T::zero(),
                    $f2: T::zero(),
                }
            }

            fn is_zero(&self) -> bool {
                self.$f1.is_zero() && self.$f2.is_zero()
            }
        }

        impl<T> $name<T>
        where
            T: num_traits::Num,
        {
            pub fn new($f1: T, $f2: T) -> Self {
                Self { $f1, $f2 }
            }
        }
    };
}

define_vec2!(V2, x, y);
define_vec2!(Coor, i, j);

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

#[test]
fn test_macro() {
    define_vec2!(TestV2, i, j);
    let c1 = TestV2::new(2_usize, 3_usize);
    let c2 = TestV2::new(4_usize, 5_usize);
    let c3 = c1 + c2;
    assert_eq!(c3.i, 6);
    assert_eq!(c3.j, 8);
}
