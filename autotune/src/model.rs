use std::path::PathBuf;

use serde::Deserialize;

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Minimize,
    Maximize,
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ScoreType {
    Absolute,
    Relative,
    Log10,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Settings {
    pub storage_path: String,
    pub direction: Direction,
    pub score_type: ScoreType,
}

#[derive(Deserialize, Debug, Clone)]
pub struct PrunerConfig {
    pub threshold: f64,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum OptunaParameterConfig {
    Int {
        name: String,
        min: i64,
        max: i64,
        default: i64,
    },
    Float {
        name: String,
        min: f64,
        max: f64,
        default: f64,
    },
}

#[derive(Deserialize, Debug, Clone)]
pub struct OptunaConfig {
    pub settings: Settings,
    pub pruner: PrunerConfig,
    pub params: Vec<OptunaParameterConfig>,
    #[serde(default)]
    pub ignore_params: Vec<String>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ParserConfig {
    Constant { index: usize },
}

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum InputParameterConfig {
    Int {
        name: String,
        parser: ParserConfig,
        partitions: Vec<i64>,
    },
    Float {
        name: String,
        parser: ParserConfig,
        partitions: Vec<f64>,
    },
    Categorical {
        name: String,
        parser: ParserConfig,
        categories: Vec<String>,
    },
}

#[derive(Deserialize, Debug, Clone)]
pub struct AutotuneConfig {
    pub basedir: PathBuf,
    pub optuna_config_path: PathBuf,
    pub timeout: u64,
    pub num_total_seed: u64,
    pub max_num_per_group: usize,
    pub input_params: Vec<InputParameterConfig>,
    pub optuna: OptunaConfig,
}
