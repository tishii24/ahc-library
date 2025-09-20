from typing import Literal

from pydantic import BaseModel, Field


class Settings(BaseModel):
    storage: str
    direction: Literal["maximize", "minimize"]
    score_type: Literal["relative", "absolute", "log10"]


class Pruner(BaseModel):
    threshold: float = Field(ge=0.0, le=1.0)


class Parameter(BaseModel):
    name: str
    type: Literal["int", "float"]
    default: int | float
    min: int | float
    max: int | float


class OptunaConfig(BaseModel):
    settings: Settings
    pruner: Pruner
    params: list[Parameter]
    ignore_params: list[str] = []


class IntInputParameter(BaseModel):
    name: str
    type: Literal["int"]
    min: int
    max: int
    partitions: list[int]


class FloatInputParameter(BaseModel):
    name: str
    type: Literal["float"]
    min: float
    max: float
    partitions: list[float]


InputParameter = IntInputParameter | FloatInputParameter


class AutotuneConfig(BaseModel):
    basedir: str
    timeout: int
    num_sample: int
    input_params: list[InputParameter]
    optuna: OptunaConfig
