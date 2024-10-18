from spl_transpiler.runtime.base import enforce_types, Expr
import pyspark.sql.functions as F


@enforce_types
def len_(x: Expr):
    return F.length(x.to_pyspark_expr())
