use std::{
    fs,
    path::PathBuf,
    process::{Command, Stdio},
};

use anyhow::Result;

pub trait InputGenerator {
    fn generate_inputs(&self, seeds: &Vec<u64>) -> Result<Vec<String>, anyhow::Error>;
}

pub struct ToolInputGenerator {
    tool_path: PathBuf,
    in_dir: PathBuf,
}

impl ToolInputGenerator {
    pub fn new(tool_path: PathBuf, in_dir: PathBuf) -> Self {
        Self { tool_path, in_dir }
    }
}

impl InputGenerator for ToolInputGenerator {
    fn generate_inputs(&self, seeds: &Vec<u64>) -> Result<Vec<String>, anyhow::Error> {
        const SEEDS_FILE: &str = "seeds.txt";

        fs::create_dir_all(&self.in_dir)?;
        fs::write(
            self.in_dir.join(SEEDS_FILE),
            seeds
                .iter()
                .map(|seed| seed.to_string())
                .collect::<Vec<_>>()
                .join("\n"),
        )?;

        Command::new("cargo")
            .env("RUSTFLAGS", "-Awarnings")
            .args([
                "run",
                "-q",
                "--manifest-path",
                self.tool_path.join("Cargo.toml").to_str().unwrap(),
                "--release",
                "--bin",
                "gen",
                self.in_dir.join(SEEDS_FILE).to_str().unwrap(),
                "--dir",
                self.in_dir.to_str().unwrap(),
            ])
            .stdout(Stdio::null())
            .status()?;

        let input_files: Vec<String> = (0..seeds.len() as u64)
            .map(|i| {
                let input_file = self.in_dir.join(format!("{:04}.txt", i));
                fs::read_to_string(&input_file).map_err(|e| {
                    anyhow::anyhow!(
                        "failed to read input file: {:?}",
                        format!("{:?}: {:?}", input_file, e)
                    )
                })
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(input_files)
    }
}
