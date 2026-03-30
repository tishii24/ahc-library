use crate::config::OptunaParameterConfig;

pub trait ParamFormatter {
    /// (group_id, best_params)のリストを受け取って、最終的な出力形式に整形する
    fn format(
        &self,
        optuna_params: &Vec<OptunaParameterConfig>,
        params: &Vec<(String, serde_json::Map<String, serde_json::Value>)>,
    ) -> String;
}

/// `params_impl!`マクロの出力形式に整形するParamFormatterの実装
/// 例えば、以下のような出力を生成することを想定
/// ```rust
/// params_impl! {
///     { START_TEMP: f64, END_TEMP: f64 },
///     [
///         "group_0" => { START_TEMP: 1000.0, END_TEMP: 10.0 },
///         "group_1" => { START_TEMP: 5000.0, END_TEMP: 100.0 },
///         _ => { START_TEMP: 2000.0, END_TEMP: 20.0 },
///     ]
/// }
/// ```
pub struct ParamsImplFormatter;

impl ParamFormatter for ParamsImplFormatter {
    fn format(
        &self,
        optuna_params: &Vec<OptunaParameterConfig>,
        params: &Vec<(String, serde_json::Map<String, serde_json::Value>)>,
    ) -> String {
        let param_defs = optuna_params
            .iter()
            .map(|param| match param {
                OptunaParameterConfig::Int {
                    name,
                    rust_type,
                    default,
                } => (name.clone(), rust_type.clone(), default.to_string()),
                OptunaParameterConfig::Float {
                    name,
                    rust_type,
                    default,
                } => (
                    name.clone(),
                    rust_type.clone(),
                    serde_json::Value::from(*default).to_string(),
                ),
            })
            .collect::<Vec<_>>();

        let header = param_defs
            .iter()
            .map(|(name, rust_type, _)| format!("{}: {}", name, rust_type))
            .collect::<Vec<_>>()
            .join(", ");

        let group_lines = params
            .iter()
            .map(|(group_id, best_params)| {
                let values = param_defs
                    .iter()
                    .map(|(name, _, default)| {
                        let value = best_params
                            .get(name)
                            .map(|v| v.to_string())
                            .unwrap_or_else(|| default.clone());
                        format!("{}: {}", name, value)
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("        \"{}\" => {{ {} }},", group_id, values)
            })
            .collect::<Vec<_>>();

        let default_values = param_defs
            .iter()
            .map(|(name, _, default)| format!("{}: {}", name, default))
            .collect::<Vec<_>>()
            .join(", ");

        let mut lines = Vec::with_capacity(group_lines.len() + 6);
        lines.push("params_impl! {".to_string());
        lines.push(format!("    {{ {} }},", header));
        lines.push("    [".to_string());
        lines.extend(group_lines);
        lines.push(format!("        _ => {{ {} }},", default_values));
        lines.push("    ]".to_string());
        lines.push("}".to_string());
        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use crate::config::OptunaParameterConfig;

    use super::*;

    #[test]
    fn test_params_impl_formatter() {
        let optuna_params = vec![
            OptunaParameterConfig::Float {
                name: "START_TEMP".to_string(),
                rust_type: "f64".to_string(),
                default: 2000.0,
            },
            OptunaParameterConfig::Float {
                name: "END_TEMP".to_string(),
                rust_type: "f64".to_string(),
                default: 20.0,
            },
        ];
        let params = vec![
            (
                "group_0".to_string(),
                serde_json::Map::from_iter(vec![
                    ("START_TEMP".to_string(), serde_json::Value::from(1000.0)),
                    ("END_TEMP".to_string(), serde_json::Value::from(10.0)),
                ]),
            ),
            (
                "group_1".to_string(),
                serde_json::Map::from_iter(vec![
                    ("START_TEMP".to_string(), serde_json::Value::from(5000.0)),
                    ("END_TEMP".to_string(), serde_json::Value::from(100.0)),
                ]),
            ),
        ];
        let formatter = ParamsImplFormatter;
        let output = formatter.format(&optuna_params, &params);
        let expected_output = r#"params_impl! {
    { START_TEMP: f64, END_TEMP: f64 },
    [
        "group_0" => { START_TEMP: 1000.0, END_TEMP: 10.0 },
        "group_1" => { START_TEMP: 5000.0, END_TEMP: 100.0 },
        _ => { START_TEMP: 2000.0, END_TEMP: 20.0 },
    ]
}"#;
        assert_eq!(output, expected_output);
    }
}
