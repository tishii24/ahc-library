import abc
import pathlib
import shutil
import subprocess
from itertools import product

import classopt
import yaml

from type import AutotuneConfig, FloatInputParameter, InputParameter, IntInputParameter

GENERATOR_CMD = "cargo run --release --bin gen seeds.txt"
STUDY_NAME = "autotune"


@classopt.classopt(default_long=True)
class Args:
    config_path: str = classopt.config(
        "--config", required=True, help="Path to config file"
    )


class Partition(abc.ABC):
    @abc.abstractmethod
    def is_included(self, inputs: dict) -> bool:
        raise NotImplementedError()


class InputParser(abc.ABC):
    @abc.abstractmethod
    def parse(self, input_file: pathlib.Path) -> dict:
        raise NotImplementedError()


class InputPartition:
    def __init__(self, partitions: list[Partition]) -> None:
        self.partitions = partitions

    def is_included(self, inputs: dict) -> bool:
        return all(p.is_included(inputs) for p in self.partitions)

    def __str__(self) -> str:
        return "_".join(str(p) for p in self.partitions)


class IntPartition(Partition):
    def __init__(self, name: str, min_value: int, max_value: int) -> None:
        self.name = name
        self.min_value = min_value
        self.max_value = max_value

    def is_included(self, inputs: dict) -> bool:
        value = inputs[self.name]
        return self.min_value <= value <= self.max_value

    def __str__(self) -> str:
        return f"{self.name}={self.min_value}-{self.max_value}"


class FloatPartition(Partition):
    def __init__(self, name: str, min_value: float, max_value: float) -> None:
        self.name = name
        self.min_value = min_value
        self.max_value = max_value

    def is_included(self, inputs: dict) -> bool:
        value = inputs[self.name]
        return self.min_value <= value <= self.max_value

    def __str__(self) -> str:
        return f"{self.name}={self.min_value}-{self.max_value}"


def optimize(
    work_dir: pathlib.Path, config_file: pathlib.Path, config: AutotuneConfig
) -> None:
    if config.optuna.settings.score_type == "relative":
        print("initial run for calculation of relative score...")
        subprocess.run("pahcer run", shell=True, cwd=work_dir)

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


def generate_input_partitions(
    input_params: list[InputParameter],
) -> list[InputPartition]:
    params: list[list] = []
    for param in input_params:
        if type(param) is IntInputParameter:
            params.append(
                [
                    IntPartition(
                        param.name, param.partitions[i], param.partitions[i + 1]
                    )
                    for i in range(len(param.partitions) - 1)
                ]
            )
        elif type(param) is FloatInputParameter:
            params.append(
                [
                    FloatPartition(
                        param.name, param.partitions[i], param.partitions[i + 1]
                    )
                    for i in range(len(param.partitions) - 1)
                ]
            )
        else:
            raise ValueError(f"Unsupported parameter type: {param.type}")

    return [InputPartition(list(p)) for p in product(*params)]


def generate_input_parser(_input_params: list[InputParameter]) -> InputParser:
    class Parser(InputParser):
        def parse(self, input_file: pathlib.Path) -> dict:
            with open(input_file, "r") as f:
                first_line = f.readline()

            n, _, _ = map(int, first_line.split())
            return {"N": n}

    return Parser()


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

        with open(work_dir / ".gitignore", "w") as f:
            f.write("*\n")

        generate_inputs(partition, input_parser, work_dir, config)
        optimize(work_dir, args.config_path, config)

    # for partition in input_partitions:
    #     work_dir = base_dir / str(partition)

    #     best_params = get_best_values_from_optuna_study(
    #         study_name=STUDY_NAME,
    #         storage=str(work_dir / config.optuna.settings.storage),
    #     )

    #     print(str(partition), best_params)


if __name__ == "__main__":
    main()
