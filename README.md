# Overview

`spl_transpiler` is a Rust + Python port of [Databricks Labs' `spl_transpiler`](https://github.com/databrickslabs/transpiler).
The goal is to provide a high-performance, highly portable, convenient tool for adapting common SPL code into PySpark code when possible, making it easy to migrate from Splunk to other data platforms for log processing.

# Installation

```pip install spl_transpiler```

# Usage

```python
from spl_transpiler import convert_spl_to_pyspark

print(convert_spl_to_pyspark(r"""multisearch
[index=regionA | fields +country, orders]
[index=regionB | fields +country, orders]"""))

# spark.table("regionA").select(F.col("country"), F.col("orders")).unionByName(
#     spark.table("regionB").select(F.col("country"), F.col("orders")),
#     allowMissingColumns=True,
# )
```

## Interactive CLI

For demonstration purposes and ease of use, an interactive CLI is also provided.

```bash
pip install spl_transpiler[cli]
python -m spl_transpiler
```

This provides an in-terminal user interface ([using `textual`](https://github.com/Textualize/textual)) where you can type an SPL query and see the converted Pyspark code in real time, alongside a visual representation of how the transpiler is understanding your query.

# Why?

Why transpile SPL into Spark?
Because a huge amount of domain knowledge is locked up in the Splunk ecosystem, but Splunk is not always the optimal place to store and analyze data.
Transpiling existing queries can make it easier for analysts and analytics to migrate iteratively onto other platforms.
SPL is also a very laser-focused language for certain analytics, and in most cases it's far more concise than other languages (PySpark _or_ SQL) at log processing tasks.
Therefore, it may be preferable to continue writing queries in SPL and use a transpiler layer to make that syntax viable on various platforms.

Why rewrite the Databricks Lab transpiler?
A few reasons:
1. The original transpiler is written in Scala and assumes access to a Spark environment. That requires a JVM to execute and possibly a whole ecosystem of software (maybe even a running Spark cluster) to be available. This transpiler stands alone and compiles natively to any platform.
2. While Scala is a common language in the Spark ecosystem, Spark isn't the only ecosystem that would benefit from having an SPL transpiler. By providing a transpiler that's both easy to use in Python and directly linkable at a system level, it becomes easy to embed and adapt the transpiler for any other use case too.
3. Speed. Scala's plenty fast, to be honest, but Rust is mind-numbingly fast. This transpiler can parse SPL queries and generate equivalent Python code in a fraction of a millisecond. This makes it viable to treat the transpiler as a realtime component, for example embedding it in a UI and re-computing the converted results after every keystroke.
4. Maintainability. Rust's type system helps keep things unambiguous as data passes through parsers and converters, and built-in unit testing makes it easy to adapt and grow the transpiler without risk of breaking existing features. While Rust is undoubtedly a language with a learning curve, the resulting code is very hard to break without noticing. This makes it much easier to maintain than a similarly complicated system would be in Python.

# Contributing

This project is in early development.
While it parses most common SPL queries and can convert a non-trivial variety of queries to PySpark, it's extremely limited and not yet ready for any serious usage.
However, it lays a solid foundation for the whole process and is modular enough to easily add incremental features to.

Ways to contribute:
- Add SPL queries and what the equivalent PySpark could would be. These test cases can drive development and prioritize the most commonly used features.
- Add support for additional functions and commands. While the SPL parser works for most commands, many do not yet the ability to render back out to PySpark. Please see `/src/pyspark/transpiler/command/eval_fns.rs` and `/convert_fns.rs` to add support for more in-command functions, and `/src/pyspark/transpiler/command/mod.rs` to add support for more top-level commands.

# Future

- Add UI (with `textual`) for interactive demonstration/use
