import re
from typing import Protocol

from pydantic import BaseModel

from spl_transpiler.spl_transpiler import detect_macros, parse


class MacroDefinition(BaseModel):
    arguments: list[str] | None = None
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
                if macro_spec.arguments is None or len(macro_spec.arguments) != len(
                    args
                ):
                    raise ValueError(
                        f"Mismatched number of arguments in macro call: expected {len(macro_spec.arguments or [])}, got {len(args)}"
                    )
                args = dict(zip(macro_spec.arguments, [value for _, value in args]))
            elif all(name is not None for name, _ in args):
                if macro_spec.arguments is None or len(macro_spec.arguments) != len(
                    args
                ):
                    raise ValueError(
                        f"Mismatched number of arguments in macro call: expected {len(macro_spec.arguments or [])}, got {len(args)}"
                    )
                args = dict(args)
            else:
                raise ValueError(
                    "Mixture of named and positional arguments in macro call"
                )

            for arg_name, arg_substitute_value in args.items():
                macro_final_value = re.sub(
                    f"\\${arg_name}\\$", arg_substitute_value, macro_final_value
                )

        query_parts.append(macro_final_value)

    query_parts.append(suffix)
    return "".join(query_parts)


def parse_with_macros(code: str, macros: MacroLoader, *args, **kwargs):
    return parse(substitute_macros(code, macros), *args, **kwargs)
