import classopt
import yaml

from ahc_utils.core.config import OptunaConfig
from ahc_utils.core.generate_impl import (
    generate_impl,
    get_best_values_from_optuna_study,
)


@classopt.classopt(default_long=True)
class Args:
    config_path: str = classopt.config(
        "--config", required=True, help="Path to config file"
    )
    study_name: str = classopt.config(
        "--study-name", required=True, help="Name of the study"
    )


def main() -> None:
    args = Args.from_args()  # type: ignore

    with open(args.config_path, "r") as file:
        config = yaml.safe_load(file)["optuna"]

    config = OptunaConfig(**config)

    try:
        best_params = get_best_values_from_optuna_study(
            args.study_name, config.settings.storage_path
        )
    except Exception:
        best_params = {}

    impl = generate_impl(best_params, config.optuna.params)
    print(impl)


if __name__ == "__main__":
    main()
