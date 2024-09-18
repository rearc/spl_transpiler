import logging
import re
from typing import Protocol

from pydantic import BaseModel

from spl_transpiler.spl_transpiler import detect_macros, parse

log = logging.getLogger(__name__)


class MacroDefinition(BaseModel):
    arguments: list[str] = []
    definition: str


class MacroLoader(Protocol):
    def __getitem__(self, item: str) -> dict | MacroDefinition:
        pass


def substitute_macros(code, macros: MacroLoader):
    (chunks, suffix) = detect_macros(code)
    query_parts = []
    for prefix, macro_call in chunks:
        query_parts.append(prefix)

        macro_name = macro_call.macro_name
        args = macro_call.args

        macro_spec = MacroDefinition.model_validate(macros[macro_name])

        macro_final_value = macro_spec.definition

        if args:
            if all(name is None for name, _ in args):
                if len(macro_spec.arguments) != len(args):
                    raise ValueError(
                        f"Mismatched number of arguments in macro call {macro_name}: expected {len(macro_spec.arguments)}, got {len(args)}: {args}"
                    )
                args = dict(zip(macro_spec.arguments, [value for _, value in args]))
            elif all(name is not None for name, _ in args):
                if len(macro_spec.arguments) != len(args):
                    raise ValueError(
                        f"Mismatched number of arguments in macro call {macro_name}: expected {len(macro_spec.arguments)}, got {len(args)}: {args}"
                    )
                args = dict(args)
            else:
                raise ValueError(
                    f"Mixture of named and positional arguments in macro call {macro_name}"
                )

            for arg_name, arg_substitute_value in args.items():
                macro_final_value = re.sub(
                    f"\\${arg_name}\\$", arg_substitute_value, macro_final_value
                )

        printable_args = ",".join(
            value if name is None else f"{name}={value}"
            for name, value in macro_call.args
        )
        log.debug(
            f"Resolved macro `{macro_name}({printable_args})` to {macro_final_value}"
        )

        query_parts.append(macro_final_value)

    query_parts.append(suffix)
    final_query = "".join(query_parts)
    final_query = final_query.rstrip().rstrip("|").strip()

    # Avoid infinite recursion
    if final_query == code:
        return final_query

    final_query = substitute_macros(final_query, macros)

    log.debug(f"Resolved query '''{code}''' to '''{final_query}")

    return final_query


def parse_with_macros(code: str, macros: MacroLoader, *args, **kwargs):
    return parse(substitute_macros(code, macros), *args, **kwargs)
