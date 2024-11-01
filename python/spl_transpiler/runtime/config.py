import importlib
from functools import cache
from typing import Annotated, Callable

from pydantic import BeforeValidator
from pydantic_settings import BaseSettings, SettingsConfigDict


def default_table_lookup(
    index=None,
    source=None,
    source_type=None,
    host=None,
    host_tag=None,
    event_type=None,
    event_type_tag=None,
    saved_splunk=None,
    splunk_server=None,
    data_model=None,
):
    from pyspark.sql import SparkSession, functions as F

    spark = SparkSession.builder.getOrCreate()

    if data_model:
        return spark.table(data_model)

    source_condition = (F.col("_source") == source) if source is not None else None
    source_type_condition = (
        (F.col("_sourcetype") == source_type) if source_type is not None else None
    )
    host_condition = (F.col("_host") == host) if host is not None else None
    host_tag_condition = (
        (F.col("_hosttag") == host_tag) if host_tag is not None else None
    )
    event_type_condition = (
        (F.col("_eventtype") == event_type) if event_type is not None else None
    )
    event_type_tag_condition = (
        (F.col("_eventtypetag") == event_type_tag)
        if event_type_tag is not None
        else None
    )
    saved_splunk_condition = (
        (F.col("_savedsearch") == saved_splunk) if saved_splunk is not None else None
    )
    splunk_server_condition = (
        (F.col("_splunk_server") == splunk_server)
        if splunk_server is not None
        else None
    )

    conditions = [
        source_condition,
        source_type_condition,
        host_condition,
        host_tag_condition,
        event_type_condition,
        event_type_tag_condition,
        saved_splunk_condition,
        splunk_server_condition,
    ]

    if index:
        table = spark.table(index)
    elif source or source_type:
        table_name = "_".join(v for v in [source, source_type, "silver"] if v)
        table = spark.table(table_name)
        conditions = conditions[2:]
    else:
        raise ValueError(
            "Either `index` or `source` or `source_type` must be provided for table lookup"
        )

    conditions = [c for c in conditions if c is not None]
    for condition in conditions:
        table = table.where(condition)

    return table


@BeforeValidator
def lookup_callable(v):
    if isinstance(v, str):
        try:
            package, name = v.split(":")
        except ValueError:
            raise ValueError(
                f"Invalid table lookup function `{v}`, should be of format `path.to.module:function_name`"
            )

        try:
            return getattr(importlib.import_module(package), name)
        except (ImportError, AttributeError) as e:
            raise ValueError(f"Error importing table lookup function `{v}`: {str(e)}")

    return v


class SplTranspilerSettings(BaseSettings):
    model_config = SettingsConfigDict(env_prefix="transpiler_")

    table_lookup_function: Annotated[Callable, lookup_callable] = default_table_lookup


@cache
def settings() -> SplTranspilerSettings:
    return SplTranspilerSettings()
