use std::path::PathBuf;

use crate::model::OptunaParameterConfig;

pub struct OptimizeRequest {
    pahcer_config_file: PathBuf,
    optuna_params: Vec<OptunaParameterConfig>,
}

pub struct OptimizeResult;

/// pahcerを利用して最適なパラメータを求めるためのtrait
/// `pahcer_config_file`の条件もとで、最適なパラメータを出力する
pub trait PahcerOptimizer {
    fn run(&self, request: OptimizeRequest) -> anyhow::Result<OptimizeResult>;
}

pub struct OptunaOptimizer;

impl PahcerOptimizer for OptunaOptimizer {
    fn run(&self, request: OptimizeRequest) -> anyhow::Result<OptimizeResult> {
        todo!()
    }
}
