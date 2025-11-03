use std::{
    collections::HashMap, fmt::Display, fs, hash::Hash, path::PathBuf, process::Command,
    str::FromStr,
};

use anyhow::Result;

use crate::{
    model::{InputParameterConfig, ParserConfig},
    parser::{ConstantParser, InputParser},
};

impl InputParameterConfig {
    pub fn to_spec(self) -> Box<dyn InputParameterSpec> {
        match self {
            InputParameterConfig::Int {
                name,
                rust_type,
                parser,
                partitions,
            } => match parser {
                ParserConfig::Constant { index } => Box::new(NumericalInputParameterSpec {
                    name,
                    rust_type,
                    parser: ConstantParser::new(index),
                    partitions,
                }),
            },
            InputParameterConfig::Float {
                name,
                rust_type,
                parser,
                partitions,
            } => match parser {
                ParserConfig::Constant { index } => Box::new(NumericalInputParameterSpec {
                    name,
                    rust_type,
                    parser: ConstantParser::new(index),
                    partitions,
                }),
            },
            InputParameterConfig::Categorical {
                name,
                rust_type,
                parser,
                categories,
            } => match parser {
                ParserConfig::Constant { index } => Box::new(CategoricalInputParameterSpec {
                    name,
                    rust_type,
                    parser: ConstantParser::new(index),
                    categories,
                }),
            },
        }
    }
}

pub struct InputPartition {
    pub key: String,
    pub match_arm_impl: String,
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct InputGroupKey(pub String);

pub struct InputGroup {
    pub key: InputGroupKey,
    pub partitions: Vec<InputPartition>,
}

impl InputGroup {
    pub fn new(partitions: Vec<InputPartition>) -> Self {
        let key = partitions
            .iter()
            .map(|p| p.key.to_owned())
            .collect::<Vec<_>>()
            .join("_");
        Self {
            key: InputGroupKey(key),
            partitions,
        }
    }

    pub fn match_arm_impl(&self) -> String {
        self.partitions
            .iter()
            .map(|p| p.match_arm_impl.to_owned())
            .collect::<Vec<_>>()
            .join(", ")
    }

    pub fn get_work_dir(&self, base_dir: &PathBuf) -> PathBuf {
        base_dir.join(&self.key.0)
    }
}

impl Hash for InputGroup {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key.hash(state);
    }
}

impl PartialEq for InputGroup {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl Eq for InputGroup {}

pub trait InputParameterSpec {
    fn get_input_partition(&self, input_content: &String) -> Option<InputPartition>;
    fn get_def_impl(&self) -> String;
}

pub struct NumericalInputParameterSpec<P, T>
where
    P: InputParser<T>,
    T: FromStr + PartialOrd + Display,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    name: String,
    rust_type: String,
    parser: P,
    partitions: Vec<T>,
}

impl<P, T> InputParameterSpec for NumericalInputParameterSpec<P, T>
where
    P: InputParser<T>,
    T: FromStr + PartialOrd + Display,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    fn get_input_partition(&self, input_content: &String) -> Option<InputPartition> {
        let value = self.parser.parse(input_content).ok()?;
        let index = self
            .partitions
            .windows(2)
            .position(|w| w[0] <= value && value < w[1])?;

        let name = format!(
            "{}={}-{}",
            self.name,
            self.partitions[index],
            self.partitions[index + 1]
        );
        let match_arm_impl = format!(
            "(({})..({}))",
            self.partitions[index],
            self.partitions[index + 1]
        );
        Some(InputPartition {
            key: name,
            match_arm_impl,
        })
    }

    fn get_def_impl(&self) -> String {
        format!("{}: {}", self.name, self.rust_type)
    }
}

pub struct CategoricalInputParameterSpec<P>
where
    P: InputParser<String>,
{
    name: String,
    rust_type: String,
    parser: P,
    categories: Vec<String>,
}

impl<P> InputParameterSpec for CategoricalInputParameterSpec<P>
where
    P: InputParser<String>,
{
    fn get_input_partition(&self, input_content: &String) -> Option<InputPartition> {
        let value = self.parser.parse(input_content).ok()?;
        let index = self.categories.iter().position(|c| c == &value)?;
        let name = format!("{}={}", self.name, self.categories[index]);
        let match_arm_impl = format!("{}", self.categories[index]);

        Some(InputPartition {
            key: name,
            match_arm_impl,
        })
    }

    fn get_def_impl(&self) -> String {
        format!("{}: {}", self.name, self.rust_type)
    }
}

pub trait InputGenerator {
    fn generate_inputs(&self, seeds: &Vec<u64>) -> Result<Vec<String>, anyhow::Error>;
}

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
        const IN_DIR: &str = ".in_autotune";
        const SEEDS_FILE: &str = "seeds.txt";
        fs::create_dir_all(self.tool_path.join(IN_DIR))?;

        fs::write(
            self.tool_path.join(IN_DIR).join(SEEDS_FILE),
            seeds
                .iter()
                .map(|seed| seed.to_string())
                .collect::<Vec<_>>()
                .join("\n"),
        )?;

        Command::new("cargo")
            .args([
                "run",
                "--release",
                "--bin",
                "gen",
                SEEDS_FILE,
                "--dir",
                IN_DIR,
            ])
            .current_dir(&self.tool_path)
            .status()?;

        let input_files: Vec<String> = (0..seeds.len() as u64)
            .map(|i| {
                let input_file = self.tool_path.join(IN_DIR).join(format!("{:04}.txt", i));
                fs::read_to_string(input_file)
                    .map_err(|e| anyhow::anyhow!("failed to read input file: {}", e))
            })
            .collect::<Result<Vec<_>>>()?;

        fs::remove_dir_all(self.tool_path.join(IN_DIR))?;

        Ok(input_files)
    }
}

pub struct InputGroupBuilder<G>
where
    G: InputGenerator,
{
    pub generator: G,
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

    pub fn get_input_group(&self, input_content: &String) -> Option<InputGroup> {
        let mut partitions = vec![];
        for param in &self.param_specs {
            let partition = param.get_input_partition(input_content)?;
            partitions.push(partition);
        }
        Some(InputGroup::new(partitions))
    }

    pub fn generate_input_group_seeds(
        &self,
        case_num_per_group: usize,
        total_seed: u64,
    ) -> HashMap<InputGroup, Vec<u64>> {
        const CHUNK_SIZE: u64 = 100;

        let mut input_group_seeds: HashMap<InputGroup, Vec<u64>> = HashMap::new();

        for seed_start in (0..total_seed).step_by(CHUNK_SIZE as usize) {
            let case_num = CHUNK_SIZE.min(total_seed - seed_start);
            let inputs = self
                .generator
                .generate_inputs(&(seed_start..seed_start + case_num).collect())
                .unwrap();
            for (i, input) in inputs.iter().enumerate() {
                let input_group = self.get_input_group(input).unwrap();
                let seed = seed_start + i as u64;

                let seeds = input_group_seeds.entry(input_group).or_insert(vec![]);
                if seeds.len() < case_num_per_group {
                    seeds.push(seed);
                }
            }
        }

        input_group_seeds
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_numerical_input_parameter_spec() {
        let parser = ConstantParser::new(0);
        let spec = NumericalInputParameterSpec {
            name: "N".to_string(),
            rust_type: "usize".to_string(),
            parser,
            partitions: vec![0, 10, 20, 30],
        };

        let input_content = "15".to_string();
        let input_partition = spec.get_input_partition(&input_content).unwrap();
        assert_eq!(input_partition.key, "N=10-20");

        let input_content = "5".to_string();
        let input_partition = spec.get_input_partition(&input_content).unwrap();
        assert_eq!(input_partition.key, "N=0-10");

        let input_content = "25".to_string();
        let input_partition = spec.get_input_partition(&input_content).unwrap();
        assert_eq!(input_partition.key, "N=20-30");

        let input_content = "30".to_string();
        let input_partition = spec.get_input_partition(&input_content);
        assert!(input_partition.is_none());
    }

    #[test]
    fn test_categorical_input_parameter_spec() {
        let parser = ConstantParser::new(0);
        let spec = CategoricalInputParameterSpec {
            name: "C".to_string(),
            rust_type: "usize".to_string(),
            parser,
            categories: vec!["0".to_string(), "1".to_string(), "2".to_string()],
        };

        let input_content = "1".to_string();
        let input_partition = spec.get_input_partition(&input_content).unwrap();
        assert_eq!(input_partition.key, "C=1");

        let input_content = "3".to_string();
        let input_partition = spec.get_input_partition(&input_content);
        assert!(input_partition.is_none());
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
                rust_type: "usize".to_string(),
                parser: ParserConfig::Constant { index: 0 },
                partitions: vec![0, 1, 10],
            },
            InputParameterConfig::Int {
                name: "M".to_string(),
                rust_type: "usize".to_string(),
                parser: ParserConfig::Constant { index: 1 },
                partitions: vec![0, 2, 10],
            },
        ];

        let input_generator = MockInputGenerator;
        let input_group_builder = InputGroupBuilder::new(input_params, input_generator);
        let case_num_per_group = 5;
        let total_seed = 10;

        let input_group_seeds =
            input_group_builder.generate_input_group_seeds(case_num_per_group, total_seed);
        let input_group_seeds = input_group_seeds
            .into_iter()
            .map(|(k, v)| (k.key.0, v))
            .collect::<HashMap<_, _>>();
        assert_eq!(input_group_seeds.len(), 3);
        assert_eq!(input_group_seeds["N=0-1_M=0-2"], vec![0]);
        assert_eq!(input_group_seeds["N=1-10_M=0-2"], vec![1]);
        assert_eq!(input_group_seeds["N=1-10_M=2-10"], vec![2, 3, 4, 5, 6]);
    }
}
