#[macro_export]
macro_rules! params_impl {
    (
        $(
            $name:ident: $type:ty = $default:expr
        ),* $(,)?
    ) => {
        #[allow(non_snake_case)]
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
                        $name: option_env!(stringify!($name))
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
