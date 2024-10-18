from spl_transpiler.spl_transpiler import parse, render_pyspark


def convert_spl_to_pyspark(
    spl_code: str, allow_runtime: bool = False, format_code: bool = True
) -> str:
    return render_pyspark(
        parse(spl_code), allow_runtime=allow_runtime, format_code=format_code
    )
