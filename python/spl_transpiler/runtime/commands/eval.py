from pyspark.sql import DataFrame

from spl_transpiler.runtime.base import enforce_types, Expr


@enforce_types
def eval(df: DataFrame, **exprs: Expr) -> DataFrame:
    for name, expr in exprs.items():
        df = df.withColumn(name, expr.to_pyspark_expr())
    return df
