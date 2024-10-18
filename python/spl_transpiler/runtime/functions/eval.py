from pyspark.sql import functions as F

from spl_transpiler.runtime.base import enforce_types, Expr


@enforce_types
def len_(x: Expr):
    return F.length(x.to_pyspark_expr())
