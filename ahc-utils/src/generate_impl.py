import classopt
import optuna
import yaml

from type import OptunaConfig


@classopt.classopt(default_long=True)
class Args:
    study_name: str = classopt.config(
        "--study-name", required=True, help="Name of the study"
    )


def get_best_values_from_optuna_study(study_name: str, storage: str) -> dict:
    study = optuna.load_study(study_name=study_name, storage=storage)
    return study.best_trial.params


def generate_impl(study_name: str, config: OptunaConfig) -> str:
    def type_to_rust_type(t: str) -> str:
        if t == "int":
            return "i64"
        elif t == "float":
            return "f64"
        else:
            raise ValueError(f"Unsupported type: {t}")

    try:
        best_params = get_best_values_from_optuna_study(
            study_name, config.settings.storage
        )
    except Exception:
        best_params = {}

    content = ""
    content += "params_impl! {\n"
    for d in config.params:
        t = type_to_rust_type(d.type)
        if d.name in best_params:
            value = best_params[d.name]
        else:
            value = d.default

        content += f"    {d.name}: {t} = {value},\n"

    content += "}\n\n"

    return content


def main() -> None:
    args = Args.from_args()  # type: ignore

    with open(args.config_path, "r") as file:
        config = yaml.safe_load(file)["optuna"]

    config = OptunaConfig(**config)

    impl = generate_impl(args.study_name, config)
    print(impl)


if __name__ == "__main__":
    main()
