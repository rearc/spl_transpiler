from spl_transpiler.runtime.base import enforce_types, Expr, ExprKind
import pyspark.sql.functions as F


@enforce_types
def _convert_to_expr(x: Expr):
    return x


def test_expr_col():
    from pyspark.sql.column import Column

    x = _convert_to_expr("col_name")
    assert x == Expr(kind=ExprKind.COLUMN, value="col_name")
    assert isinstance(x.to_pyspark_expr(), Column)


def test_expr_col_twice():
    from pyspark.sql.column import Column

    x = _convert_to_expr(_convert_to_expr("col_name"))
    assert x == Expr(kind=ExprKind.COLUMN, value="col_name")
    assert isinstance(x.to_pyspark_expr(), Column)


def test_expr_int():
    from pyspark.sql.column import Column

    x = _convert_to_expr(5)
    assert x == Expr(kind=ExprKind.LITERAL, value=5)
    assert isinstance(x.to_pyspark_expr(), Column)


def test_expr_func():
    from pyspark.sql.column import Column

    x = _convert_to_expr(F.length(F.lit("text")))
    assert isinstance(x, Expr)
    assert x.kind == ExprKind.RAW
    assert str(x.value) == str(F.length(F.lit("text")))
    assert isinstance(x.to_pyspark_expr(), Column)
