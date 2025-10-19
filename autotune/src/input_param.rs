use std::{collections::HashMap, fmt::Display, fs, path::PathBuf, process::Command, str::FromStr};

use anyhow::Result;

use crate::{
    model::{InputParameterConfig, ParserConfig},
    parser::{ConstantParser, InputParser},
};

impl InputParameterConfig {
    fn to_spec(self) -> Box<dyn InputParameterSpec> {
        match self {
            InputParameterConfig::Int {
                name,
                parser,
                partitions,
            } => match parser {
                ParserConfig::Constant { index } => Box::new(NumericalInputParameterSpec {
                    name,
                    parser: ConstantParser::new(index),
                    partitions,
                }),
            },
            InputParameterConfig::Float {
                name,
                parser,
                partitions,
            } => match parser {
                ParserConfig::Constant { index } => Box::new(NumericalInputParameterSpec {
                    name,
                    parser: ConstantParser::new(index),
                    partitions,
                }),
            },
            InputParameterConfig::Categorical {
                name,
                parser,
                categories,
            } => match parser {
                ParserConfig::Constant { index } => Box::new(CategoricalInputParameter {
                    name,
                    parser: ConstantParser::new(index),
                    categories,
                }),
            },
        }
    }
}

pub trait InputParameterSpec {
    fn get_partition_name(&self, input_content: &String) -> Option<String>;
}

pub struct NumericalInputParameterSpec<P, T>
where
    P: InputParser<T>,
    T: FromStr + PartialOrd + Display,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    pub name: String,
    pub parser: P,
    pub partitions: Vec<T>,
}

impl<P, T> InputParameterSpec for NumericalInputParameterSpec<P, T>
where
    P: InputParser<T>,
    T: FromStr + PartialOrd + Display,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    fn get_partition_name(&self, input_content: &String) -> Option<String> {
        let value = self.parser.parse(input_content).ok()?;
        let index = self
            .partitions
            .windows(2)
            .position(|w| w[0] <= value && value < w[1])?;
        Some(format!(
            "{}={}-{}",
            self.name,
            self.partitions[index],
            self.partitions[index + 1]
        ))
    }
}

pub struct CategoricalInputParameter<P>
where
    P: InputParser<String>,
{
    pub name: String,
    pub parser: P,
    pub categories: Vec<String>,
}

impl<P> InputParameterSpec for CategoricalInputParameter<P>
where
    P: InputParser<String>,
{
    fn get_partition_name(&self, input_content: &String) -> Option<String> {
        let value = self.parser.parse(input_content).ok()?;
        let index = self.categories.iter().position(|c| c == &value)?;
        Some(format!("{}={}", self.name, self.categories[index]))
    }
}

pub trait InputGenerator {
    fn generate_inputs(&self, seeds: &Vec<u64>) -> Result<Vec<String>, anyhow::Error>;
}

#[derive(Clone)]
pub struct ToolInputGenerator {
    tool_path: PathBuf,
}

impl ToolInputGenerator {
    pub fn new(tool_path: PathBuf) -> Self {
        Self { tool_path }
    }
}

impl InputGenerator for ToolInputGenerator {
    fn generate_inputs(&self, seeds: &Vec<u64>) -> Result<Vec<String>, anyhow::Error> {
        fs::write(
            self.tool_path.join("seeds.txt"),
            seeds
                .iter()
                .map(|seed| seed.to_string())
                .collect::<Vec<_>>()
                .join("\n"),
        )?;

        Command::new("cargo")
            .args(["run", "--release", "--bin", "gen", "seeds.txt"])
            .current_dir(&self.tool_path)
            .status()?;

        let input_files: Vec<String> = (0..seeds.len() as u64)
            .map(|i| {
                let input_file = self.tool_path.join("in").join(format!("{:04}.txt", i));
                fs::read_to_string(input_file)
                    .map_err(|e| anyhow::anyhow!("failed to read input file: {}", e))
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(input_files)
    }
}

pub struct InputGroupBuilder<G>
where
    G: InputGenerator,
{
    generator: G,
    param_specs: Vec<Box<dyn InputParameterSpec>>,
}

impl<G> InputGroupBuilder<G>
where
    G: InputGenerator,
{
    pub fn new(params: Vec<InputParameterConfig>, generator: G) -> Self {
        let param_specs = params.into_iter().map(|p| p.to_spec()).collect();
        Self {
            param_specs,
            generator,
        }
    }

    pub fn get_input_group_name(&self, input_content: &String) -> Option<String> {
        let mut names = vec![];
        for param in &self.param_specs {
            let name = param.get_partition_name(input_content)?;
            names.push(name);
        }
        Some(names.join("_"))
    }

    pub fn generate_input_group_seeds(
        &self,
        max_num_per_group: usize,
        total_seed: u64,
    ) -> HashMap<String, Vec<u64>> {
        const CHUNK_SIZE: u64 = 100;

        let mut input_group_seeds: HashMap<String, Vec<u64>> = HashMap::new();

        for seed_start in (0..total_seed).step_by(CHUNK_SIZE as usize) {
            let case_num = CHUNK_SIZE.min(total_seed - seed_start);
            let inputs = self
                .generator
                .generate_inputs(&(seed_start..seed_start + case_num).collect())
                .unwrap();
            for (i, input) in inputs.iter().enumerate() {
                let group_name = self.get_input_group_name(input).unwrap();
                let seed = seed_start + i as u64;

                let seeds = input_group_seeds.entry(group_name).or_insert(vec![]);
                if seeds.len() < max_num_per_group {
                    seeds.push(seed);
                }
            }
        }

        input_group_seeds
    }
}

#[test]
fn test_numerical_input_parameter_spec() {
    let parser = ConstantParser::new(0);
    let spec = NumericalInputParameterSpec {
        name: "N".to_string(),
        parser,
        partitions: vec![0, 10, 20, 30],
    };

    let input_content = "15".to_string();
    let partition_name = spec.get_partition_name(&input_content).unwrap();
    assert_eq!(partition_name, "N=10-20");

    let input_content = "5".to_string();
    let partition_name = spec.get_partition_name(&input_content).unwrap();
    assert_eq!(partition_name, "N=0-10");

    let input_content = "25".to_string();
    let partition_name = spec.get_partition_name(&input_content).unwrap();
    assert_eq!(partition_name, "N=20-30");

    let input_content = "30".to_string();
    let partition_name = spec.get_partition_name(&input_content);
    assert!(partition_name.is_none());
}

#[test]
fn test_categorical_input_parameter_spec() {
    let parser = ConstantParser::new(0);
    let spec = CategoricalInputParameter {
        name: "C".to_string(),
        parser,
        categories: vec!["0".to_string(), "1".to_string(), "2".to_string()],
    };

    let input_content = "1".to_string();
    let partition_name = spec.get_partition_name(&input_content).unwrap();
    assert_eq!(partition_name, "C=1");

    let input_content = "3".to_string();
    let partition_name = spec.get_partition_name(&input_content);
    assert!(partition_name.is_none());
}

#[test]
fn test_input_group_builder() {
    struct MockInputGenerator;
    impl InputGenerator for MockInputGenerator {
        fn generate_inputs(&self, seeds: &Vec<u64>) -> Result<Vec<String>, anyhow::Error> {
            let inputs: Vec<String> = seeds
                .iter()
                .map(|seed| format!("{} {}", seed, seed))
                .collect();
            Ok(inputs)
        }
    }

    let input_params = vec![
        InputParameterConfig::Int {
            name: "N".to_string(),
            parser: ParserConfig::Constant { index: 0 },
            partitions: vec![0, 1, 10],
        },
        InputParameterConfig::Int {
            name: "M".to_string(),
            parser: ParserConfig::Constant { index: 1 },
            partitions: vec![0, 2, 10],
        },
    ];

    let input_generator = MockInputGenerator;
    let input_group_builder = InputGroupBuilder::new(input_params, input_generator);
    let max_num_per_group = 5;
    let total_seed = 10;

    let input_group_seeds =
        input_group_builder.generate_input_group_seeds(max_num_per_group, total_seed);

    assert_eq!(input_group_seeds.len(), 3);
    assert_eq!(input_group_seeds["N=0-1_M=0-2"], vec![0]);
    assert_eq!(input_group_seeds["N=1-10_M=0-2"], vec![1]);
    assert_eq!(input_group_seeds["N=1-10_M=2-10"], vec![2, 3, 4, 5, 6]);
}
