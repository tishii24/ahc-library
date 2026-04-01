use std::path::PathBuf;

use toml::Value;

pub struct PahcerConfig {
    pub config_path: PathBuf,
    pub out_dir: PathBuf,
    pub stdin_dir: PathBuf,
    pub stdout_dir: PathBuf,
    pub stderr_dir: PathBuf,
    pub case_num_per_group: usize,
}
impl PahcerConfig {
    pub fn new(work_dir: &PathBuf, group_id: &str, case_num_per_group: usize) -> Self {
        Self {
            config_path: work_dir.join(group_id).join("pahcer_config.toml"),
            out_dir: work_dir.join(group_id).join("pahcer"),
            stdin_dir: work_dir.join(group_id).join("in"),
            stdout_dir: work_dir.join(group_id).join("out"),
            stderr_dir: work_dir.join(group_id).join("err"),
            case_num_per_group,
        }
    }

    pub fn build_all(&self, base_pahcer_toml: &str) -> anyhow::Result<()> {
        std::fs::create_dir_all(self.config_path.parent().unwrap())?;
        std::fs::create_dir_all(&self.out_dir)?;
        std::fs::create_dir_all(self.stdin_dir.parent().unwrap())?;
        std::fs::create_dir_all(self.stdout_dir.parent().unwrap())?;
        std::fs::create_dir_all(self.stderr_dir.parent().unwrap())?;

        let config_str = generate_pahcer_config(self, base_pahcer_toml);
        std::fs::write(&self.config_path, config_str)?;
        Ok(())
    }
}

/// `base_pahcer_file`を元に、`PahcerConfig` の内容を反映した pahcer 設定を作成する
/// 変更する箇所は以下の通り
/// - `test.out_dir`: `{work_dir}/{group_id}/pahcer`
/// - `test.end_seed`: `{case_num_per_group}`
/// - `test.test_steps[0].stdin`: `{work_dir}/{group_id}/in/{SEED04}.txt`
/// - `test.test_steps[0].stdout`: `{work_dir}/{group_id}/out/{SEED04}.txt`
/// - `test.test_steps[0].stderr`: `{work_dir}/{group_id}/err/{SEED04}.txt`
/// - `test.test_steps[1].args`: `["{work_dir}/{group_id}/in/{SEED04}.txt", "{work_dir}/{group_id}/out/{SEED04}.txt"]`
fn generate_pahcer_config(pahcer_config: &PahcerConfig, base_pahcer_toml: &str) -> String {
    let mut config: Value = toml::from_str(base_pahcer_toml).expect("invalid pahcer config toml");
    let test = config
        .get_mut("test")
        .and_then(Value::as_table_mut)
        .expect("missing [test] table");
    test.insert(
        "out_dir".to_string(),
        Value::String(format!("{}", pahcer_config.out_dir.display())),
    );
    test.insert(
        "end_seed".to_string(),
        Value::Integer(pahcer_config.case_num_per_group as i64),
    );

    let first_test_step = test
        .get_mut("test_steps")
        .and_then(Value::as_array_mut)
        .and_then(|steps| steps.first_mut())
        .and_then(Value::as_table_mut)
        .expect("missing test.test_steps[0]");

    first_test_step.insert(
        "stdin".to_string(),
        Value::String(format!(
            "{}/{{SEED04}}.txt",
            pahcer_config.stdin_dir.display()
        )),
    );
    first_test_step.insert(
        "stdout".to_string(),
        Value::String(format!(
            "{}/{{SEED04}}.txt",
            pahcer_config.stdout_dir.display()
        )),
    );
    first_test_step.insert(
        "stderr".to_string(),
        Value::String(format!(
            "{}/{{SEED04}}.txt",
            pahcer_config.stderr_dir.display()
        )),
    );

    let second_test_step = test
        .get_mut("test_steps")
        .and_then(Value::as_array_mut)
        .and_then(|steps| steps.get_mut(1))
        .and_then(Value::as_table_mut)
        .expect("missing test.test_steps[1]");
    second_test_step.insert(
        "args".to_string(),
        Value::Array(vec![
            Value::String(format!(
                "{}/{{SEED04}}.txt",
                pahcer_config.stdin_dir.display()
            )),
            Value::String(format!(
                "{}/{{SEED04}}.txt",
                pahcer_config.stdout_dir.display()
            )),
        ]),
    );

    toml::to_string_pretty(&config).expect("failed to serialize pahcer config toml")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_pahcer_config_rewrites_target_paths() {
        let base_pahcer_toml = r#"
[test]
start_seed = 0
end_seed = 10
out_dir = "./pahcer"

[[test.test_steps]]
program = "./example"
stdin = "./tools/in/{SEED04}.txt"
stdout = "./tools/out/{SEED04}.txt"
stderr = "./tools/err/{SEED04}.txt"

[[test.test_steps]]
args = ["./tools/in/{SEED04}.txt", "./tools/out/{SEED04}.txt"]
"#;

        let pahcer_config = PahcerConfig::new(&PathBuf::from("./autotune"), "group_a", 20);
        let actual = generate_pahcer_config(&pahcer_config, base_pahcer_toml);
        let expected = r#"[test]
start_seed = 0
end_seed = 20
out_dir = "./autotune/group_a/pahcer"

[[test.test_steps]]
program = "./example"
stderr = "./autotune/group_a/err/{SEED04}.txt"
stdin = "./autotune/group_a/in/{SEED04}.txt"
stdout = "./autotune/group_a/out/{SEED04}.txt"

[[test.test_steps]]
args = [
    "./autotune/group_a/in/{SEED04}.txt",
    "./autotune/group_a/out/{SEED04}.txt",
]
"#;

        let actual: Value = toml::from_str(&actual).expect("invalid generated pahcer config toml");
        let expected: Value =
            toml::from_str(&expected).expect("invalid expected pahcer config toml");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_generate_pahcer_config_preserves_unrelated_settings() {
        let base_pahcer_toml = r#"
[general]
version = "0.3.1"

[problem]
problem_name = "example"

[test]
start_seed = 0
end_seed = 10
threads = 4
out_dir = "./pahcer"

[[test.compile_steps]]
program = "cargo"
args = ["build", "--release"]

[[test.test_steps]]
program = "./example"
args = []
stdin = "./tools/in/{SEED04}.txt"
stdout = "./tools/out/{SEED04}.txt"
stderr = "./tools/err/{SEED04}.txt"
measure_time = true

[[test.test_steps]]
program = "./vis"
args = ["./tools/in/{SEED04}.txt", "./tools/out/{SEED04}.txt"]
measure_time = false
"#;

        let pahcer_config = PahcerConfig::new(&PathBuf::from("./autotune"), "x", 30);
        let actual = generate_pahcer_config(&pahcer_config, base_pahcer_toml);
        let expected = r#"[general]
version = "0.3.1"

[problem]
problem_name = "example"

[test]
start_seed = 0
end_seed = 30
threads = 4
out_dir = "./autotune/x/pahcer"

[[test.compile_steps]]
args = [
    "build",
    "--release",
]
program = "cargo"

[[test.test_steps]]
args = []
measure_time = true
program = "./example"
stderr = "./autotune/x/err/{SEED04}.txt"
stdin = "./autotune/x/in/{SEED04}.txt"
stdout = "./autotune/x/out/{SEED04}.txt"

[[test.test_steps]]
args = [
    "./autotune/x/in/{SEED04}.txt",
    "./autotune/x/out/{SEED04}.txt",
]
measure_time = false
program = "./vis"
"#;

        let actual: Value = toml::from_str(&actual).expect("invalid generated pahcer config toml");
        let expected: Value =
            toml::from_str(&expected).expect("invalid expected pahcer config toml");
        assert_eq!(actual, expected);
    }
}
