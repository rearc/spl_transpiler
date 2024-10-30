from pydantic import BaseModel
from pyspark.sql import DataFrame

from spl_transpiler.runtime import commands
from spl_transpiler.runtime.base import enforce_types, Expr
from spl_transpiler.runtime.commands import data_model
from spl_transpiler.runtime.commands.fill_null import fill_null
from spl_transpiler.runtime.functions.stats import StatsFunction


class DataModelSpecifier(BaseModel):
    datamodel: str
    nodename: str | None = None

    def __str__(self) -> str:
        return ".".join(
            s
            for name in [self.datamodel.split("."), self.nodename or ""]
            for s in name.split(".")
            if s
        )


@enforce_types
def tstats(
    df: DataFrame | None = None,
    *,
    fill_null_value=None,
    from_: DataModelSpecifier | None = None,
    where: Expr | None = None,
    by: list[Expr] = (),
    **stat_exprs: StatsFunction,
) -> DataFrame:
    match (df, from_):
        case (None, from_):
            df = data_model(
                data_model_name=from_.datamodel,
                dataset_name=from_.nodename,
                summaries_only=False,
            )
        case (df, None):
            pass
        case (None, None):
            raise ValueError("Either `df` or `from_` must be provided")
        case _:
            raise ValueError("Invalid combination of `df` and `from_`")

    if fill_null_value is not None:
        df = fill_null(df, value=fill_null_value)

    if where:
        df = df.where(where.to_pyspark_expr())

    df = commands.stats(df, by=by, **stat_exprs)

    return df
