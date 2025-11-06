use std::collections::HashMap;

use crate::{
    input_param::{InputGroup, InputParameterSpec},
    model::OptunaParameterConfig,
};

/// Generates below.
/// ```ignore
/// params_impl! {
///     { n: usize, m: usize },
///     { START_TEMP: f64, END_TEMP: f64 },
///     [
///         ((0)..(10), (10)..(20)) => { START_TEMP: 1000., END_TEMP: 10. },
///         ((10)..(20), (10)..(20)) => { START_TEMP: 2000., END_TEMP: 20. },
///         _ => { START_TEMP: 2000., END_TEMP: 20. },
///     ]
/// }
/// ```
pub fn generate_param_impl(
    best_params: &Vec<(InputGroup, HashMap<String, String>)>,
    input_params: &Vec<Box<dyn InputParameterSpec>>,
    optuna_params: &Vec<OptunaParameterConfig>,
) -> String {
    fn joined_param_defs(
        optuna_params: &Vec<OptunaParameterConfig>,
        params: &HashMap<String, String>,
    ) -> String {
        let param_defs: Vec<String> = optuna_params
            .iter()
            .map(|p| p.get_name())
            .map(|name| {
                let value = params.get(&name).unwrap();
                format!("{}: {}", name, value)
            })
            .collect();
        param_defs.join(", ")
    }

    let mut ret = String::new();
    ret.push_str("params_impl! {\n");

    // Input parameters
    ret.push_str("    { ");
    let input_param_defs: Vec<String> = input_params.iter().map(|p| p.get_def_impl()).collect();
    ret.push_str(&input_param_defs.join(", "));
    ret.push_str(" },\n");

    // Optuna parameters
    ret.push_str("    { ");
    let optuna_param_defs: Vec<String> = optuna_params.iter().map(|p| p.get_def_impl()).collect();
    ret.push_str(&optuna_param_defs.join(", "));
    ret.push_str(" },\n");

    // Best parameters
    ret.push_str("    [\n");
    for (group, params) in best_params {
        ret.push_str(&format!("        ({}) => {{ ", group.match_arm_impl()));
        ret.push_str(&joined_param_defs(optuna_params, params));
        ret.push_str(" },\n");
    }

    // Default case
    ret.push_str("        _ => { ");
    let params = optuna_params
        .iter()
        .cloned()
        .map(|p| (p.get_name(), p.get_default_value_str()))
        .collect::<HashMap<_, _>>();
    ret.push_str(&joined_param_defs(optuna_params, &params));
    ret.push_str(" },\n");

    ret.push_str("    ]\n");
    ret.push_str("}\n");

    ret
}

#[cfg(test)]
mod tests {
    use crate::{
        input_param::InputPartition,
        model::{InputParameterConfig, ParserConfig},
    };

    use super::*;

    #[test]
    fn test_generate_param_impl() {
        let input_params = vec![
            InputParameterConfig::Int {
                name: "n".to_string(),
                rust_type: "usize".to_string(),
                parser: ParserConfig::Constant { index: 0 },
                partitions: vec![0, 10, 20],
            }
            .to_spec(),
            InputParameterConfig::Int {
                name: "m".to_string(),
                rust_type: "usize".to_string(),
                parser: ParserConfig::Constant { index: 1 },
                partitions: vec![10, 20],
            }
            .to_spec(),
        ];
        let optuna_params = vec![
            OptunaParameterConfig::Float {
                name: "START_TEMP".to_string(),
                rust_type: "f64".to_string(),
                min: 10.0,
                max: 1000.0,
                default: 100.0,
            },
            OptunaParameterConfig::Float {
                name: "END_TEMP".to_string(),
                rust_type: "f64".to_string(),
                min: 1.0,
                max: 100.0,
                default: 10.0,
            },
        ];
        let group1 = InputGroup::new(vec![InputPartition {
            key: "group1key".to_owned(),
            match_arm_impl: "group1impl".to_owned(),
        }]);
        let group2 = InputGroup::new(vec![InputPartition {
            key: "group2key".to_owned(),
            match_arm_impl: "group2impl".to_owned(),
        }]);
        let best_params = vec![
            (
                group1,
                vec![
                    ("START_TEMP".to_string(), "2000.0".to_string()),
                    ("END_TEMP".to_string(), "20.0".to_string()),
                ]
                .into_iter()
                .collect(),
            ),
            (
                group2,
                vec![
                    ("START_TEMP".to_string(), "3000.0".to_string()),
                    ("END_TEMP".to_string(), "30.0".to_string()),
                ]
                .into_iter()
                .collect(),
            ),
        ];

        let result = generate_param_impl(&best_params, &input_params, &optuna_params);
        let expected = r#"params_impl! {
    { n: usize, m: usize },
    { START_TEMP: f64, END_TEMP: f64 },
    [
        (group1impl) => { START_TEMP: 2000.0, END_TEMP: 20.0 },
        (group2impl) => { START_TEMP: 3000.0, END_TEMP: 30.0 },
        _ => { START_TEMP: 100.00000, END_TEMP: 10.00000 },
    ]
}
"#;

        assert_eq!(result, expected);
    }
}
