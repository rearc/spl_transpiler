from pyspark import Row

import spl_transpiler.runtime.commands.eval
import spl_transpiler.runtime.functions.eval
from spl_transpiler import convert_spl_to_pyspark
from utils import assert_python_code_equals


def test_basic_eval(sample_data_1):
    df = spl_transpiler.runtime.commands.eval.eval(
        sample_data_1, raw_len=spl_transpiler.runtime.functions.eval.len_("raw")
    )
    results = df.collect()
    assert results == [
        Row(sourcetype="src1", raw="hello world", raw_len=11),
        Row(sourcetype="src1", raw="some text", raw_len=9),
        Row(sourcetype="src2", raw="y=3", raw_len=3),
    ]


def test_transpiled_eval():
    transpiled_code = convert_spl_to_pyspark(
        "eval raw_len=len(raw)",
        allow_runtime=True,
        format_code=True,
    )
    expected_code = r"""
    df_1 = commands.eval(spark.table("main"), raw_len=F.length(F.col("raw")))
    df_1
    """
    assert_python_code_equals(transpiled_code, expected_code)
