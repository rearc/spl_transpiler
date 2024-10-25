from pyspark.sql import DataFrame, functions as F

from spl_transpiler.runtime.base import enforce_types


@enforce_types
def fill_null(
    df: DataFrame | None,
    value: str,
    fields: list[str] | None = None,
) -> DataFrame:
    type_map = dict(df.dtypes)
    fields = fields if fields is not None else list(type_map)

    for field in fields:
        df = df.withColumn(
            field,
            F.when(F.col(field).isNull(), F.lit(value).cast(type_map[field])).otherwise(
                F.col(field)
            ),
        )

    return df
