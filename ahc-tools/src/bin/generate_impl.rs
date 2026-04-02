use ahc_tools::{
    common::config::PahcerOptunaConfig,
    common::formatter::{ParamFormatter, ParamsImplFormatter},
    external::optuna::{OptunaCliClient, OptunaClient},
};
use anyhow::Context;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long = "optuna_config_path")]
    optuna_config_path: PathBuf,
    #[arg(long = "study_name", default_value = None)]
    study_name: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let config_str =
        std::fs::read_to_string(&args.optuna_config_path).context("Failed to read config file")?;
    let optuna_config: PahcerOptunaConfig =
        serde_yaml::from_str(&config_str).context("Failed to parse config file")?;

    let optuna_client = OptunaCliClient;
    let best_params = if let Some(study_name) = &args.study_name {
        Some(
            optuna_client
                .get_best_params(study_name, &optuna_config.settings.storage_path)
                .context("Failed to get best params from Optuna")?,
        )
    } else {
        None
    };

    let formatter = ParamsImplFormatter;
    let output = formatter.format_single(&optuna_config.params, &best_params);
    println!("{}", output);

    Ok(())
}
