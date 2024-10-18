from functools import reduce

from pyspark.sql import DataFrame
import pyspark.sql.functions as F
from ..base import Expr, enforce_types


@enforce_types
def search(df: DataFrame, *exprs: Expr, **field_exprs: Expr) -> DataFrame:
    conditions = []
    for expr in exprs:
        conditions.append(expr.to_pyspark_expr())
    for field, expr in field_exprs.items():
        conditions.append(F.col(field) == expr.to_pyspark_expr())

    condition = reduce(lambda prev, cur: prev & cur, conditions, F.lit(True))
    df = df.where(condition)
    return df


@enforce_types
def eval(df: DataFrame, **exprs: Expr) -> DataFrame:
    for name, expr in exprs.items():
        df = df.withColumn(name, expr.to_pyspark_expr())
    return df
