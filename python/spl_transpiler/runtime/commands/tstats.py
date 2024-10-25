from typing import Any

from pydantic import BaseModel
from pyspark.sql import DataFrame

from spl_transpiler.runtime.base import enforce_types, Expr
from spl_transpiler.runtime.commands import data_model
from spl_transpiler.runtime.commands.fill_null import fill_null


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


class StatsExpr(BaseModel):
    def to_pyspark_expr(self, df) -> tuple[DataFrame, Any]:
        raise NotImplementedError()


@enforce_types
def tstats(
    # df: DataFrame | None,
    *,
    prestats=False,
    local=False,
    append=False,
    include_reduced_buckets=False,
    fill_null_value=None,
    from_: DataModelSpecifier | None = None,
    where: Expr | None = None,
    by: list[Expr] = (),
    **stat_exprs: StatsExpr,
) -> DataFrame:
    if prestats:
        raise NotImplementedError("`tstats` command does not implement prestats=True")
    if local:
        raise NotImplementedError("`tstats` command does not implement local=True")
    if include_reduced_buckets:
        raise NotImplementedError(
            "`tstats` command does not implement include_reduced_buckets=True"
        )
    if append:
        raise NotImplementedError("`tstats` command does not implement append=True")

    df = data_model(
        data_model_name=from_.datamodel,
        dataset_name=from_.nodename,
        summaries_only=False,
    )

    if fill_null_value is not None:
        df = fill_null(df, value=fill_null_value)

    if where:
        df = df.where(where.to_pyspark_expr())

    aggs = []
    for label, expr in stat_exprs.items():
        df, agg_expr = expr.to_pyspark_expr(df)
        aggs.append(agg_expr.alias(label))

    df = df.groupBy(*by)
    df = df.agg(*aggs)

    return df
