from typing import Any

import optuna

from ahc_utils.core.input_param import InputParameter
from ahc_utils.core.optuna_param import OptunaParameter
from ahc_utils.core.partition import InputPartition


def get_best_values_from_optuna_study(study_name: str, storage_path: str) -> dict:
    study = optuna.load_study(
        study_name=study_name, storage=f"sqlite:///{storage_path}"
    )
    return study.best_trial.params


def generate_impl(
    best_params: dict[str, Any],
    params: list[OptunaParameter],
) -> str:
    """
    params_impl! {
        START_TEMP: f64 = 1000.,
        END_TEMP: f64 = 10.,
    }
    """

    def type_to_rust_type(t: str) -> str:
        if t == "int":
            return "i64"
        elif t == "float":
            return "f64"
        else:
            return t

    content = ""
    content += "params_impl! {\n"
    for d in params:
        t = type_to_rust_type(d.type)
        if d.name in best_params:
            value = best_params[d.name]
        else:
            value = d.default

        content += f"    {d.name}: {t} = {value},\n"

    content += "}\n\n"

    return content


def generate_param_impl(
    input_partitions_params: dict[InputPartition, dict],
    input_params: list[InputParameter],
    optuna_params: list[OptunaParameter],
) -> str:
    """
    params_impl! {
        { n: usize, m: usize },
        { START_TEMP: f64, END_TEMP: f64 },
        [
            ((0)..=(10), (10)..=(20)) => { START_TEMP: 1000., END_TEMP: 10. },
            ((11)..=(20), (10)..=(20)) => { START_TEMP: 2000., END_TEMP: 20. },
            _ => { START_TEMP: 2000., END_TEMP: 20. },
        ]
    }
    """

    ret = ""
    ret += "params_impl! {\n"
    ret += "    { " + ", ".join(p.to_param_impl() for p in input_params) + " },\n"
    ret += "    { " + ", ".join(p.to_param_impl() for p in optuna_params) + " },\n"
    ret += "    [\n"

    for partition, params in input_partitions_params.items():
        ret += f"        {partition.to_param_impl()} => {{ "
        ret += ", ".join(
            f"{p.name}: {params.get(p.name, p.default)}"
            for p in optuna_params
            if p.name in params
        )
        ret += " },\n"

    # default case
    ret += "        _ => { "
    ret += ", ".join(f"{p.name}: {p.default}" for p in optuna_params)
    ret += " },\n"
    ret += "    ]\n"
    ret += "}\n\n"

    return ret
