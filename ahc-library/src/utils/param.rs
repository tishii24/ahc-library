#[macro_export]
macro_rules! params_impl {
    (
        $(
            $name:ident: $type:ty = $default:expr
        ),* $(,)?
    ) => {
        #[allow(non_snake_case, unused)]
        #[derive(Debug, Clone)]
        pub struct Params {
            $(
                pub $name: $type,
            )*
        }

        impl Params {
            fn load() -> Self {
                Self {
                    $(
                        // $name: option_env!(stringify!($name))
                        //     .map(|v| v.parse::<$type>().unwrap())
                        //     .unwrap_or($default),
                        // $name: $default,
                        $name: std::env::var(stringify!($name))
                            .ok()
                            .map(|v| v.parse::<$type>().unwrap())
                            .unwrap_or($default),
                    )*
                }
            }
        }
    };
}

struct A {
    a: f64,
    b: f64,
}

impl A {
    fn new(n: usize, m: usize) -> Self {
        match (n, m) {
            (0..=1, 0..=1) => Self { a: 0.0, b: 0.0 },
            (0..=2, 0..=2) => Self { a: 1.0, b: 2.0 },
            (2, 3) => Self { a: 2.0, b: 3.0 },
            _ => Self { a: 0.0, b: 0.0 },
        }
    }
}

#[test]
fn test_define_params() {
    params_impl! {
        START_TEMP: f64 = 1000.0,
        END_TEMP: f64 = 1.0,
    }
    let params = Params::load();
    println!("{:?}", params);
    assert_eq!(params.START_TEMP, 1000.0);
    assert_eq!(params.END_TEMP, 1.0);
}
