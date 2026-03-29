use std::collections::HashMap;

use crate::{
    input::specs::{InputGroup, InputParameterSpec},
    model::OptunaParameterConfig,
};

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
    input_params: &Vec<InputParameterSpec>,
    optuna_params: &Vec<OptunaParameterConfig>,
) -> String {
    todo!()
}
