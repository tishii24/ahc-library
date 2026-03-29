from typing import Any

import optuna

from ahc_utils.core.config import OptunaParameter


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
