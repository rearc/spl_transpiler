import ast

from argparse import ArgumentParser, BooleanOptionalAction
from spl_transpiler import convert_spl_to_pyspark

try:
    from IPython.core.magic import register_cell_magic
except ImportError:

    def register_cell_magic(*args, **kwargs):
        return


# https://stackoverflow.com/questions/33908794/get-value-of-last-expression-in-exec-call
def exec_with_return(code: str, globals: dict, locals: dict) -> ... | None:
    a = ast.parse(code)
    last_expression = None
    if a.body:
        if isinstance(a_last := a.body[-1], ast.Expr):
            last_expression = ast.unparse(a.body.pop())
        elif isinstance(a_last, ast.Assign):
            last_expression = ast.unparse(a_last.targets[0])
        elif isinstance(a_last, (ast.AnnAssign, ast.AugAssign)):
            last_expression = ast.unparse(a_last.target)
    exec(ast.unparse(a), globals, locals)
    if last_expression:
        return eval(last_expression, globals, locals)


@register_cell_magic
def spl(line, cell):
    """Transpile SPL code into Pyspark and execute it."""
    from rich.jupyter import print
    from rich.syntax import Syntax

    parser = ArgumentParser()
    parser.add_argument(
        "--dry-run",
        action=BooleanOptionalAction,
        help="Dry run, do not execute code",
        default=False,
    )
    parser.add_argument(
        "--use-runtime",
        action=BooleanOptionalAction,
        help="Allow runtime execution",
        default=True,
    )
    parser.add_argument(
        "--format-code",
        action=BooleanOptionalAction,
        help="Format PySpark code",
        default=True,
    )
    parser.add_argument(
        "--print-code",
        action=BooleanOptionalAction,
        help="Print PySpark code",
        default=True,
    )
    args = parser.parse_args(line.split())

    spl_code = cell

    pyspark_code = convert_spl_to_pyspark(
        spl_code, allow_runtime=args.use_runtime, format_code=args.format_code
    )

    if args.print_code:
        print(Syntax(pyspark_code, "python"))

    if not args.dry_run:
        try:
            return exec_with_return(pyspark_code, globals(), {})
        except Exception as e:
            raise RuntimeError("Error executing PySpark code") from e
