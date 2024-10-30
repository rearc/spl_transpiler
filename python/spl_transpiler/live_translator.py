import logging

import pyperclip
from rich.console import RenderableType
from textual.logging import TextualHandler
from textual.reactive import reactive

from spl_transpiler import parse, render_pyspark
from textual import on
from textual.app import App, ComposeResult
from textual.containers import Horizontal, Vertical
from textual.message import Message
from textual.widgets import Header, Footer, TextArea, Tree, Static, RichLog, Switch
from textual.widgets._tree import TreeNode

logging.basicConfig(level=logging.INFO, handlers=[TextualHandler()])


# Simple app with a single text box where the user types SPL code.
# Then we have another text box below that where we show the translated PySpark code
# In a side panel, we'll show the AST tree for the parsed SPL code.


class SplCode(Static):
    code: reactive[str] = ""

    class Changed(Message):
        def __init__(self, code):
            self.code = code
            super().__init__()

    def compose(self):
        yield TextArea.code_editor(
            id="spl_code", soft_wrap=True, tooltip="Type your SPL code here"
        )

    @on(TextArea.Changed, "#spl_code")
    def on_update(self, event):
        self.code = event.text_area.text
        self.post_message(self.Changed(self.code))


class PysparkCode(Static):
    code: reactive[str] = ""

    def compose(self):
        yield TextArea.code_editor(
            id="py_code",
            language="python",
            soft_wrap=True,
            read_only=True,
            tooltip="Auto-generated Pyspark code",
        )

    def update(self, renderable: RenderableType = "") -> None:
        self.code = str(renderable)
        self.query_one("#py_code").text = renderable


class ASTTree(Static):
    def compose(self):
        yield Tree(id="tree", label="root")

    def set_ast(self, ast):
        def _create_tree(node: TreeNode, obj):
            if hasattr(obj, "_0") and not hasattr(obj, "_1"):
                v = obj._0
                if ": " not in node.label:
                    repr_v = repr(v)
                    if not (repr_v.startswith("<") and repr_v.endswith(">")):
                        node.label = f"{node.label}: {repr_v}"
                _create_tree(node, v)
                return

            for attr in dir(obj):
                if attr.startswith("__") or attr[0].isupper():
                    continue

                match getattr(obj, attr):
                    case str(v) | int(v) | bool(v):
                        node.add(
                            f"[blue][strong]{attr}:[/strong][/blue] {v}", expand=True
                        )
                    case list(lst):
                        child = node.add(f"[blue][strong]{attr}:", expand=True)
                        for i, v in enumerate(lst):
                            _create_tree(child.add(f"[dim][strong]{i}", expand=True), v)
                    case dict(lst):
                        child = node.add(f"[blue][strong]{attr}:", expand=True)
                        for k, v in enumerate(lst):
                            _create_tree(child.add(f"[dim][strong]{k}", expand=True), v)
                    case v:
                        repr_v = repr(v)
                        if repr_v.startswith("<") and repr_v.endswith(">"):
                            label = f"[dim][strong]{attr}"
                        else:
                            label = f"[dim][strong]{attr}:[/strong][/dim] {repr(v)}"
                        _create_tree(node.add(label, expand=True), v)

        tree: Tree = self.query_exactly_one("#tree")
        tree.clear()
        _create_tree(tree.root, ast)
        tree.root.expand()


class SPLApp(App):
    """A Textual app to transpile SPL code to PySpark."""

    BINDINGS = [
        ("ctrl+r", "toggle_runtime", "Toggle runtime"),
        ("ctrl+t", "toggle_imports", "Toggle imports"),
        ("ctrl+s", "copy_pyspark", "Copy PySpark to clipboard"),
    ]
    CSS_PATH = "live_translator.tcss"

    def compose(self) -> ComposeResult:
        """Create child widgets for the app."""
        yield Header()
        yield Horizontal(
            Vertical(
                SplCode("SPL code", id="spl"),
                PysparkCode("Pyspark", id="pyspark"),
                id="code_area",
            ),
            ASTTree(id="ast"),
            id="body",
        )
        yield Horizontal(
            Static("Use runtime: ", classes="label"),
            Switch(value=True, id="use_runtime"),
            Static("Include imports: ", classes="label"),
            Switch(value=True, id="add_imports"),
            RichLog(id="log", markup=True),
            classes="container",
        )
        yield Footer()

    def action_toggle_runtime(self) -> None:
        self.query_exactly_one("#use_runtime").toggle()

    def action_toggle_imports(self) -> None:
        self.query_exactly_one("#add_imports").toggle()

    def action_copy_pyspark(self) -> None:
        pyperclip.copy(self.query_exactly_one("#pyspark").code)
        self.notify(
            "Pyspark code copied successfully to your clipboard",
            severity="information",
            timeout=3,
            title="Copy successful",
        )

    # @on(SplCode.Changed, "#spl")
    def on_spl_code_changed(self, event):
        self.update_pyspark_code()

    def on_switch_changed(self, event):
        self.update_pyspark_code()

    def update_pyspark_code(self):
        use_runtime = self.query_exactly_one("#use_runtime").value
        add_imports = self.query_exactly_one("#add_imports").value
        log: RichLog = self.query_exactly_one("#log")
        log.clear()

        try:
            ast = parse(self.query_exactly_one("#spl").code)
        except ValueError as e:
            self.query_exactly_one("#ast").add_class("error")
            log.write(f"[red]Failed to parse SPL:[/red] [blue]{e}")
            return
        else:
            self.query_exactly_one("#ast").remove_class("error")
        self.query_exactly_one("#ast").set_ast(ast)

        try:
            pyspark_code = render_pyspark(
                ast, allow_runtime=use_runtime, format_code=True
            )
        except RuntimeError as e:
            self.query_exactly_one("#pyspark").add_class("error")
            log.write(f"[red]Failed to generate Pyspark:[/red] [blue]{e}")
            return
        else:
            self.query_exactly_one("#pyspark").remove_class("error")
        if add_imports:
            chunks = []
            chunks.append("from pyspark.sql import functions as F")
            if use_runtime:
                chunks.append("from spl_transpiler.runtime import commands, functions")
            chunks.append(pyspark_code)
            pyspark_code = "\n".join(chunks)
        self.query_exactly_one("#pyspark").update(pyspark_code)
