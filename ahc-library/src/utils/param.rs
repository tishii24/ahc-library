/// `Params` 構造体と、その読み込み処理を定義するマクロ
///
/// このマクロは、以下の2つの形式に対応しています
///
/// # 1. 環境変数ベースのパラメータ定義
///
/// 各パラメータにデフォルト値を与えつつ、同名の環境変数で上書きできるようにしたい場合に使います
///
/// ```ignore
/// params_impl! {
///     START_TEMP: f64 = 1000.0,
///     END_TEMP: f64 = 1.0,
/// }
///
/// let params = Params::load();
/// assert_eq!(params.START_TEMP, 1000.0);
/// ```
///
/// # 2. パターンマッチによるパラメータテーブル定義
///
/// 問題サイズやモードなど、実行時の入力値に応じてパラメータを切り替えたい場合に使います
///
/// ```ignore
/// params_impl! {
///     { START_TEMP: f64, END_TEMP: f64 },
///     [
///         "group_0" => { START_TEMP: 1000.0, END_TEMP: 10.0 },
///         "group_1" => { START_TEMP: 5000.0, END_TEMP: 100.0 },
///         _ => { START_TEMP: 2000.0, END_TEMP: 20.0 },
///     ]
/// }
///
/// let params = Params::load("group_0");
/// assert_eq!(params.START_TEMP, 1000.0);
/// ```
#[macro_export]
macro_rules! params_impl {
    (
        { $( $pname:ident : $pty:ty ),* $(,)? },
        [ $( $pat:pat => { $( $fname:ident : $fval:expr ),* $(,)? } ),+ $(,)? ]
    ) => {
        #[allow(non_snake_case, unused)]
        #[derive(Debug, Clone)]
        pub struct Params {
            $( pub $pname: $pty, )*
        }

        impl Params {
            pub fn load(group_id: &str) -> Self {
                match group_id {
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

#[cfg(test)]
mod tests {
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
    #[ignore] // it is flaky...
    fn test_define_params_with_env() {
        temp_env::with_vars(
            [("START_TEMP", Some("5000.0")), ("END_TEMP", Some("5.0"))],
            || {
                params_impl! {
                    START_TEMP: f64 = 1000.0,
                    END_TEMP: f64 = 1.0,
                }

                let params = Params::load();
                println!("{:?}", params);
                assert_eq!(params.START_TEMP, 5000.0);
                assert_eq!(params.END_TEMP, 5.0);
            },
        );
    }

    #[test]
    fn test_global_params() {
        mod p {
            use std::sync::LazyLock;

            params_impl! {
                START_TEMP: f64 = 1000.0,
                END_TEMP: f64 = 1.0,
            }

            pub static PARAMS: LazyLock<Params> = LazyLock::new(Params::load);
        }

        use p::PARAMS;
        assert_eq!(PARAMS.START_TEMP, 1000.0);
        assert_eq!(PARAMS.END_TEMP, 1.0);
    }

    #[test]
    fn test_pattern_params() {
        params_impl! {
            { START_TEMP: f64, END_TEMP: f64 },
            [
                "group_0" => { START_TEMP: 1000.0, END_TEMP: 10.0 },
                "group_1" => { START_TEMP: 5000.0, END_TEMP: 100.0 },
                _ => { START_TEMP: 2000.0, END_TEMP: 20.0 },
            ]
        }
        let params = Params::load("group_0");
        println!("{:?}", params);
        assert_eq!(params.START_TEMP, 1000.0);
        assert_eq!(params.END_TEMP, 10.0);
    }
}
