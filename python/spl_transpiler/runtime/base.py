import enum
from typing import Any

from pydantic import validate_call, ConfigDict, BaseModel, model_validator
import pyspark.sql.functions as F

enforce_types = validate_call(
    config=ConfigDict(arbitrary_types_allowed=True), validate_return=True
)


class ExprKind(str, enum.Enum):
    COLUMN = "column"
    LITERAL = "literal"
    RAW = "raw"


class Expr(BaseModel):
    kind: ExprKind
    value: Any

    @model_validator(mode="before")
    @classmethod
    def support_plain_types(cls, root_value):
        match root_value:
            case BaseModel() | dict():
                return root_value
            case str():
                return dict(kind=ExprKind.COLUMN, value=root_value)
            case int():
                return dict(kind=ExprKind.LITERAL, value=root_value)
            case float():
                return dict(kind=ExprKind.LITERAL, value=root_value)
            case bool():
                return dict(kind=ExprKind.LITERAL, value=root_value)
            case None:
                return dict(kind=ExprKind.LITERAL, value=None)
            case _:
                return dict(kind=ExprKind.RAW, value=root_value)

    def to_pyspark_expr(self):
        match self.kind:
            case ExprKind.COLUMN:
                return F.col(self.value)
            case ExprKind.LITERAL:
                return F.lit(self.value)
            case ExprKind.RAW:
                return self.value
            case _:
                raise ValueError(f"Unsupported expression kind: {self.kind}")


class TimeScale(str, enum.Enum):
    MONTHS = "months"
    WEEKS = "weeks"
    DAYS = "days"
    HOURS = "hours"
    MINUTES = "minutes"
    SECONDS = "seconds"


class TimeSpan(BaseModel):
    value: int
    scale: TimeScale
