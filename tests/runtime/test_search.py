from pyspark import Row

from spl_transpiler.runtime import commands
from spl_transpiler import convert_spl_to_pyspark
import pyspark.sql.functions as F

from utils import assert_python_code_equals


def test_basic_search(sample_data_1):
    df = commands.search(sample_data_1, F.col("_sourcetype") == F.lit("src1"))
    results = df.collect()
    assert results == [
        Row(_sourcetype="src1", raw="hello world"),
        Row(_sourcetype="src1", raw="some text"),
    ]


def test_basic_search_kw(sample_data_1):
    df = commands.search(sample_data_1, _sourcetype=F.lit("src1"))
    results = df.collect()
    assert results == [
        Row(_sourcetype="src1", raw="hello world"),
        Row(_sourcetype="src1", raw="some text"),
    ]


def test_transpiled_search():
    transpiled_code = convert_spl_to_pyspark(
        r'index="lol" sourcetype="src1"',
        allow_runtime=True,
        format_code=True,
    )
    expected_code = r"""
    df_1 = commands.search(None, index='lol', source_type='src1')
    df_1
    """
    assert_python_code_equals(transpiled_code, expected_code)
