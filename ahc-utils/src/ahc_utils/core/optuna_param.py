from pydantic import BaseModel


class OptunaParameter(BaseModel):
    """
    optunaによってチューニングされるパラメータ
    """

    name: str
    type: str
    default: int | float
    min: int | float
    max: int | float

    def to_param_impl(self) -> str:
        def type_to_rust_type(t: str) -> str:
            if t == "int":
                return "i64"
            elif t == "float":
                return "f64"
            else:
                return t

        return f"{self.name}: {type_to_rust_type(self.type)}"

    def to_param_value(self, value: int | float) -> str:
        return f"{self.name}: {value}"
