import ast
import logging
from typing import Any

log = logging.getLogger(__name__)


# https://stackoverflow.com/questions/33908794/get-value-of-last-expression-in-exec-call
def exec_with_return(code: str, globals: dict, locals: dict) -> Any | None:
    a = ast.parse(code)
    last_expression = None
    if a.body:
        if isinstance(a_last := a.body[-1], ast.Expr):
            last_expression = ast.unparse(a.body.pop())
        elif isinstance(a_last, ast.Assign):
            last_expression = ast.unparse(a_last.targets[0])
        elif isinstance(a_last, (ast.AnnAssign, ast.AugAssign)):
            last_expression = ast.unparse(a_last.target)
    main_body = ast.unparse(a)
    log.debug(f"Executing main code:\n```\n{main_body}\n```")
    exec(main_body, globals, locals)
    if last_expression:
        log.debug(f"Returning expression: `{last_expression}`")
        return eval(last_expression, globals, locals)
