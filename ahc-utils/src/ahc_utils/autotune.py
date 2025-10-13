import pathlib
import shutil
import subprocess
from itertools import product

import classopt
import yaml

from ahc_utils.core.config import AutotuneConfig
from ahc_utils.core.generate_impl import (
    generate_param_impl,
    get_best_values_from_optuna_study,
)
from ahc_utils.core.input_param import InputParameter
from ahc_utils.core.parser import InputParser, ParserGroup
from ahc_utils.core.partition import InputPartition, InputPartitionGroup

GENERATOR_CMD = "cargo run --release --bin gen seeds.txt"
STUDY_NAME = "autotune"


@classopt.classopt(default_long=True)
class Args:
    config_path: str = classopt.config(
        "--config", required=True, help="Path to config file"
    )


def optimize(
    work_dir: pathlib.Path, config_file: pathlib.Path, config: AutotuneConfig
) -> None:
    if config.optuna.settings.score_type == "relative":
        print("initial run for calculation of relative score...")
        subprocess.run(["pahcer", "run"], cwd=work_dir)

    subprocess.run(
        [
            "pahcer-optuna",
            "--study_name",
            STUDY_NAME,
            "--config_path",
            str(config_file),
            "--timeout",
            str(config.timeout),
        ],
        cwd=work_dir,
    )


def generate_inputs(
    partition: InputPartition,
    input_parser: InputParser,
    work_dir: pathlib.Path,
    config: AutotuneConfig,
) -> None:
    # generate samples
    tool_dir = work_dir / "tools"
    with open(tool_dir / "seeds.txt", "w") as f:
        f.write("\n".join(str(x) for x in range(config.num_sample)))
    subprocess.run(GENERATOR_CMD, shell=True, cwd=tool_dir)

    adopted_files = []
    for file in (tool_dir / "in").glob("*.txt"):
        inputs = input_parser.parse(file)
        if partition.is_included(inputs):
            adopted_files.append(file)

    print("adopted files:", len(adopted_files))

    for i, adopted_file in enumerate(adopted_files):
        shutil.copy(adopted_file, tool_dir / "in" / f"{i:04}.txt")


def generate_input_parser(_input_params: list[InputParameter]) -> InputParser:
    parsers = [param.to_parser() for param in _input_params]
    return ParserGroup(parsers=parsers)


def generate_input_partitions(
    input_params: list[InputParameter],
) -> list[InputPartition]:
    partitions = [param.to_partitions() for param in input_params]
    return [InputPartitionGroup(list(p)) for p in product(*partitions)]


def main() -> None:
    args = Args.from_args()  # type: ignore

    with open(args.config_path, "r") as file:
        config = yaml.safe_load(file)

    config = AutotuneConfig(**config)
    base_dir = pathlib.Path(config.basedir)
    base_dir.mkdir(parents=True, exist_ok=True)

    input_partitions = generate_input_partitions(config.input_params)
    input_parser = generate_input_parser(config.input_params)

    for partition in input_partitions:
        work_dir = base_dir / str(partition)

        if work_dir.exists():
            print("removing existing directory:", work_dir)
            shutil.rmtree(work_dir)

        shutil.copytree(
            ".",
            work_dir,
            ignore=shutil.ignore_patterns(".git", config.basedir),
            dirs_exist_ok=True,
        )
        shutil.rmtree(work_dir / "pahcer")

        with open(work_dir / ".gitignore", "w") as f:
            f.write("*\n")

        generate_inputs(partition, input_parser, work_dir, config)
        optimize(work_dir, args.config_path, config)

    input_partitions_params = {}
    for partition in input_partitions:
        work_dir = base_dir / str(partition)
        try:
            best_values = get_best_values_from_optuna_study(
                STUDY_NAME, work_dir / config.optuna.settings.storage_path
            )
            input_partitions_params[partition] = best_values
        except Exception as e:
            print(f"failed to get best values for {partition}: {e}")
            input_partitions_params[partition] = {}

    impl = generate_param_impl(
        input_partitions_params, config.input_params, config.optuna.params
    )
    print(impl)


if __name__ == "__main__":
    main()
