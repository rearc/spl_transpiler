from pyspark import Row

from spl_transpiler.runtime import commands
from spl_transpiler import convert_spl_to_pyspark
from utils import assert_python_code_equals


def test_basic_fill_null_specific(sample_data_2):
    df = commands.fill_null(sample_data_2, value="-1", fields=["maybe_raw_length"])
    results = df.collect()
    assert results == [
        Row(sourcetype="src1", raw="hello world", maybe_raw_length=11),
        Row(sourcetype="src1", raw="some text", maybe_raw_length=9),
        Row(sourcetype="src2", raw="y=3", maybe_raw_length=-1),
    ]


def test_basic_fill_null_global(sample_data_2):
    df = commands.fill_null(sample_data_2, value="-1", fields=None)
    results = df.collect()
    assert results == [
        Row(sourcetype="src1", raw="hello world", maybe_raw_length=11),
        Row(sourcetype="src1", raw="some text", maybe_raw_length=9),
        Row(sourcetype="src2", raw="y=3", maybe_raw_length=-1),
    ]


def test_transpiled_fill_null():
    transpiled_code = convert_spl_to_pyspark(
        "fillnull value=-1 maybe_raw_length",
        allow_runtime=True,
        format_code=True,
    )
    expected_code = r"""
    df_1 = commands.fill_null(None, value="-1", fields=["maybe_raw_length"])
    df_1
    """
    assert_python_code_equals(transpiled_code, expected_code)
