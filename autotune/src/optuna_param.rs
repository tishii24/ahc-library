use crate::model::OptunaParameterConfig;

impl OptunaParameterConfig {
    pub fn get_name(&self) -> String {
        match self {
            OptunaParameterConfig::Int { name, .. } | OptunaParameterConfig::Float { name, .. } => {
                name.clone()
            }
        }
    }

    pub fn get_def_impl(&self) -> String {
        match self {
            OptunaParameterConfig::Int {
                name, rust_type, ..
            }
            | OptunaParameterConfig::Float {
                name, rust_type, ..
            } => {
                format!("{}: {}", name, rust_type)
            }
        }
    }

    pub fn get_default_value_str(&self) -> String {
        match self {
            OptunaParameterConfig::Int { default, .. } => format!("{}", default),
            OptunaParameterConfig::Float { default, .. } => format!("{:.5}", default),
        }
    }
}
