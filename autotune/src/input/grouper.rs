use std::{
    fs,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
    sync::Mutex,
};

use anyhow::{anyhow, Context};
use tempfile::TempDir;

pub trait InputGrouper {
    fn get_group_id(&self, input: &str) -> anyhow::Result<String>;
}

pub struct InputFnGrouper {
    input_fn: String,
    compiled: Mutex<Option<CompiledInputFn>>,
}

struct CompiledInputFn {
    _temp_dir: TempDir,
    bin_path: PathBuf,
}

impl InputFnGrouper {
    pub fn new(input_fn: String) -> Self {
        Self {
            input_fn,
            compiled: Mutex::new(None),
        }
    }

    fn ensure_compiled(&self) -> anyhow::Result<PathBuf> {
        let mut compiled = self
            .compiled
            .lock()
            .map_err(|_| anyhow!("failed to lock input grouper state"))?;
        if let Some(compiled) = compiled.as_ref() {
            return Ok(compiled.bin_path.clone());
        }

        let compiled_input_fn = self.compile_input_fn()?;
        let bin_path = compiled_input_fn.bin_path.clone();
        *compiled = Some(compiled_input_fn);
        Ok(bin_path)
    }

    fn compile_input_fn(&self) -> anyhow::Result<CompiledInputFn> {
        const CRATE_NAME: &str = "autotune_input_grouper";

        let temp_dir = tempfile::tempdir().context("failed to create temp dir")?;
        let src_dir = temp_dir.path().join("src");
        fs::create_dir_all(&src_dir).context("failed to create temp src dir")?;

        fs::write(
            temp_dir.path().join("Cargo.toml"),
            Self::cargo_toml(CRATE_NAME),
        )
        .context("failed to write temp Cargo.toml")?;
        fs::write(src_dir.join("main.rs"), self.generated_main_rs())
            .context("failed to write temp main.rs")?;

        let output = Command::new("cargo")
            .args(["build", "--release"])
            .current_dir(temp_dir.path())
            .output()
            .context("failed to build input grouper")?;

        if !output.status.success() {
            return Err(anyhow!(
                "failed to compile input grouper\nstdout:\n{}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let bin_path = temp_dir
            .path()
            .join("target")
            .join("release")
            .join(CRATE_NAME);

        Ok(CompiledInputFn {
            _temp_dir: temp_dir,
            bin_path,
        })
    }

    fn cargo_toml(crate_name: &str) -> String {
        format!(
            r#"[package]
name = "{crate_name}"
version = "0.1.0"
edition = "2021"

[dependencies]
proconio = "0.5"
"#
        )
    }

    fn generated_main_rs(&self) -> String {
        format!(
            r#"fn get_group() -> String {{
{input_fn}
}}

fn main() {{
    let group = get_group();
    println!("{{}}", group);
}}
"#,
            input_fn = self.input_fn
        )
    }
}

impl InputGrouper for InputFnGrouper {
    fn get_group_id(&self, input: &str) -> anyhow::Result<String> {
        let bin_path = self.ensure_compiled()?;

        let mut child = Command::new(&bin_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .with_context(|| format!("failed to execute input grouper: {:?}", bin_path))?;

        {
            let stdin = child.stdin.as_mut().context("failed to open child stdin")?;
            stdin
                .write_all(input.as_bytes())
                .context("failed to write input to child stdin")?;
        }

        let output = child
            .wait_with_output()
            .context("failed to wait for child process")?;

        if !output.status.success() {
            return Err(anyhow!(
                "input grouper execution failed\nstderr:\n{}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let group = String::from_utf8(output.stdout).context("stdout was not valid UTF-8")?;
        let group = group.trim().to_string();
        if group.is_empty() {
            return Err(anyhow!("input grouper returned an empty group"));
        }

        Ok(group)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_fn_grouper_returns_group_by_input_values() {
        let grouper = InputFnGrouper::new(
            r#"
use proconio::input;
input! { n: usize, m: usize }
if n < 10 && m < 10 {
    "small".to_string()
} else {
    "large".to_string()
}
"#
            .to_string(),
        );

        assert_eq!(grouper.get_group_id("1 2\n").unwrap(), "small");
        assert_eq!(grouper.get_group_id("10 2\n").unwrap(), "large");
    }

    #[test]
    fn test_input_fn_grouper_allows_return_statement() {
        let grouper = InputFnGrouper::new(
            r#"
use proconio::input;
input! { n: usize }
if n == 0 {
    return "zero".to_string();
}
"nonzero".to_string()
"#
            .to_string(),
        );

        assert_eq!(grouper.get_group_id("0\n").unwrap(), "zero");
        assert_eq!(grouper.get_group_id("3\n").unwrap(), "nonzero");
    }

    #[test]
    fn test_input_fn_grouper_can_be_reused_multiple_times() {
        let grouper = InputFnGrouper::new(
            r#"
use proconio::input;
input! { n: usize }
format!("group-{}", n % 2)
"#
            .to_string(),
        );

        let groups = ["1\n", "2\n", "5\n"]
            .into_iter()
            .map(|input| grouper.get_group_id(input).unwrap())
            .collect::<Vec<_>>();

        assert_eq!(groups, vec!["group-1", "group-0", "group-1"]);
    }
}
