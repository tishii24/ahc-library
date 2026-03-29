// use std::{collections::HashMap, fs, hash::Hash, path::PathBuf, process::Command};

// use anyhow::Result;

// use crate::{
//     model::{InputParameterConfig, ParserConfig},
//     parser::{ConstantParser, InputParser},
// };

// impl InputParameterConfig {
//     pub fn to_spec(self) -> InputParameterSpec {
//         match self {
//             InputParameterConfig::Int {
//                 name,
//                 rust_type,
//                 parser,
//                 partitions,
//             } => match parser {
//                 ParserConfig::Constant { index } => InputParameterSpec::Int {
//                     name,
//                     rust_type,
//                     index,
//                     partitions,
//                 },
//             },
//             InputParameterConfig::Float {
//                 name,
//                 rust_type,
//                 parser,
//                 partitions,
//             } => match parser {
//                 ParserConfig::Constant { index } => InputParameterSpec::Float {
//                     name,
//                     rust_type,
//                     index,
//                     partitions,
//                 },
//             },
//         }
//     }
// }

// #[derive(Clone, Debug)]
// pub struct InputPartition {
//     pub key: String,
//     pub match_arm_impl: String,
// }

// #[derive(Clone, Hash, PartialEq, Eq, Debug)]
// pub struct InputGroupKey(pub String);

// #[derive(Clone, Debug)]
// pub struct InputGroup {
//     pub key: InputGroupKey,
//     pub partitions: Vec<InputPartition>,
// }

// impl InputGroup {
//     pub fn new(partitions: Vec<InputPartition>) -> Self {
//         let key = partitions
//             .iter()
//             .map(|p| p.key.to_owned())
//             .collect::<Vec<_>>()
//             .join("_");
//         Self {
//             key: InputGroupKey(key),
//             partitions,
//         }
//     }

//     pub fn match_arm_impl(&self) -> String {
//         self.partitions
//             .iter()
//             .map(|p| p.match_arm_impl.to_owned())
//             .collect::<Vec<_>>()
//             .join(", ")
//     }
// }

// impl Hash for InputGroup {
//     fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
//         self.key.hash(state);
//     }
// }

// impl PartialEq for InputGroup {
//     fn eq(&self, other: &Self) -> bool {
//         self.key == other.key
//     }
// }

// impl Eq for InputGroup {}

// pub enum InputParameterSpec {
//     Int {
//         name: String,
//         rust_type: String,
//         index: usize,
//         partitions: Vec<i64>,
//     },
//     Float {
//         name: String,
//         rust_type: String,
//         index: usize,
//         partitions: Vec<f64>,
//     },
//     Categorical {
//         name: String,
//         rust_type: String,
//         index: usize,
//         categories: Vec<String>,
//     },
// }

// impl InputParameterSpec {
//     pub fn get_input_partition(&self, input_content: &String) -> Option<InputPartition> {
//         match self {
//             Self::Int {
//                 name,
//                 index,
//                 partitions,
//                 ..
//             } => {
//                 let value: i64 = ConstantParser::new(*index).parse(input_content).ok()?;
//                 let partition_index = partitions
//                     .windows(2)
//                     .position(|w| w[0] <= value && value < w[1])?;

//                 Some(InputPartition {
//                     key: format!(
//                         "{}={}-{}",
//                         name,
//                         partitions[partition_index],
//                         partitions[partition_index + 1]
//                     ),
//                     match_arm_impl: format!(
//                         "(({})..({}))",
//                         partitions[partition_index],
//                         partitions[partition_index + 1]
//                     ),
//                 })
//             }
//             Self::Float {
//                 name,
//                 index,
//                 partitions,
//                 ..
//             } => {
//                 let value: f64 = ConstantParser::new(*index).parse(input_content).ok()?;
//                 let partition_index = partitions
//                     .windows(2)
//                     .position(|w| w[0] <= value && value < w[1])?;

//                 Some(InputPartition {
//                     key: format!(
//                         "{}={}-{}",
//                         name,
//                         partitions[partition_index],
//                         partitions[partition_index + 1]
//                     ),
//                     match_arm_impl: format!(
//                         "(({})..({}))",
//                         partitions[partition_index],
//                         partitions[partition_index + 1]
//                     ),
//                 })
//             }
//             Self::Categorical {
//                 name,
//                 index,
//                 categories,
//                 ..
//             } => {
//                 let value: String = ConstantParser::new(*index).parse(input_content).ok()?;
//                 let category_index = categories.iter().position(|c| c == &value)?;

//                 Some(InputPartition {
//                     key: format!("{}={}", name, categories[category_index]),
//                     match_arm_impl: categories[category_index].clone(),
//                 })
//             }
//         }
//     }

//     pub fn name(&self) -> &str {
//         match self {
//             Self::Int { name, .. } => name,
//             Self::Float { name, .. } => name,
//             Self::Categorical { name, .. } => name,
//         }
//     }

//     pub fn rust_type(&self) -> &str {
//         match self {
//             Self::Int { rust_type, .. } => rust_type,
//             Self::Float { rust_type, .. } => rust_type,
//             Self::Categorical { rust_type, .. } => rust_type,
//         }
//     }
// }

// pub struct InputGroupBuilder<G>
// where
//     G: InputGenerator,
// {
//     pub generator: G,
//     param_specs: Vec<InputParameterSpec>,
// }

// impl<G> InputGroupBuilder<G>
// where
//     G: InputGenerator,
// {
//     pub fn new(params: Vec<InputParameterConfig>, generator: G) -> Self {
//         let param_specs = params.into_iter().map(|p| p.to_spec()).collect();
//         Self {
//             param_specs,
//             generator,
//         }
//     }

//     pub fn get_input_group(&self, input_content: &String) -> Option<InputGroup> {
//         let mut partitions = vec![];
//         for param in &self.param_specs {
//             let partition = param.get_input_partition(input_content)?;
//             partitions.push(partition);
//         }
//         Some(InputGroup::new(partitions))
//     }

//     pub fn generate_input_group_seeds(
//         &self,
//         case_num_per_group: usize,
//         total_seed: u64,
//     ) -> Result<HashMap<InputGroup, Vec<u64>>> {
//         const CHUNK_SIZE: u64 = 100;

//         let mut input_group_seeds: HashMap<InputGroup, Vec<u64>> = HashMap::new();

//         for seed_start in (0..total_seed).step_by(CHUNK_SIZE as usize) {
//             let case_num = CHUNK_SIZE.min(total_seed - seed_start);
//             let inputs = self
//                 .generator
//                 .generate_inputs(&(seed_start..seed_start + case_num).collect())?;
//             for (i, input) in inputs.iter().enumerate() {
//                 let Some(input_group) = self.get_input_group(input) else {
//                     continue;
//                 };
//                 let seed = seed_start + i as u64;

//                 let seeds = input_group_seeds.entry(input_group).or_insert(vec![]);
//                 if seeds.len() < case_num_per_group {
//                     seeds.push(seed);
//                 }
//             }
//         }

//         Ok(input_group_seeds)
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_numerical_input_parameter_spec() {
//         let spec = InputParameterSpec::Int {
//             name: "N".to_string(),
//             rust_type: "usize".to_string(),
//             index: 0,
//             partitions: vec![0, 10, 20, 30],
//         };

//         let input_content = "15".to_string();
//         let input_partition = spec.get_input_partition(&input_content).unwrap();
//         assert_eq!(input_partition.key, "N=10-20");

//         let input_content = "5".to_string();
//         let input_partition = spec.get_input_partition(&input_content).unwrap();
//         assert_eq!(input_partition.key, "N=0-10");

//         let input_content = "25".to_string();
//         let input_partition = spec.get_input_partition(&input_content).unwrap();
//         assert_eq!(input_partition.key, "N=20-30");

//         let input_content = "30".to_string();
//         let input_partition = spec.get_input_partition(&input_content);
//         assert!(input_partition.is_none());
//     }

//     #[test]
//     fn test_categorical_input_parameter_spec() {
//         let spec = InputParameterSpec::Categorical {
//             name: "C".to_string(),
//             rust_type: "usize".to_string(),
//             index: 0,
//             categories: vec!["0".to_string(), "1".to_string(), "2".to_string()],
//         };

//         let input_content = "1".to_string();
//         let input_partition = spec.get_input_partition(&input_content).unwrap();
//         assert_eq!(input_partition.key, "C=1");

//         let input_content = "3".to_string();
//         let input_partition = spec.get_input_partition(&input_content);
//         assert!(input_partition.is_none());
//     }

//     #[test]
//     fn test_input_group_builder() {
//         struct MockInputGenerator;
//         impl InputGenerator for MockInputGenerator {
//             fn generate_inputs(&self, seeds: &Vec<u64>) -> Result<Vec<String>, anyhow::Error> {
//                 let inputs: Vec<String> = seeds
//                     .iter()
//                     .map(|seed| format!("{} {}", seed, seed))
//                     .collect();
//                 Ok(inputs)
//             }
//         }

//         let input_params = vec![
//             InputParameterConfig::Int {
//                 name: "N".to_string(),
//                 rust_type: "usize".to_string(),
//                 parser: ParserConfig::Constant { index: 0 },
//                 partitions: vec![0, 1, 10],
//             },
//             InputParameterConfig::Int {
//                 name: "M".to_string(),
//                 rust_type: "usize".to_string(),
//                 parser: ParserConfig::Constant { index: 1 },
//                 partitions: vec![0, 2, 10],
//             },
//         ];

//         let input_generator = MockInputGenerator;
//         let input_group_builder = InputGroupBuilder::new(input_params, input_generator);
//         let case_num_per_group = 5;
//         let total_seed = 10;

//         let input_group_seeds = input_group_builder
//             .generate_input_group_seeds(case_num_per_group, total_seed)
//             .unwrap();
//         let input_group_seeds = input_group_seeds
//             .into_iter()
//             .map(|(k, v)| (k.key.0, v))
//             .collect::<HashMap<_, _>>();
//         assert_eq!(input_group_seeds.len(), 3);
//         assert_eq!(input_group_seeds["N=0-1_M=0-2"], vec![0]);
//         assert_eq!(input_group_seeds["N=1-10_M=0-2"], vec![1]);
//         assert_eq!(input_group_seeds["N=1-10_M=2-10"], vec![2, 3, 4, 5, 6]);
//     }
// }
