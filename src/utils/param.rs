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
                        $name: std::env::var(stringify!($name))
                            .ok()
                            .and_then(|v| v.parse().ok())
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
        start_temp: f64 = 1000.0,
        end_temp: f64 = 1.0,
    }
    let params = Params::load();
    println!("{:?}", params);
    assert_eq!(params.start_temp, 1000.0);
    assert_eq!(params.end_temp, 1.0);
}
