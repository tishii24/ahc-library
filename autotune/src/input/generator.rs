use std::{fs, path::PathBuf, process::Command};

use anyhow::Result;

pub trait InputGenerator {
    fn generate_inputs(&self, seeds: &Vec<u64>) -> Result<Vec<String>, anyhow::Error>;
}

pub struct ToolInputGenerator {
    tool_path: PathBuf,
    temp_in_dir: PathBuf,
}

impl ToolInputGenerator {
    pub fn new(tool_path: PathBuf, temp_in_dir: PathBuf) -> Self {
        Self {
            tool_path,
            temp_in_dir,
        }
    }
}

impl InputGenerator for ToolInputGenerator {
    fn generate_inputs(&self, seeds: &Vec<u64>) -> Result<Vec<String>, anyhow::Error> {
        const SEEDS_FILE: &str = "seeds.txt";

        fs::create_dir_all(&self.temp_in_dir)?;
        fs::write(
            self.temp_in_dir.join(SEEDS_FILE),
            seeds
                .iter()
                .map(|seed| seed.to_string())
                .collect::<Vec<_>>()
                .join("\n"),
        )?;

        Command::new("cargo")
            .args([
                "run",
                "--release",
                "--bin",
                "gen",
                self.temp_in_dir.join(SEEDS_FILE).to_str().unwrap(),
                "--dir",
                self.temp_in_dir.to_str().unwrap(),
            ])
            .current_dir(&self.tool_path)
            .status()?;

        let input_files: Vec<String> = (0..seeds.len() as u64)
            .map(|i| {
                let input_file = self.temp_in_dir.join(format!("{:04}.txt", i));
                fs::read_to_string(&input_file).map_err(|e| {
                    anyhow::anyhow!(
                        "failed to read input file: {:?}",
                        format!("{:?}: {:?}", input_file, e)
                    )
                })
            })
            .collect::<Result<Vec<_>>>()?;

        fs::remove_dir_all(&self.temp_in_dir)?;

        Ok(input_files)
    }
}
