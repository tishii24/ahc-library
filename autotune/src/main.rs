mod client;
mod generate_impl;
mod input_param;
mod model;
mod optuna_param;
mod parser;

use std::{fs, path::PathBuf};

use crate::{
    client::{PahcerOptunaClient, RunnerClient},
    generate_impl::generate_param_impl,
    input_param::{InputGenerator, InputGroup, InputGroupBuilder, ToolInputGenerator},
    model::{AutotuneConfig, InputParameterConfig},
};
use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long = "config_path")]
    config_path: PathBuf,
    #[arg(long = "root_dir", default_value = ".")]
    root_dir: PathBuf,
    #[arg(long = "generate_only", default_value_t = false)]
    generate_only: bool,
}

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
            let file_name = path.file_name().unwrap();
            if ignores
                .as_ref()
                .map_or(false, |i| i.contains(&file_name.into()))
            {
                continue;
            }

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

    let in_dir = work_dir.join("tools").join("in");
    fs::create_dir_all(&in_dir)?;

    copy_dir(
        root_dir,
        work_dir,
        &Some(vec![base_dir.clone().file_name().unwrap().into()]),
    )?;

    for (i, input_content) in input_contents.iter().enumerate() {
        fs::write(in_dir.join(format!("{:04}.txt", i)), input_content)?;
    }

    Ok(())
}

#[test]
fn test_setup_group_directory() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let temp_dir = temp_dir.path();
    let root_dir = temp_dir.join("root");
    let base_dir = temp_dir.join("root").join(".autotune");
    let work_dir = temp_dir.join("root").join(".autotune").join("group1");

    fs::create_dir_all(&root_dir)?;
    fs::write(root_dir.join("file1.txt"), "root file")?;

    let input_contents = vec!["input1".to_string(), "input2".to_string()];

    setup_group_directory(&work_dir, &base_dir, &root_dir, &input_contents)?;

    assert!(work_dir.join("file1.txt").exists());
    assert!(!work_dir.join(".autotune").exists());
    assert!(work_dir.join("in").join("0000.txt").exists());
    assert!(work_dir.join("in").join("0001.txt").exists());
    assert!(!work_dir.join("in").join("0002.txt").exists());

    Ok(())
}

fn prepare_input_groups<G>(
    root_dir: &PathBuf,
    base_dir: &PathBuf,
    input_params: &Vec<InputParameterConfig>,
    case_num_per_group: usize,
    num_total_seed: u64,
    input_generator: G,
) -> Result<Vec<InputGroup>>
where
    G: InputGenerator,
{
    fs::create_dir_all(&base_dir)?;
    fs::write(base_dir.join(".gitignore"), "*\n")?;

    let input_group_builder = InputGroupBuilder::new(input_params.clone(), input_generator);

    let group_seeds =
        input_group_builder.generate_input_group_seeds(case_num_per_group, num_total_seed)?;

    for (group, seeds) in group_seeds.iter() {
        let work_dir = group.get_work_dir(&base_dir);
        if work_dir.exists() {
            println!("existing directory, skip generating: {:?}", work_dir);
            continue;
        }

        let input_contents = input_group_builder.generator.generate_inputs(&seeds)?;
        if input_contents.len() < case_num_per_group {
            return Err(anyhow::anyhow!(
                "insufficient inputs for group {}",
                group.key.0
            ));
        }

        setup_group_directory(&work_dir, &base_dir, &root_dir, &input_contents)?;
    }

    Ok(group_seeds
        .into_keys()
        .map(|group| group)
        .collect::<Vec<_>>())
}

fn main() -> Result<()> {
    const STUDY_NAME: &str = "autotune";

    let args = Args::parse();
    let config_str = fs::read_to_string(&args.config_path)?;
    let config: AutotuneConfig = serde_yaml::from_str(&config_str)?;

    let input_generator = ToolInputGenerator::new(args.root_dir.join("tools"));
    let client = PahcerOptunaClient;

    let input_groups = prepare_input_groups(
        &args.root_dir,
        &config.base_dir,
        &config.input_params,
        config.case_num_per_group,
        config.num_total_seed,
        input_generator,
    )?;

    if !args.generate_only {
        for group in input_groups.iter() {
            client.run_optuna(
                &group.get_work_dir(&config.base_dir),
                STUDY_NAME,
                &config.optuna_config_path,
                config.timeout,
            )?;
        }
    }

    let best_params = input_groups
        .into_iter()
        .filter_map(|group| {
            match client.get_best_params(
                &group.get_work_dir(&config.base_dir),
                STUDY_NAME,
                &config.optuna.settings.storage_path,
            ) {
                Ok(params) => Ok((group, params)),
                Err(err) => {
                    eprintln!(
                        "Failed to get best params for group {}: {}",
                        group.key.0, err
                    );
                    Err(err)
                }
            }
            .ok()
        })
        .collect::<Vec<_>>();
    let param_specs = config
        .input_params
        .iter()
        .cloned()
        .map(|p| p.to_spec())
        .collect();
    let param_impl = generate_param_impl(&best_params, &param_specs, &config.optuna.params);
    println!("{}", param_impl);

    Ok(())
}
