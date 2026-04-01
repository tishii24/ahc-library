use std::{
    path::PathBuf,
    process::{Command, Stdio},
};

use tracing::info;

use crate::external::{optuna::OptunaClient, pahcer::PahcerConfig};

pub struct OptimizeRequest {
    study_prefix: String,
    group_id: String,
    pahcer_config: PahcerConfig,
    optuna_config_path: PathBuf,
    optuna_storage_path: PathBuf,
}
impl OptimizeRequest {
    pub fn new(
        group_id: String,
        study_prefix: String,
        pahcer_config: PahcerConfig,
        optuna_config_path: PathBuf,
        optuna_storage_path: PathBuf,
    ) -> Self {
        Self {
            group_id,
            study_prefix,
            pahcer_config,
            optuna_config_path,
            optuna_storage_path,
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

pub struct PahcerOptunaOptimizer<O: OptunaClient> {
    optuna_client: O,
}

impl<O: OptunaClient> PahcerOptunaOptimizer<O> {
    pub fn new(optuna_client: O) -> Self {
        Self { optuna_client }
    }
}

impl<O: OptunaClient> PahcerOptimizer for PahcerOptunaOptimizer<O> {
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

        let args = &[
            "--study_name",
            &study_name,
            "--optuna_config_path",
            request.optuna_config_path.to_str().unwrap(),
            "--pahcer_config_path",
            request.pahcer_config.config_path.to_str().unwrap(),
        ];
        info!("pahcer-optuna args: {:?}", args);
        Command::new("pahcer-optuna").args(args).status()?;

        let best_params = self
            .optuna_client
            .get_best_params(&study_name, &request.optuna_storage_path)?;
        Ok(OptimizeResult { best_params })
    }
}
