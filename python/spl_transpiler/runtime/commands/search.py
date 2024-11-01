from functools import reduce

from pyspark.sql import DataFrame, functions as F

from spl_transpiler.runtime.base import enforce_types, Expr
from spl_transpiler.runtime.config import settings


@enforce_types
def search(
    df: DataFrame | None = None,
    *exprs: Expr,
    index=None,
    source=None,
    source_type=None,
    host=None,
    host_tag=None,
    event_type=None,
    event_type_tag=None,
    saved_splunk=None,
    splunk_server=None,
    **field_exprs: Expr,
) -> DataFrame:
    if df is None:
        df = settings().table_lookup_function(
            index=index,
            source=source,
            source_type=source_type,
            host=host,
            host_tag=host_tag,
            event_type=event_type,
            event_type_tag=event_type_tag,
            saved_splunk=saved_splunk,
            splunk_server=splunk_server,
        )

    conditions = []
    for expr in exprs:
        conditions.append(expr.to_pyspark_expr())
    for field, expr in field_exprs.items():
        conditions.append(F.col(field) == expr.to_pyspark_expr())

    condition = reduce(lambda prev, cur: prev & cur, conditions, F.lit(True))
    df = df.where(condition)
    return df
