from spl_transpiler import render_pyspark
from spl_transpiler.macros import substitute_macros, parse_with_macros


def test_macro_substitution():
    macros = {
        "f": dict(definition="index=main"),
        "g": dict(arguments=["f"], definition="ctime($f$)"),
    }

    assert substitute_macros("`f`", macros) == "index=main"
    assert substitute_macros("`f` | eval x=y", macros) == "index=main | eval x=y"
    assert (
        substitute_macros("`f` | eval x=`g(y)`", macros)
        == "index=main | eval x=ctime(y)"
    )
    assert (
        substitute_macros("`f` | eval x=`g(f=y)`", macros)
        == "index=main | eval x=ctime(y)"
    )

    from black import format_str, FileMode

    spl = "`f`"
    pyspark = r"spark.table('main')"

    assert (
        render_pyspark(parse_with_macros(spl, macros), format_code=True)
        == format_str(pyspark, mode=FileMode()).strip()
    )
