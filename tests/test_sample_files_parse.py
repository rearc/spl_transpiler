import json
import logging
import re
from collections import defaultdict
from pathlib import Path
from typing import Iterable

import pyparsing as pp
import pyparsing.exceptions
import pytest
from pydantic import BaseModel
from tabulate import tabulate

from spl_transpiler import render_pyspark, parse
from spl_transpiler.macros import (
    FolderMacroLoader,
    substitute_macros,
    MacroDefinition,
    CombinedMacroLoader,
)

log = logging.getLogger(__name__)

SAMPLE_DATA_ROOT = Path(__file__).parent / "sample_data"
SAMPLE_QUERIES_ROOT = SAMPLE_DATA_ROOT / "queries"
SAMPLE_MACROS_ROOT = SAMPLE_DATA_ROOT / "macros"

ALL_SPL_COMMANDS = [
    "abstract",
    "accum",
    "addcoltotals",
    "addinfo",
    "addtotals",
    "analyzefields",
    "anomalies",
    "anomalousvalue",
    "anomalydetection",
    "append",
    "appendcols",
    "appendpipe",
    "arules",
    "associate",
    "autoregress",
    "bin",
    "bucketdir",
    "chart",
    "cluster",
    "cofilter",
    "collect",
    "concurrency",
    "contingency",
    "convert",
    "correlate",
    "datamodel",
    "dbinspect",
    "dedup",
    "delete",
    "delta",
    "diff",
    "erex",
    "eval",
    "eventcount",
    "eventstats",
    "extract",
    "fieldformat",
    "fields",
    "fieldsummary",
    "filldown",
    "fillnull",
    "findtypes",
    "folderize",
    "foreach",
    "format",
    "from",
    "gauge",
    "gentimes",
    "geom",
    "geomfilter",
    "geostats",
    "head",
    "highlight",
    "history",
    "iconify",
    "inputcsv",
    "inputlookup",
    "iplocation",
    "join",
    "kmeans",
    "kvform",
    "loadjob",
    "localize",
    "localop",
    "lookup",
    "makecontinuous",
    "makemv",
    "makeresults",
    "map",
    "mcollect",
    "metadata",
    "metasearch",
    "meventcollect",
    "mpreview",
    "msearch",
    "mstats",
    "multikv",
    "multisearch",
    "mvcombine",
    "mvexpand",
    "nomv",
    "outlier",
    "outputcsv",
    "outputlookup",
    "outputtext",
    "overlap",
    "pivot",
    "predict",
    "rangemap",
    "rare",
    "redistribute",
    "regex",
    "reltime",
    "rename",
    "replace",
    "require",
    "rest",
    "return",
    "reverse",
    "rex",
    "rtorder",
    "savedsearch",
    "script",
    "scrub",
    "search",
    "searchtxn",
    "selfjoin",
    "sendalert",
    "sendemail",
    "set",
    "setfields",
    "sichart",
    "sirare",
    "sistats",
    "sitimechart",
    "sitop",
    "sort",
    "spath",
    "stats",
    "strcat",
    "streamstats",
    "table",
    "tags",
    "tail",
    "timechart",
    "timewrap",
    "tojson",
    "top",
    "transaction",
    "transpose",
    "trendline",
    "tscollect",
    "tstats",
    "typeahead",
    "typelearner",
    "typer",
    "union",
    "uniq",
    "untable",
    "walklex",
    "where",
    "x11",
    "xmlkv",
    "xmlunescape",
    "xpath",
    "xyseries",
]
SPL_COMMAND_ALIASES = {
    "bucket": "bin",
    "kv": "extract",
    "run": "script",
}

_query_files = [
    f"{f.parent.name}/{f.name}"
    for f in SAMPLE_QUERIES_ROOT.rglob("*.spl")
    if not f.name.startswith("__ignore__")
]


# class MacroLoader:
#     def __init__(self, root_dir=SAMPLE_MACROS_ROOT):
#         self.root_dir = root_dir
#
#     @cache
#     def __getitem__(self, item) -> MacroDefinition:
#         paths = list(self.root_dir.glob(f"*/{item}.json"))
#         if not paths:
#             log.warning(f"Could not find macro `{item}`, returning empty macro")
#             return MacroDefinition(definition="")
#         if len(paths) > 1:
#             raise RuntimeError(f"Found multiple macros named `{item}`")
#         path = paths[0]
#         return MacroDefinition.model_validate_json(path.read_text())


def empty_macro(item):
    log.warning(f"Could not find macro `{item}`, returning empty macro")
    return MacroDefinition(definition="")


@pytest.fixture(scope="session")
def macros():
    return CombinedMacroLoader(
        FolderMacroLoader(SAMPLE_MACROS_ROOT), fallback_fn=empty_macro
    )


def _expand(file_name, macros):
    path = SAMPLE_QUERIES_ROOT / file_name
    spl_query = path.read_text(encoding="utf-8")
    expanded_query = substitute_macros(spl_query, macros)
    if expanded_query != spl_query:
        log.info(f"{expanded_query=}")
    return expanded_query


def _print(*args, **kwargs):
    print(f"{args} {kwargs}")


class AstCommand(BaseModel):
    query: str
    subqueries: list["AstPipeline"] = []

    @classmethod
    def parse(cls, s, loc, toks):
        query = toks[0].strip()
        subqueries = [
            chunk["subquery"].model_dump()
            for chunk in toks["chunks"]
            if not isinstance(chunk, str) and "subquery" in chunk
        ]
        return cls(query=query, subqueries=subqueries)

    def __str__(self):
        return self.query

    def get_queries(self) -> Iterable[str]:
        for subquery in self.subqueries:
            yield from subquery.get_queries()

    def get_commands(self) -> Iterable[str]:
        yield self.query
        for subquery in self.subqueries:
            yield from subquery.get_commands()


class AstPipeline(BaseModel):
    query: str
    commands: list[AstCommand]
    prefix: str = ""

    @classmethod
    def parse(cls, s, loc, toks):
        prefix = "".join(toks.as_dict().get("prefix", []))
        commands = toks["pipeline"].as_list()
        query = " | ".join(c.query for c in commands)
        query = f"{prefix} {query}".strip()
        return cls(prefix=prefix, query=query, commands=commands)

    def __str__(self):
        return self.query

    def get_queries(self) -> Iterable[str]:
        yield self.query
        for command in self.commands:
            yield from command.get_queries()

    def get_commands(self) -> Iterable[str]:
        for command in self.commands:
            yield from command.get_commands()


Pipeline = pp.Forward()
Command = pp.Forward()
Subquery = pp.Group("[" + Pipeline("subquery") + "]")
QuotedString = pp.QuotedString(
    quote_char='"',
    unquote_results=False,
    esc_char="\\",
    convert_whitespace_escapes=False,
    multiline=True,
)
Command <<= pp.Combine(
    (pp.CharsNotIn('[]|"') | QuotedString | Subquery)("chunks")[1, ...]
).set_parse_action(AstCommand.parse)
Pipeline <<= (
    pp.Optional(pp.Optional(pp.White(min=0)) + "|")("prefix")
    + pp.DelimitedList(pp.Optional(Command("commands")), delim="|")("pipeline")
).set_parse_action(AstPipeline.parse)


def command_splitter(spl_query) -> AstPipeline:
    try:
        return Pipeline.parse_string(spl_query, parse_all=True)[0]
    except pyparsing.exceptions.ParseException as e:
        raise RuntimeError(f"Failed to pre-parse {spl_query=}") from e


class CommandTester:
    def __init__(self, command_query):
        self.query = command_query.strip()
        self.command = None
        self.parsed = None
        self.transpiled = None
        self.transpiled_code = None

    def run_test(self):
        self.parsed = self.transpiled = False

        self.command = self.query.split()[0].lower()
        self.command = SPL_COMMAND_ALIASES.get(self.command, self.command)
        if self.command not in ALL_SPL_COMMANDS:
            logging.info(f"Assuming implicit `search` for {self.query}")
            self.command = "search"

        try:
            parsed_ast = parse(self.query)
        except ValueError:
            raise RuntimeError("Failed to parse")
        self.parsed = True

        try:
            self.transpiled_code = render_pyspark(parsed_ast, format=False)
        except ValueError:
            raise RuntimeError("Failed to transpile")
        self.transpiled = True

    def to_json(self):
        return {
            "command": self.command,
            "query": self.query,
            "parsed": self.parsed,
            "transpiled": self.transpiled,
            "transpiled_code": self.transpiled_code,
        }


class QueryTester:
    def __init__(self, spl_query):
        self.query = spl_query
        self.parsed = None
        self.transpiled = None
        self.transpiled_code = None

    def run_test(self):
        try:
            parsed_ast = parse(self.query)
        except ValueError:
            raise RuntimeError("Failed to parse")
        self.parsed = True

        try:
            self.transpiled_code = render_pyspark(parsed_ast, format=False)
        except ValueError:
            raise RuntimeError("Failed to transpile")
        self.transpiled = True

    def to_json(self):
        return {
            "query": self.query,
            "parsed": self.parsed,
            "transpiled": self.transpiled,
            "transpiled_code": self.transpiled_code,
        }


class FileTester:
    def __init__(self, file_name, macros):
        self.file = SAMPLE_QUERIES_ROOT / file_name
        self.macros = macros
        self.raw_query = None
        self.expanded_query = None
        self.expanded = None
        self.preparsed = None
        self.query_tests = []
        self.command_tests = []

    def run_test(self):
        self.expanded = self.preparsed = False

        self.raw_query = self.file.read_text(encoding="utf-8")
        try:
            self.expanded_query = substitute_macros(self.raw_query, self.macros)
        except ValueError:
            return False
        self.expanded = True

        try:
            preparsed_query = command_splitter(self.expanded_query)
        except RuntimeError:
            log.exception(f"failure:file:preparse:{self.file.name}")
            return False
        self.preparsed = True

        self.query_tests = [
            QueryTester(query) for query in preparsed_query.get_queries()
        ]
        self.command_tests = [
            CommandTester(cmd) for cmd in preparsed_query.get_commands()
        ]

        success = True
        for i, query in enumerate(self.query_tests):
            try:
                query.run_test()
            except KeyboardInterrupt:
                raise
            except RuntimeError:
                log.warning(f"failure:query:{i}:{self.file.name}")
                success = False

        for i, cmd in enumerate(self.command_tests):
            try:
                cmd.run_test()
            except KeyboardInterrupt:
                raise
            except RuntimeError:
                log.warning(f"failure:command:{cmd.command}:{i}:{self.file.name}")
                success = False

        return success

    def to_json(self):
        return {
            "name": self.file.name,
            "raw_query": self.raw_query,
            "expanded_query": self.expanded_query,
            "expanded": self.expanded,
            "query_tests": [t.to_json() for t in self.query_tests],
            "command_tests": [t.to_json() for t in self.command_tests],
        }


def _print_file_results(file_tests: list[FileTester]):
    num_files_expanded = sum(bool(t.expanded) for t in file_tests)
    num_files_preparsed = sum(bool(t.preparsed) for t in file_tests)
    num_files = len(file_tests)

    log.error(f"Total Files:           {num_files}")
    log.error(
        f"Successful Expands:    {num_files_expanded} ({num_files_expanded / num_files:.2%})"
    )
    log.error(
        f"Successful Preparses:  {num_files_preparsed} ({num_files_preparsed / num_files:.2%})"
    )

    results = defaultdict(lambda: dict(prevented_parse=0, prevented_transpile=0))

    for file_test in file_tests:
        failed_to_parse = [c.command for c in file_test.command_tests if not c.parsed]
        failed_to_transpile = [
            c.command for c in file_test.command_tests if not c.transpiled
        ]

        failed_to_parse = ",".join(sorted(set(failed_to_parse)))
        failed_to_transpile = ",".join(sorted(set(failed_to_transpile)))

        if failed_to_parse:
            results[failed_to_parse]["prevented_parse"] += 1

        if failed_to_transpile:
            results[failed_to_transpile]["prevented_transpile"] += 1

    log.error(
        "Top-Level Failure Results:\n"
        + tabulate(
            [
                (cmd_set, counts["prevented_parse"], counts["prevented_transpile"])
                for cmd_set, counts in sorted(
                    results.items(),
                    key=lambda x: (
                        x[1]["prevented_transpile"],
                        x[1]["prevented_parse"],
                        x[0],
                    ),
                )
            ],
            headers=[
                "Command Set",
                "Prevented Parsing",
                "Prevented Transpiling",
            ],
            tablefmt="github",
        )
    )


def _print_query_results(query_tests: list[QueryTester]):
    num_queries_parsed = sum(bool(t.parsed) for t in query_tests)
    num_queries_transpiled = sum(bool(t.transpiled) for t in query_tests)
    num_queries = len(query_tests)

    log.error(f"Total Queries:          {num_queries}")
    log.error(
        f"Successful Parses:      {num_queries_parsed} ({num_queries_parsed / num_queries:.2%})"
    )
    log.error(
        f"Successful Transpiles:  {num_queries_transpiled} ({num_queries_transpiled / num_queries:.2%})"
    )


def _print_command_results(command_tests: list[CommandTester]):
    results = []

    for command in sorted(
        ALL_SPL_COMMANDS,
        key=lambda c: sum(not bool(t.parsed) for t in command_tests if t.command == c),
    ):
        relevant_tests = [t for t in command_tests if t.command == command]
        if not relevant_tests:
            continue

        num_commands_parsed = sum(bool(t.parsed) for t in relevant_tests)
        num_commands_transpiled = sum(bool(t.transpiled) for t in relevant_tests)
        num_commands = len(relevant_tests)

        results.append(
            [
                command,
                num_commands,
                f"{num_commands_parsed: <5d} ({num_commands_parsed / num_commands: >6.1%})",
                f"{num_commands_transpiled: <5d} ({num_commands_transpiled / num_commands: >6.1%})",
            ]
        )

    num_commands_parsed = sum(t.parsed for t in command_tests)
    num_commands_transpiled = sum(t.transpiled for t in command_tests)
    num_commands = len(command_tests)
    results.append(
        [
            "TOTAL",
            num_commands,
            f"{num_commands_parsed: <5d} ({num_commands_parsed / num_commands: >6.1%})",
            f"{num_commands_transpiled: <5d} ({num_commands_transpiled / num_commands: >6.1%})",
        ]
    )

    log.error(
        "Command Results:\n"
        + tabulate(
            results,
            headers=[
                "Command",
                "Samples",
                "Parses",
                "Transpiles",
            ],
            tablefmt="github",
        )
    )


def _print_function_results(command_tests: list[CommandTester]):
    counts = defaultdict(lambda: dict(total=0, success=0, failure=0))

    for command in command_tests:
        for func in re.findall(r"\b([a-z_]+)\(", command.query):
            counts[func]["total"] += 1
            counts[func]["success" if command.transpiled else "failure"] += 1

    log.error(
        "Function Results:\n"
        + tabulate(
            [
                (
                    func,
                    func_counts["total"],
                    func_counts["success"],
                    func_counts["failure"],
                )
                for func, func_counts in sorted(
                    counts.items(),
                    key=lambda item: (
                        item[1]["failure"] / item[1]["total"],
                        item[1]["total"],
                        item[0],
                    ),
                )
            ],
            headers=[
                "Function",
                "Samples",
                "Associated Successes",
                "Associated Failures",
            ],
            tablefmt="github",
        )
    )


def _save_results(file_tests: list[FileTester]):
    results_file = Path(__file__).resolve().parent / "sample_files_results.json"

    results = [t.to_json() for t in file_tests]
    json.dump(results, results_file.open("w", encoding="utf-8"), indent=2)


@pytest.mark.filterwarnings("ignore:.*Could not find macro")
def test_sample_queries(macros):
    success = True
    file_tests = [FileTester(f, macros) for f in _query_files]
    for test in file_tests:
        success &= test.run_test()

    query_tests = [query_test for test in file_tests for query_test in test.query_tests]
    command_tests = [cmd_test for test in file_tests for cmd_test in test.command_tests]

    _print_file_results(file_tests)
    _print_query_results(query_tests)
    _print_function_results(command_tests)
    _print_command_results(command_tests)
    _save_results(file_tests)
    assert success, "Not all queries or commands parsed properly"
