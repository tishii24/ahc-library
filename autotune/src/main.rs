mod client;
mod generate_impl;
mod input;
mod model;
mod optuna;
mod optuna_param;
mod parser;

use std::{fs, path::PathBuf};

use crate::{
    client::RunnerClient,
    input::generator::ToolInputGenerator,
    model::{AutotuneConfig, OptunaParameterConfig},
};
use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long = "config_path")]
    config_path: PathBuf,
    #[arg(long = "root_dir", default_value = ".")]
    root_dir: PathBuf,
    #[arg(long = "work_dir", default_value = ".")]
    work_dir: PathBuf,
    #[arg(long = "generate_only", default_value_t = false)]
    generate_only: bool,
}

fn run_pahcer_optimizer<R: RunnerClient>(
    runner: R,
    pahcer_configs: &Vec<PathBuf>,
    optuna_params: &Vec<OptunaParameterConfig>,
) {
    todo!()
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let config_str = fs::read_to_string(&args.config_path)?;
    let config: AutotuneConfig = serde_yaml::from_str(&config_str)?;

    // 1. input-grouperでVec<InputGroup>を作成する
    let input_generator =
        ToolInputGenerator::new(args.root_dir.join("tools"), args.work_dir.join("in"));
    let input_groups = vec![todo!()];

    for group in input_groups {
        // 2. keyからpahcer-config.tomlと入力フォルダを生成する
        todo!()

        // 3. runnerにpahcer-config.tomlを渡して最適なパラメータを求める
    }

    // 4. TODO: 出力する

    Ok(())
}
