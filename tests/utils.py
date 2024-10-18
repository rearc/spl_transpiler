import textwrap
from black import format_str, FileMode


def format_python_code(code):
    return format_str(
        textwrap.dedent(code),
        mode=FileMode(),
    ).strip()


def assert_python_code_equals(actual, expected):
    actual = format_python_code(actual)
    expected = format_python_code(expected)
    assert actual == expected
