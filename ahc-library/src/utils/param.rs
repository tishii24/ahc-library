#[macro_export]
macro_rules! params_impl {
    (
        { $( $arg:ident : $arg_ty:ty ),* $(,)? },
        { $( $pname:ident : $pty:ty ),* $(,)? },
        [ $( $pat:pat => { $( $fname:ident : $fval:expr ),* $(,)? } ),+ $(,)? ]
    ) => {
        #[allow(non_snake_case, unused)]
        #[derive(Debug, Clone)]
        pub struct Params {
            $( pub $pname: $pty, )*
        }

        impl Params {
            pub fn load( $( $arg: $arg_ty ),* ) -> Self {
                match ( $( $arg ),* ) {
                    $(
                        $pat => Self {
                            $( $fname: $fval, )*
                        },
                    )*
                }
            }
        }
    };

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

#[test]
fn test_global_params() {
    mod p {
        use std::sync::LazyLock;

        params_impl! {
            START_TEMP: f64 = 1000.0,
            END_TEMP: f64 = 1.0,
        }

        pub static PARAMS: LazyLock<Params> = LazyLock::new(|| Params::load());
    }

    use p::PARAMS;
    assert_eq!(PARAMS.START_TEMP, 1000.0);
    assert_eq!(PARAMS.END_TEMP, 1.0);
}

#[test]
fn test_define_params_with_ranges() {
    params_impl! {
        { n: usize, m: usize },
        { START_TEMP: f64, END_TEMP: f64 },
        [
            (0..10, 10..20) => { START_TEMP: 1000., END_TEMP: 10. },
            (10..20, 10..20) => { START_TEMP: 2000., END_TEMP: 20. },
            _ => { START_TEMP: 2000., END_TEMP: 20. },
        ]
    }

    let p1 = Params::load(5, 15);
    assert_eq!(p1.START_TEMP, 1000.);
    assert_eq!(p1.END_TEMP, 10.);

    let p2 = Params::load(12, 15);
    assert_eq!(p2.START_TEMP, 2000.);
    assert_eq!(p2.END_TEMP, 20.);

    let p3 = Params::load(100, 100);
    assert_eq!(p3.START_TEMP, 2000.);
    assert_eq!(p3.END_TEMP, 20.);
}
