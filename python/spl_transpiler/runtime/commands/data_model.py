import warnings

from pyspark.sql import DataFrame

from spl_transpiler.runtime.base import enforce_types

from spl_transpiler.runtime.config import settings


@enforce_types
def data_model(
    # df: DataFrame | None,
    data_model_name: str,
    dataset_name: str | None = None,
    search_mode: str | None = "search",
    strict_fields: bool | None = None,
    allow_old_summaries: bool | None = None,
    summaries_only: bool | None = None,
) -> DataFrame:
    # if df is not None:
    #     raise NotImplementedError("Cannot pass input to `data_model`")
    if search_mode and search_mode != "search":
        raise NotImplementedError(
            "`data_model` command does not support search_mode other than 'search'"
        )
    if strict_fields:
        warnings.warn(
            "`data_model` `strict_fields` parameter does not have any effect in Pyspark"
        )
    if allow_old_summaries:
        warnings.warn(
            "`data_model` `allow_old_summaries` parameter does not have any effect in Pyspark"
        )
    if summaries_only:
        warnings.warn(
            "`data_model` `summaries_only` parameter does not have any effect in Pyspark"
        )

    table_name = ".".join(
        data_model_name.split(".") + (dataset_name or "").split(".")[1:]
    )
    df = settings().table_lookup_function(table_name)

    return df
