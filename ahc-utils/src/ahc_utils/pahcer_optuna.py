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

from ahc_utils.core.config import OptunaConfig


@classopt.classopt(default_long=True)
class Args:
    study_name: str = classopt.config(
        "--study-name", required=True, help="Name of the study"
    )
    storage_path: str = classopt.config(
        "--storage-path", required=True, help="Path to the storage file"
    )
    config_path: str = classopt.config(
        "--config", required=True, help="Path to config file"
    )


class Objective:
    def __init__(self, config: OptunaConfig) -> None:
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
        score_type = self.config.settings.score_type
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
        for d in self.config.params:
            if d.name in self.config.ignore_params:
                params[d.name] = d.default
                continue

            if d.type in {"float", "f16", "f32", "f64"}:
                params[d.name] = trial.suggest_float(
                    name=d.name,
                    low=d.min,
                    high=d.max,
                )
            elif d.type in {
                "int",
                "i16",
                "i32",
                "i64",
                "isize",
                "u16",
                "u32",
                "u64",
                "usize",
            }:
                params[d.name] = trial.suggest_int(
                    name=d.name,
                    low=int(d.min),
                    high=int(d.max),
                )
            else:
                raise ValueError(f"Unsupported type: {d.type}")

        return {k: str(v) for k, v in params.items()}


def run_optuna(study_name: str, storage_path: str, config: OptunaConfig) -> None:
    study = optuna.create_study(
        storage=storage_path,
        direction=config.settings.direction,
        study_name=study_name,
        pruner=optuna.pruners.WilcoxonPruner(p_threshold=config.pruner.threshold),
        sampler=optuna.samplers.TPESampler(),
        load_if_exists=True,
    )
    study.optimize(Objective(config=config), timeout=config.settings.timeout)


def main() -> None:
    args = Args.from_args()  # type: ignore

    with open(args.config_path, "r") as file:
        config = yaml.safe_load(file)

    config = OptunaConfig(**config)

    print("args:", args)
    print("config:", config)

    try:
        run_optuna(args.study_name, args.storage_path, config)
    except KeyboardInterrupt:
        print("interrupted.")


if __name__ == "__main__":
    main()
