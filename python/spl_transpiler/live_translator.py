from rich.console import RenderableType
from spl_transpiler import parse, render_pyspark
from textual import on
from textual.app import App, ComposeResult
from textual.containers import Horizontal, Vertical
from textual.message import Message
from textual.widgets import Header, Footer, TextArea, Tree, Static, RichLog
from textual.widgets._tree import TreeNode

# logging.basicConfig(level=logging.INFO, handlers=[TextualHandler()])
# log = logging.getLogger(__name__)


# Simple app with a single text box where the user types SPL code.
# Then we have another text box below that where we show the translated PySpark code
# In a side panel, we'll show the AST tree for the parsed SPL code.


class SplCode(Static):
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
        self.post_message(self.Changed(event.text_area.text))


class PysparkCode(Static):
    def compose(self):
        yield TextArea.code_editor(
            id="py_code",
            language="python",
            soft_wrap=True,
            read_only=True,
            tooltip="Auto-generated Pyspark code",
        )

    def update(self, renderable: RenderableType = "") -> None:
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

    # BINDINGS = [("d", "toggle_dark", "Toggle dark mode")]
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
        yield RichLog(id="log", markup=True)
        yield Footer()

    # @on(SplCode.Changed, "#spl")
    def on_spl_code_changed(self, event):
        log: RichLog = self.query_exactly_one("#log")
        log.clear()

        try:
            ast = parse(event.code)
        except ValueError as e:
            self.query_exactly_one("#ast").add_class("error")
            log.write(f"[red]Failed to parse SPL:[/red] [blue]{e}")
            return
        else:
            self.query_exactly_one("#ast").remove_class("error")
        self.query_exactly_one("#ast").set_ast(ast)

        try:
            pyspark_code = render_pyspark(ast)
        except RuntimeError as e:
            self.query_exactly_one("#pyspark").add_class("error")
            log.write(f"[red]Failed to generate Pyspark:[/red] [blue]{e}")
            return
        else:
            self.query_exactly_one("#pyspark").remove_class("error")
        self.query_exactly_one("#pyspark").update(pyspark_code)
