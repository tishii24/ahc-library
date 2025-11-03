use std::{collections::HashMap, path::PathBuf, process::Command};

use anyhow::Result;

pub trait RunnerClient {
    fn run(&self, work_dir: &PathBuf) -> Result<()>;
    fn run_optuna(
        &self,
        work_dir: &PathBuf,
        study_name: &str,
        config_path: &PathBuf,
        timeout: u64,
    ) -> Result<()>;
    fn get_best_params(
        &self,
        work_dir: &PathBuf,
        study_name: &str,
        storage_path: &PathBuf,
    ) -> Result<HashMap<String, String>>;
}

pub struct PahcerOptunaClient;

impl RunnerClient for PahcerOptunaClient {
    fn run(&self, work_dir: &PathBuf) -> Result<()> {
        std::process::Command::new("pahcer")
            .arg("run")
            .current_dir(work_dir)
            .status()?;
        Ok(())
    }

    fn run_optuna(
        &self,
        work_dir: &PathBuf,
        study_name: &str,
        config_path: &PathBuf,
        timeout: u64,
    ) -> Result<()> {
        // initial run for calculation of relative score
        self.run(work_dir)?;

        std::process::Command::new("pahcer-optuna")
            .args([
                "--study_name",
                study_name,
                "--config_path",
                config_path.to_str().unwrap(),
                "--timeout",
                &timeout.to_string(),
            ])
            .current_dir(work_dir)
            .status()?;
        Ok(())
    }

    fn get_best_params(
        &self,
        work_dir: &PathBuf,
        study_name: &str,
        storage_path: &PathBuf,
    ) -> Result<HashMap<String, String>> {
        let output = Command::new("optuna")
            .args([
                "best-trial",
                "--study-name",
                study_name,
                "--storage",
                storage_path.to_str().unwrap(),
                "-f",
                "json",
            ])
            .current_dir(work_dir)
            .output()?;

        if !output.status.success() {
            return Err(anyhow::anyhow!("Failed to get best parameters"));
        }

        let stdout = String::from_utf8(output.stdout)?;
        let json: serde_json::Map<String, serde_json::Value> = serde_json::from_str(&stdout)?;
        Ok(json
            .iter()
            .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
            .collect())
    }
}
