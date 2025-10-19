mod backend;
mod generate_impl;
mod input_param;
mod model;
mod parser;

use std::{fs, path::PathBuf, process::Command};

use crate::{
    input_param::{InputGenerator, InputGroupBuilder, ToolInputGenerator},
    model::{AutotuneConfig, ScoreType},
};
use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long = "config")]
    config_path: PathBuf,
}

const STUDY_NAME: &str = "autotune";

fn optimize(work_dir: &PathBuf, config: &AutotuneConfig) -> Result<()> {
    if config.optuna.settings.score_type == ScoreType::Relative {
        println!("initial run for calculation of relative score...");
        Command::new("pahcer")
            .arg("run")
            .current_dir(work_dir)
            .status()?;
    }

    Command::new("pahcer-optuna")
        .args([
            "--study_name",
            STUDY_NAME,
            "--config_path",
            config.optuna_config_path.to_str().unwrap(),
            "--timeout",
            &config.timeout.to_string(),
        ])
        .current_dir(work_dir)
        .status()?;

    Ok(())
}

fn run_autotune<G>(config: &AutotuneConfig, input_generator: G) -> Result<()>
where
    G: InputGenerator + Clone,
{
    fs::create_dir_all(&config.basedir)?;

    let input_group_builder =
        InputGroupBuilder::new(config.input_params.clone(), input_generator.clone());

    let group_seeds = input_group_builder
        .generate_input_group_seeds(config.max_num_per_group, config.num_total_seed);

    for (group_name, seeds) in group_seeds {
        let input_contents = input_generator.generate_inputs(&seeds)?;
        let work_dir = config.basedir.join(&group_name);
        if work_dir.exists() {
            println!("removing existing directory: {:?}", work_dir);
            fs::remove_dir_all(&work_dir)?;
        }
        fs::create_dir_all(&work_dir)?;
        fs::write(work_dir.join(".gitignore"), "*\n")?;

        for (i, input_content) in input_contents.iter().enumerate() {
            fs::write(
                work_dir.join("in").join(format!("{:04}.txt", i)),
                input_content,
            )?;
        }

        optimize(&work_dir, &config)?;
    }

    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();
    let config_str = fs::read_to_string(&args.config_path)?;
    let config: AutotuneConfig = serde_yaml::from_str(&config_str)?;

    let input_generator = ToolInputGenerator::new(config.basedir.clone());
    run_autotune(&config, input_generator)
}
