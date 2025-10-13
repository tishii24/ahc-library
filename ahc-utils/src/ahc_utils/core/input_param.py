from pydantic import BaseModel

from ahc_utils.core.parser import ConstantParser, InputParser
from ahc_utils.core.partition import InputPartition, NumericalInputPartition


class ParserType(BaseModel):
    type: str
    params: dict


class InputParameter(BaseModel):
    """
    入力パラメータ
    """

    name: str
    type: str
    min: int | float
    max: int | float
    parser: ParserType
    partitions: list[int | float]

    def to_parser(self) -> InputParser:
        if self.parser.type == "constant":
            return ConstantParser(
                name=self.name,
                index=self.parser.params["index"],
                type=self.type,
            )
        else:
            raise ValueError(f"Unsupported parser type: {self.parser.type}")

    def to_partitions(self) -> list[InputPartition]:
        return [
            NumericalInputPartition(
                self.name, self.partitions[i], self.partitions[i + 1]
            )
            for i in range(len(self.partitions) - 1)
        ]

    def to_param_impl(self) -> str:
        return f"{self.name}: {self.type}"
