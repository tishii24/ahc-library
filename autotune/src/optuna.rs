use std::{
    path::PathBuf,
    process::{Command, Stdio},
};

use anyhow::Result;

use crate::pahcer::config::PahcerConfig;

pub struct OptimizeRequest {
    storage_path: PathBuf,
    study_prefix: String,
    group_id: String,
    pahcer_config: PahcerConfig,
    optuna_config_path: PathBuf,
}
impl OptimizeRequest {
    pub fn new(
        storage_path: PathBuf,
        group_id: String,
        study_prefix: String,
        pahcer_config: PahcerConfig,
        optuna_config_path: PathBuf,
    ) -> Self {
        Self {
            storage_path,
            group_id,
            study_prefix,
            pahcer_config,
            optuna_config_path,
        }
    }
}

pub struct OptimizeResult {
    pub best_params: serde_json::Map<String, serde_json::Value>,
}

/// pahcerを利用して最適なパラメータを求めるためのtrait
/// `pahcer_config_file`の条件もとで、最適なパラメータを出力する
pub trait PahcerOptimizer {
    fn run(&self, request: OptimizeRequest) -> anyhow::Result<OptimizeResult>;
}

pub struct PahcerOptunaOptimizer;

impl PahcerOptimizer for PahcerOptunaOptimizer {
    fn run(&self, request: OptimizeRequest) -> anyhow::Result<OptimizeResult> {
        let study_name = format!("{}_{}", request.study_prefix, request.group_id);

        Command::new("pahcer")
            .arg("run")
            .args(&[
                "--setting-file",
                request.pahcer_config.config_path.to_str().unwrap(),
            ])
            .stdout(Stdio::null())
            .status()?;

        Command::new("pahcer-optuna")
            .args(&[
                "--study_name",
                &study_name,
                "--config_path",
                request.optuna_config_path.to_str().unwrap(),
                "--storage_path",
                request.storage_path.to_str().unwrap(),
            ])
            .status()?;

        let best_params = get_best_params(&study_name, &request.storage_path)?;
        Ok(OptimizeResult { best_params })
    }
}

fn get_best_params(
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
