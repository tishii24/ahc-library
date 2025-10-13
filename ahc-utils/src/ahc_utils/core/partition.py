import abc


class InputPartition(abc.ABC):
    @abc.abstractmethod
    def is_included(self, inputs: dict) -> bool:
        raise NotImplementedError()

    @abc.abstractmethod
    def to_param_impl(self) -> str:
        raise NotImplementedError()


class NumericalInputPartition(InputPartition):
    def __init__(
        self, name: str, min_value: int | float, max_value: int | float
    ) -> None:
        self.name = name
        self.min_value = min_value
        self.max_value = max_value

    def is_included(self, inputs: dict) -> bool:
        value = inputs[self.name]
        return self.min_value <= value <= self.max_value

    def to_param_impl(self) -> str:
        return f"({self.min_value})..=({self.max_value})"

    def __str__(self) -> str:
        return f"{self.name}={self.min_value}-{self.max_value}"


class InputPartitionGroup(InputPartition):
    def __init__(self, partitions: list[InputPartition]) -> None:
        self.partitions = partitions

    def is_included(self, inputs: dict) -> bool:
        return all(p.is_included(inputs) for p in self.partitions)

    def to_param_impl(self) -> str:
        return "(" + ", ".join(p.to_param_impl() for p in self.partitions) + ")"

    def __str__(self) -> str:
        return "_".join(str(p) for p in self.partitions)
