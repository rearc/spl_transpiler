from pyspark import Row

from spl_transpiler.runtime import commands, functions
from .utils import execute_spl_code


def test_basic_tstats():
    df = commands.tstats(
        from_=dict(datamodel="Model"),
        by=["sourcetype"],
        count=functions.stats.count(),
        avg=functions.stats.avg("maybe_raw_length"),
    )
    results = df.collect()
    assert results == [
        Row(sourcetype="src1", count=2, avg=10),
        Row(sourcetype="src2", count=1, avg=None),
    ]


def test_transpiled_tstats():
    spl_code = """
    tstats count avg(maybe_raw_length) from datamodel=Model by sourcetype
    """
    result = execute_spl_code(spl_code).collect()
    assert result == [
        Row(sourcetype="src1", count=2, avg=10),
        Row(sourcetype="src2", count=1, avg=None),
    ]
