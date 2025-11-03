mod backend;
mod client;
mod generate_impl;
mod input_param;
mod model;
mod optuna_param;
mod parser;

use std::{fs, path::PathBuf};

use crate::{
    client::RunnerClient,
    generate_impl::generate_param_impl,
    input_param::{InputGenerator, InputGroup, InputGroupBuilder, ToolInputGenerator},
    model::AutotuneConfig,
};
use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long = "config_path")]
    config_path: PathBuf,
    #[arg(long = "root_dir", default_value = ".")]
    root_dir: PathBuf,
}

const STUDY_NAME: &str = "autotune";

fn setup_group_directory(
    work_dir: &PathBuf,
    base_dir: &PathBuf,
    root_dir: &PathBuf,
    input_contents: &Vec<String>,
) -> Result<()> {
    fn copy_dir(src: &PathBuf, dst: &PathBuf, ignores: &Option<Vec<PathBuf>>) -> Result<()> {
        fs::create_dir_all(dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let path = entry.path();
            if ignores.as_ref().map_or(false, |i| i.contains(&path)) {
                continue;
            }
            let file_name = path.file_name().unwrap();
            let dest_path = dst.join(file_name);
            if path.is_dir() {
                fs::create_dir_all(&dest_path)?;
                fs_extra::dir::copy(&path, &dst, &fs_extra::dir::CopyOptions::new())?;
            } else {
                fs::copy(&path, &dest_path)?;
            }
        }

        Ok(())
    }

    fs::create_dir_all(work_dir.join("in"))?;
    fs::write(work_dir.join(".gitignore"), "*\n")?;

    copy_dir(root_dir, work_dir, &Some(vec![base_dir.clone()]))?;

    for (i, input_content) in input_contents.iter().enumerate() {
        fs::write(
            work_dir.join("in").join(format!("{:04}.txt", i)),
            input_content,
        )?;
    }

    Ok(())
}

fn prepare_input_groups<G>(
    root_dir: &PathBuf,
    config: &AutotuneConfig,
    input_generator: G,
) -> Result<Vec<InputGroup>>
where
    G: InputGenerator,
{
    fs::create_dir_all(&config.base_dir)?;
    fs::write(config.base_dir.join(".gitignore"), "*\n")?;

    let input_group_builder = InputGroupBuilder::new(config.input_params.clone(), input_generator);

    let group_seeds = input_group_builder
        .generate_input_group_seeds(config.case_num_per_group, config.num_total_seed);

    for (group, seeds) in group_seeds.iter() {
        let work_dir = group.get_work_dir(&config.base_dir);
        if work_dir.exists() {
            println!("existing directory, skip generating: {:?}", work_dir);
            continue;
        }

        let input_contents = input_group_builder.generator.generate_inputs(&seeds)?;
        if input_contents.len() < config.case_num_per_group {
            return Err(anyhow::anyhow!(
                "insufficient inputs for group {}",
                group.key.0
            ));
        }

        setup_group_directory(&work_dir, &config.base_dir, &root_dir, &input_contents)?;
    }

    Ok(group_seeds
        .into_keys()
        .map(|group| group)
        .collect::<Vec<_>>())
}

fn main() -> Result<()> {
    let args = Args::parse();
    let config_str = fs::read_to_string(&args.config_path)?;
    let config: AutotuneConfig = serde_yaml::from_str(&config_str)?;

    let input_generator = ToolInputGenerator::new(config.base_dir.clone());
    let client = client::PahcerOptunaClient;

    let input_groups = prepare_input_groups(&args.root_dir, &config, input_generator)?;
    for group in input_groups.iter() {
        client.run_optuna(
            &group.get_work_dir(&config.base_dir),
            STUDY_NAME,
            &config.optuna_config_path,
            config.timeout,
        )?;
    }

    let best_params = input_groups
        .into_iter()
        .map(|group| {
            let params = client.get_best_params(
                &group.get_work_dir(&config.base_dir),
                STUDY_NAME,
                &config.optuna.settings.storage_path,
            )?;
            Ok((group, params))
        })
        .collect::<Result<Vec<_>>>()?;

    let param_specs = config
        .input_params
        .into_iter()
        .map(|p| p.to_spec())
        .collect();

    let param_impl = generate_param_impl(&best_params, &param_specs, &config.optuna.params);
    println!("{}", param_impl);

    Ok(())
}
