import pytest
import os
import sys

# Ensures local spark cluster uses same python as is running this test
os.environ["PYSPARK_PYTHON"] = sys.executable


@pytest.fixture(scope="session", autouse=True)
def spark():
    from pyspark.sql import SparkSession

    return (
        SparkSession.builder.appName("Testing PySpark Example")
        .master("local[*]")
        .getOrCreate()
    )


@pytest.fixture(scope="session", autouse=True)
def sample_data_1(spark):
    return spark.createDataFrame(
        [
            ("src1", "hello world"),
            ("src1", "some text"),
            ("src2", "y=3"),
        ],
        ["_sourcetype", "raw"],
    )


@pytest.fixture(scope="session", autouse=True)
def sample_data_2(spark):
    from pyspark.sql.types import StructType, StructField, StringType, IntegerType

    return spark.createDataFrame(
        [
            ("src1", "hello world", 11),
            ("src1", "some text", 9),
            ("src2", "y=3", None),
        ],
        schema=StructType(
            [
                StructField("_sourcetype", StringType(), True),
                StructField("raw", StringType(), True),
                StructField("maybe_raw_length", IntegerType(), True),
            ]
        ),
    )


@pytest.fixture(scope="session", autouse=True)
def sample_silver_table(spark, sample_data_2):
    sample_data_2.write.saveAsTable("src1_silver", mode="overwrite")
    try:
        yield
    finally:
        spark.sql("DROP TABLE src1_silver")


@pytest.fixture(scope="session", autouse=True)
def sample_model_table(spark, sample_data_2):
    sample_data_2.write.saveAsTable("Model", mode="overwrite")
    try:
        yield
    finally:
        spark.sql("DROP TABLE Model")
