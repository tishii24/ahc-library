mod config;
mod formatter;
mod input;
mod optuna;
mod pahcer;

use crate::{
    config::{AutotuneConfig, OptunaConfig},
    formatter::{ParamFormatter, ParamsImplFormatter},
    input::{
        builder::InputBuilder,
        generator::{InputGenerator, ToolInputGenerator},
        grouper::InputFnGrouper,
    },
    optuna::{OptimizeRequest, PahcerOptimizer, PahcerOptunaOptimizer},
    pahcer::config::PahcerConfig,
};
use clap::Parser;
use std::{fs, path::PathBuf};
use tracing::info;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long = "config_path")]
    config_path: PathBuf,
    #[arg(long = "optuna_study_prefix")]
    optuna_study_prefix: String,
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    let config_str = fs::read_to_string(&args.config_path)?;
    let config: AutotuneConfig = serde_yaml::from_str(&config_str)?;
    let optuna_config_str = fs::read_to_string(&config.optuna_config_path)?;
    let optuna_config: OptunaConfig = serde_yaml::from_str(&optuna_config_str)?;
    let base_pahcer_toml = fs::read_to_string(&config.base_pahcer_file)?;

    let input_generator =
        ToolInputGenerator::new(config.tools_dir.clone(), config.work_dir.join("in"));
    let input_grouper = InputFnGrouper::new(config.input_fn.clone());
    let input_builder = InputBuilder::new(input_generator, input_grouper);

    info!("Generating input files...");
    const TRIAL_COUNT: u64 = 1_000;
    let mut input_groups = input_builder
        .build_inputs(config.case_num_per_group, TRIAL_COUNT)?
        .into_iter()
        .collect::<Vec<_>>();
    input_groups.sort_by_cached_key(|(group_id, _)| group_id.clone());

    let mut results: Vec<(String, serde_json::Map<String, serde_json::Value>)> =
        Vec::with_capacity(input_groups.len());

    for (group_id, seeds) in input_groups {
        info!("Optimizing for group: {}, seeds: {:?}", group_id, seeds);
        let pahcer_config = PahcerConfig::new(&config.work_dir, &group_id);
        pahcer_config.build_all(&base_pahcer_toml)?;

        let input_generator =
            ToolInputGenerator::new(config.tools_dir.clone(), pahcer_config.stdin_dir.clone());
        let _ = input_generator.generate_inputs(&seeds);

        let optimizer = PahcerOptunaOptimizer;
        let request = OptimizeRequest::new(
            group_id.clone(),
            args.optuna_study_prefix.clone(),
            pahcer_config,
            config.optuna_config_path.clone(),
            optuna_config.settings.storage_path.clone(),
        );
        let result = optimizer.run(request)?;
        results.push((group_id, result.best_params));
    }

    let formatter = ParamsImplFormatter;
    let output = formatter.format(&optuna_config.params, &results);
    println!("{}", output);

    Ok(())
}
