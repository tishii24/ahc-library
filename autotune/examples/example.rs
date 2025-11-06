use std::process::Command;

fn main() {
    Command::new("cargo")
        .args(&["install", "--path", "."])
        .status()
        .unwrap();
    assert!(Command::new("autotune")
        .args(&["--config_path", "config_example.yaml"])
        .current_dir("examples")
        .status()
        .is_ok());
}
