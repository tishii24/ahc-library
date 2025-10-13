import abc
import pathlib
from typing import Any, Callable


class InputParser(abc.ABC):
    @abc.abstractmethod
    def parse(self, input_file: pathlib.Path) -> dict:
        raise NotImplementedError()


class ConstantParser(InputParser):
    def __init__(
        self,
        name: str,
        index: int,
        type: str,
    ) -> None:
        self.name = name
        self.index = index
        self.type = type

        if self.type in {"float", "f16", "f32", "f64"}:
            self.transform_fn: Callable[[str], Any] = float
        elif self.type in {
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
            self.transform_fn = int
        else:
            raise ValueError(f"Unsupported parser type: {self.type}")

    def parse(self, input_file: pathlib.Path) -> dict:
        with open(input_file, "r") as f:
            inputs = f.read().split()
            if self.index >= len(inputs):
                raise ValueError(f"Index {self.index} is out of range")

        return {self.name: self.transform_fn(inputs[self.index])}


class ParserGroup(InputParser):
    def __init__(self, parsers: list[InputParser]) -> None:
        self.parsers = parsers

    def parse(self, input_file: pathlib.Path) -> dict:
        params = {}
        for parser in self.parsers:
            params.update(parser.parse(input_file))
        return params
