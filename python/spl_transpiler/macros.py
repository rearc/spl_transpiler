import json
import logging
import re
from functools import cache
from itertools import islice
from pathlib import Path
from typing import Protocol, Mapping, Callable

from pydantic import BaseModel

from spl_transpiler.spl_transpiler import detect_macros, parse

log = logging.getLogger(__name__)


class MacroDefinition(BaseModel):
    arguments: list[str] = []
    definition: str


class MacroLoader(Protocol):
    def __getitem__(self, item: str) -> dict | MacroDefinition:
        pass


class RegistryMacroLoader(MacroLoader):
    def __init__(self, values: Mapping[str, MacroDefinition]):
        self.registry = {}
        self.registry.update(values)

    def register(self, name: str, definition: MacroDefinition):
        self.registry[name] = definition

    def __getitem__(self, item):
        return self.registry[item]


class FolderMacroLoader(MacroLoader):
    def __init__(self, root_dir: Path, suffix: str = ".json", loader=json.load):
        self.root_dir = root_dir
        self.suffix = suffix
        self.loader = loader

    @cache
    def __getitem__(self, item):
        macro_files = list(islice(self.root_dir.rglob(f"{item}{self.suffix}"), 2))
        if len(macro_files) > 1:
            raise ValueError(
                f"Multiple macro files found for `{item}`: {list(macro_files)}"
            )
        elif len(macro_files) < 1:
            raise KeyError(f"No macro found for `{item}`")
        macro_file = macro_files[0]

        if not macro_file.is_file():
            raise KeyError(f"Item found for `{item}` is not a file")

        with macro_file.open() as file:
            definition = self.loader(file)

        return MacroDefinition.model_validate(definition)


class CombinedMacroLoader(MacroLoader):
    def __init__(
        self, *loaders, fallback_fn: Callable[[str], MacroDefinition] | None = None
    ):
        self.loaders = loaders
        self.fallback_fn = fallback_fn

    def __getitem__(self, item):
        for loader in self.loaders:
            try:
                return loader[item]
            except KeyError:
                pass

        if self.fallback_fn is not None:
            return self.fallback_fn(item)

        raise KeyError(f"No macro found for `{item}`")


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
