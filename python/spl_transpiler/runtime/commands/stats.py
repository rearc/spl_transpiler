from pyspark.sql import DataFrame

from spl_transpiler.runtime.base import enforce_types, Expr
from spl_transpiler.runtime.functions.stats import StatsFunction


@enforce_types
def stats(
    df: DataFrame,
    *,
    by: list[Expr] = (),
    **stat_exprs: StatsFunction,
) -> DataFrame:
    aggs = []
    for label, expr in stat_exprs.items():
        df, agg_expr = expr.to_pyspark_expr(df)
        aggs.append(agg_expr.alias(label))

    df = df.groupBy(*(v.to_pyspark_expr() for v in by))
    df = df.agg(*aggs)

    return df
