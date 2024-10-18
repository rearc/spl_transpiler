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


@pytest.fixture()
def sample_data_1(spark):
    return spark.createDataFrame(
        [
            ("src1", "hello world"),
            ("src1", "some text"),
            ("src2", "y=3"),
        ],
        ["sourcetype", "raw"],
    )
