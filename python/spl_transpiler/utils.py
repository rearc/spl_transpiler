from spl_transpiler.spl_transpiler import parse, render_pyspark


def convert_spl_to_pyspark(spl_code: str, format: bool = True) -> str:
    return render_pyspark(parse(spl_code), format)
