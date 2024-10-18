from typing import Any

from pyspark.sql import DataFrame

from spl_transpiler.runtime.base import enforce_types, Expr
from pydantic import BaseModel
from pyspark.sql import SparkSession


class DataModelSpecifier(BaseModel):
    datamodel: str
    nodename: str | None = None


class StatsExpr(BaseModel):
    name: str
    args: list[Expr]

    def to_pyspark_expr(self, df) -> tuple[DataFrame, Any]:
        raise NotImplementedError()


@enforce_types
def tstats(
    df: DataFrame | None,
    *,
    prestats=True,
    local=True,
    append=True,
    include_reduced_buckets=True,
    fill_null_value="",
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

    spark = SparkSession.builder.getOrCreate()
    match (df, from_):
        case (None, from_):
            match from_:
                case DataModelSpecifier(datamodel=datamodel, nodename=None):
                    df = spark.table(datamodel)
                case DataModelSpecifier(datamodel=datamodel, nodename=nodename):
                    table_name = ".".join(
                        datamodel.split(".") + nodename.split(".")[1:]
                    )
                    df = spark.table(table_name)
                case _:
                    raise ValueError(from_)
        case (df_, None):
            df = df_
        case (df_, from_):
            raise ValueError(
                "Cannot specify both an input dataframe and a source data model"
            )

    if fill_null_value is not None:
        df = df.na.fill(fill_null_value)

    if where:
        df = df.where(where.to_pyspark_expr())

    aggs = []
    for label, expr in stat_exprs.items():
        df, agg_expr = expr.to_pyspark_expr(df)
        aggs.append(agg_expr.alias(label))

    df = df.groupBy(*by)
    df = df.agg(*aggs)

    return df
