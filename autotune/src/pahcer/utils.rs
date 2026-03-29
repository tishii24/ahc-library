/// `base_pahcer_file`を元に、`pahcer_config_{key}.toml`を作成する
/// 変更する箇所は以下の通り
/// - `test.out_dir`: `autotune/pahcer/{key}`
/// - `test.test_steps[0].stdin`: `autotune/in/{key}/{SEED04}.txt"`
/// - `test.test_steps[0].stdout`: `autotune/out/{key}/{SEED04}.txt"`
/// - `test.test_steps[0].stderr`: `autotune/err/{key}/{SEED04}.txt"`
pub fn generate_pahcer_config(key: &str, base_pahcer_toml: &str) -> String {
    todo!()
}
