use std::{path::PathBuf, process::Command};

use anyhow::Result;
pub trait OptunaClient {
    fn get_best_params(
        &self,
        study_name: &str,
        storage_path: &PathBuf,
    ) -> Result<serde_json::Map<String, serde_json::Value>>;
}

pub struct OptunaCliClient;

impl OptunaClient for OptunaCliClient {
    fn get_best_params(
        &self,
        study_name: &str,
        storage_path: &PathBuf,
    ) -> Result<serde_json::Map<String, serde_json::Value>> {
        let storage_path = storage_path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid storage path"))?;
        let stdout = Command::new("optuna")
            .args(&[
                "best-trial",
                "--study-name",
                study_name,
                "--storage",
                storage_path,
                "-f",
                "json",
            ])
            .output()?;
        if !stdout.status.success() {
            anyhow::bail!(
                "Failed to get best params: {}",
                String::from_utf8_lossy(&stdout.stderr)
            );
        }
        let output = String::from_utf8(stdout.stdout)?;
        let json: serde_json::Map<String, serde_json::Value> = serde_json::from_str(&output)?;
        Ok(json
            .get("params")
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_default())
    }
}
