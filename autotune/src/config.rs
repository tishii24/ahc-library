use std::path::PathBuf;

use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct OptunaSettings {
    pub storage_path: PathBuf,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum OptunaParameterConfig {
    Int {
        name: String,
        rust_type: String,
        default: i64,
    },
    Float {
        name: String,
        rust_type: String,
        default: f64,
    },
}

#[derive(Deserialize, Debug, Clone)]
pub struct OptunaConfig {
    pub params: Vec<OptunaParameterConfig>,
    pub settings: OptunaSettings,
}

#[derive(Deserialize, Debug, Clone)]
pub struct AutotuneConfig {
    #[serde(default = "default_work_dir")]
    pub work_dir: PathBuf,
    #[serde(default = "default_tools_dir")]
    pub tools_dir: PathBuf,
    #[serde(default = "default_base_pahcer_file")]
    pub base_pahcer_file: PathBuf,
    #[serde(default = "default_optuna_config_path")]
    pub optuna_config_path: PathBuf,
    #[serde(default = "default_case_num_per_group")]
    pub case_num_per_group: usize,
    pub input_fn: String,
}

fn default_work_dir() -> PathBuf {
    PathBuf::from("autotune")
}
fn default_tools_dir() -> PathBuf {
    PathBuf::from("tools")
}
fn default_base_pahcer_file() -> PathBuf {
    PathBuf::from("pahcer_config.toml")
}
fn default_optuna_config_path() -> PathBuf {
    PathBuf::from("optuna_config.yaml")
}
fn default_case_num_per_group() -> usize {
    100
}
