"""
inspired from https://github.com/terry-u16/pahcer/tree/main/optuna-sample
"""

import json
import math
import os
import signal
import subprocess

import classopt
import optuna
import yaml


@classopt.classopt(default_long=True)
class Args:
    study_name: str = classopt.config(
        "--study-name", required=True, help="Name of the study"
    )
    config_path: str = classopt.config(
        "--config", required=True, help="Path to config file"
    )
    timeout: int = classopt.config(
        "--timeout", default=600, help="Timeout in seconds for the optimization"
    )
    skip_optimize: bool = classopt.config(
        "--skip-optimize", action="store_true", help="Skip optimization"
    )
    param_file: str = classopt.config(
        "--param-file", default="params.rs", help="Path to the param file"
    )


class Objective:
    def __init__(self, config: dict) -> None:
        self.config = config

    def __call__(self, trial: optuna.trial.Trial) -> float:
        params = self.generate_params(trial)
        env = os.environ.copy()
        env.update(params)

        scores = []

        cmd = [
            "pahcer",
            "run",
            "--json",
            "--shuffle",
            "--no-result-file",
            "--freeze-best-scores",
        ]

        if trial.number != 0:
            cmd.append("--no-compile")

        process = subprocess.Popen(
            cmd,
            stdout=subprocess.PIPE,
            env=env,
        )

        # see also: https://tech.preferred.jp/ja/blog/wilcoxonpruner/
        for line in process.stdout:  # type: ignore
            result = json.loads(line)

            # If an error occurs, stop the process and raise an exception
            if result["error_message"] != "":
                process.send_signal(signal.SIGINT)
                score = 0.0
            else:
                score = self.extract_score(result)

            seed = result["seed"]
            scores.append(score)
            trial.report(score, seed)

            if trial.should_prune():
                print(f"Trial {trial.number} pruned.")
                process.send_signal(signal.SIGINT)

                objective_value = sum(scores) / len(scores)
                is_better_than_best = (
                    trial.study.direction == optuna.study.StudyDirection.MINIMIZE
                    and objective_value < trial.study.best_value
                ) or (
                    trial.study.direction == optuna.study.StudyDirection.MAXIMIZE
                    and objective_value > trial.study.best_value
                )

                if is_better_than_best:
                    # Avoid updating the best value
                    raise optuna.TrialPruned()
                else:
                    # It is recommended to return the value of the objective function
                    # at the current step instead of raising TrialPruned.
                    # This is a workaround to report the evaluation information
                    # of the pruned Trial to Optuna.
                    return sum(scores) / len(scores)

        return sum(scores) / len(scores)

    def extract_score(self, result: dict) -> float:
        score_type = self.config["problem"]["score_type"]
        if score_type == "absolute":
            absolute_score = result["score"]
            return absolute_score
        elif score_type == "log10":
            absolute_score = result["score"]
            log10_score = math.log10(absolute_score) if absolute_score > 0.0 else 0.0
            return log10_score
        elif score_type == "relative":
            relative_score = result["relative_score"]
            return relative_score
        else:
            raise ValueError(f"Unknown score_type: {score_type}")

    def generate_params(self, trial: optuna.trial.Trial) -> dict[str, str]:
        params = {}
        for d in self.config["params"]:
            if d["name"] in self.config["ignore_params"]:
                params[d["name"]] = d["default"]
                continue

            if d["type"] == "float":
                params[d["name"]] = trial.suggest_float(
                    d["name"],
                    eval(d["min"]),
                    eval(d["max"]),
                )
            elif d["type"] == "int":
                params[d["name"]] = trial.suggest_int(
                    d["name"],
                    eval(d["min"]),
                    eval(d["max"]),
                )
            else:
                raise ValueError(f"Unsupported type: {d['type']}")

        return {k: str(v) for k, v in params.items()}


def run_optuna(config: dict, args: Args) -> None:
    study = optuna.create_study(
        storage=config["settings"]["storage"],
        direction=config["problem"]["direction"],
        study_name=args.study_name,
        pruner=optuna.pruners.WilcoxonPruner(p_threshold=config["pruner"]["threshold"]),
        sampler=optuna.samplers.TPESampler(),
        load_if_exists=True,
    )
    study.optimize(Objective(config=config), timeout=args.timeout)


def generate_impl(study_name: str, config: dict) -> str:
    def type_to_rust_type(t: str) -> str:
        if t == "int":
            return "i64"
        elif t == "float":
            return "f64"
        else:
            raise ValueError(f"Unsupported type: {t}")

    def get_best_values_from_optuna_study(study_name: str, config: dict) -> dict:
        study = optuna.load_study(
            study_name=study_name, storage=config["settings"]["storage"]
        )
        return study.best_trial.params

    try:
        best_params = get_best_values_from_optuna_study(study_name, config)
    except Exception:
        best_params = {}

    content = ""
    content += "params_impl! {\n"
    for d in config["params"]:
        t = type_to_rust_type(d["type"])
        if d["name"] in best_params:
            value = best_params[d["name"]]
        else:
            value = d["default"]

        content += f'\t{d["name"]}: {t} = {value},\n'

    content += "}\n\n"

    return content


def main() -> None:
    args = Args.from_args()  # type: ignore

    with open(args.config_path, "r") as file:
        config = yaml.safe_load(file)

    print("args:", args)
    print("config:", config)

    try:
        if not args.skip_optimize:
            run_optuna(config, args)
        else:
            print("skip optimization")
    finally:
        impl = generate_impl(args.study_name, config)
        print("impl:")
        print(impl)

        with open(args.param_file, "w") as f:
            f.write(impl)


if __name__ == "__main__":
    main()
