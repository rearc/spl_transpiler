from functools import reduce

from pyspark.sql import DataFrame, functions as F

from spl_transpiler.runtime.base import enforce_types, Expr


@enforce_types
def search(df: DataFrame | None, *exprs: Expr, **field_exprs: Expr) -> DataFrame:
    conditions = []
    for expr in exprs:
        conditions.append(expr.to_pyspark_expr())
    for field, expr in field_exprs.items():
        conditions.append(F.col(field) == expr.to_pyspark_expr())

    condition = reduce(lambda prev, cur: prev & cur, conditions, F.lit(True))
    df = df.where(condition)
    return df
