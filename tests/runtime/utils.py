from pyspark.sql import DataFrame, functions as F

from spl_transpiler import convert_spl_to_pyspark
from spl_transpiler.runtime import commands, functions
from spl_transpiler.runtime.utils import exec_with_return

from rich import print
from rich.syntax import Syntax


def execute_spl_code(spl_code: str) -> DataFrame:
    pyspark_code = convert_spl_to_pyspark(
        spl_code=spl_code, allow_runtime=True, format_code=True
    )

    global_vars = {
        "commands": commands,
        "functions": functions,
        "F": F,
    }
    local_vars = {}

    print(Syntax(pyspark_code, "python"))

    return exec_with_return(pyspark_code, global_vars, local_vars)
