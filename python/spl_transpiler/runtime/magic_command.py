from argparse import ArgumentParser, BooleanOptionalAction

from spl_transpiler import convert_spl_to_pyspark
from spl_transpiler.runtime.utils import exec_with_return
from spl_transpiler.runtime import commands, functions

try:
    from IPython.core.magic import register_cell_magic, get_ipython  # noqa: F401
except ImportError:
    pass
else:

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
                return exec_with_return(
                    pyspark_code,
                    {},
                    {
                        "commands": commands,
                        "functions": functions,
                    },
                )
            except Exception as e:
                raise RuntimeError("Error executing PySpark code") from e
