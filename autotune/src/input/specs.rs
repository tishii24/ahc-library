use std::collections::HashMap;

use crate::{
    input::generator::InputGenerator,
    model::{InputParameterConfig, ParserConfig},
};

/// 入力パラメータの形式・グルーピングの方法を定義する列挙型
pub enum InputParameterSpec {
    Int {
        name: String,
        rust_type: String,
        parser: InputParser,
        grouper: IntGrouper,
    },
    Float {
        name: String,
        rust_type: String,
        parser: InputParser,
        grouper: FloatGrouper,
    },
}
impl InputParameterSpec {
    pub fn name(&self) -> String {
        match self {
            InputParameterSpec::Int { name, .. } | InputParameterSpec::Float { name, .. } => {
                name.clone()
            }
        }
    }

    pub fn rust_type(&self) -> String {
        match self {
            InputParameterSpec::Int { rust_type, .. }
            | InputParameterSpec::Float { rust_type, .. } => rust_type.clone(),
        }
    }
}

impl InputParameterConfig {
    pub fn to_spec(self) -> InputParameterSpec {
        match self {
            InputParameterConfig::Int {
                name,
                rust_type,
                parser,
                partitions,
            } => match parser {
                ParserConfig::Constant { index } => InputParameterSpec::Int {
                    name,
                    rust_type,
                    parser: InputParser::Constant { index },
                    grouper: IntGrouper::Partition { partitions },
                },
            },
            InputParameterConfig::Float {
                name,
                rust_type,
                parser,
                partitions,
            } => match parser {
                ParserConfig::Constant { index } => InputParameterSpec::Float {
                    name,
                    rust_type,
                    parser: InputParser::Constant { index },
                    grouper: FloatGrouper::Partition { partitions },
                },
            },
        }
    }
}

enum InputParser {
    Constant { index: usize },
}
impl InputParser {
    pub fn parse<T: std::str::FromStr>(&self, input: &str) -> anyhow::Result<T> {
        match self {
            InputParser::Constant { index } => {
                let parts: Vec<&str> = input.split_whitespace().collect();
                if *index >= parts.len() {
                    anyhow::bail!("Index out of bounds for input parsing");
                }
                let value_str = parts[*index];
                let value = value_str
                    .parse::<T>()
                    .map_err(|_| anyhow::anyhow!("Failed to parse input parameter"))?;
                Ok(value)
            }
        }
    }
}

trait InputGrouper {
    /// 入力文字列を受け取って、どのグループに属するかを返す
    fn get_group_index(&self, input: &str) -> anyhow::Result<usize>;

    /// グループのindexを受け取って、グループのkeyを返す
    fn index_to_key(&self, index: usize) -> String;
}

enum IntGrouper {
    Partition { partitions: Vec<i64> },
}
impl InputGrouper for IntGrouper {
    fn get_group_index(&self, input: &str) -> anyhow::Result<usize> {
        let value: i64 = input
            .parse()
            .map_err(|_| anyhow::anyhow!("Failed to parse input parameter"))?;
        match self {
            IntGrouper::Partition { partitions } => {
                for (i, partition) in partitions.iter().enumerate() {
                    if value < *partition {
                        return Ok(i);
                    }
                }
                anyhow::bail!(
                    "Value out of bounds for grouping: value={}, partitions={:?}",
                    value,
                    partitions
                );
            }
        }
    }

    fn index_to_key(&self, index: usize) -> String {
        match self {
            IntGrouper::Partition { .. } => {
                format!("group_{}", index)
            }
        }
    }
}

enum FloatGrouper {
    Partition { partitions: Vec<f64> },
}
impl InputGrouper for FloatGrouper {
    fn get_group_index(&self, input: &str) -> anyhow::Result<usize> {
        match self {
            FloatGrouper::Partition { partitions } => {
                let value: f64 = input
                    .parse()
                    .map_err(|_| anyhow::anyhow!("Failed to parse input parameter"))?;
                for (i, partition) in partitions.iter().enumerate() {
                    if value < *partition {
                        return Ok(i);
                    }
                }
                anyhow::bail!(
                    "Value out of bounds for grouping: value={}, partitions={:?}",
                    value,
                    partitions
                );
            }
        }
    }

    fn index_to_key(&self, index: usize) -> String {
        match self {
            FloatGrouper::Partition { .. } => {
                format!("group_{}", index)
            }
        }
    }
}

pub struct InputGroup {
    key: String,
    seeds: Vec<u64>,
    /// param_group_indices[key] := specs[key]でグループにされるパラメータのindex
    param_group_indices: HashMap<String, usize>,
}
impl InputGroup {
    pub fn new(key: String, seeds: Vec<u64>, param_group_indices: HashMap<String, usize>) -> Self {
        Self {
            key,
            seeds,
            param_group_indices,
        }
    }
}

pub trait InputGroupGenerator {
    fn generate_input_groups<G: InputGenerator>(
        input_generator: G,
        input_param_specs: &Vec<InputParameterSpec>,
    ) -> anyhow::Result<Vec<InputGroup>>;
}
