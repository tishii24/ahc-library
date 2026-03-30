use std::process::Command;

fn main() {
    Command::new("cargo")
        .args(&["install", "--path", "."])
        .status()
        .unwrap();
    assert!(Command::new("autotune")
        .args(&[
            "--config_path",
            "autotune_config.yaml",
            "--optuna_study_prefix",
            "study0"
        ])
        .current_dir("examples/example")
        .status()
        .map(|status| status.success())
        .unwrap_or(false));
}
