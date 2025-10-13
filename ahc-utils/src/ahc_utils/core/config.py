from typing import Literal

from pydantic import BaseModel, Field

from ahc_utils.core.input_param import InputParameter
from ahc_utils.core.optuna_param import OptunaParameter


class Settings(BaseModel):
    storage_path: str
    direction: Literal["maximize", "minimize"]
    score_type: Literal["relative", "absolute", "log10"]


class Pruner(BaseModel):
    threshold: float = Field(ge=0.0, le=1.0)


class OptunaConfig(BaseModel):
    settings: Settings
    pruner: Pruner
    params: list[OptunaParameter]
    ignore_params: list[str] = []


class AutotuneConfig(BaseModel):
    basedir: str
    timeout: int
    num_sample: int
    input_params: list[InputParameter]
    optuna: OptunaConfig
