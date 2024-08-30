use crate::ast::ast;
use crate::pyspark::ast::*;
use crate::pyspark::dealias::Dealias;
use crate::pyspark::transpiler::utils::convert_time_format;
use anyhow::{bail, Result};
use phf::phf_map;

impl TryFrom<ast::Expr> for i64 {
    type Error = anyhow::Error;

    fn try_from(value: ast::Expr) -> std::result::Result<i64, Self::Error> {
        match value {
            ast::Expr::Leaf(ast::LeafExpr::Constant(ast::Constant::Int(ast::IntValue(val)))) => {
                Ok(val)
            }
            _ => bail!("No default conversion from {:?} to i64", value),
        }
    }
}

impl TryFrom<ast::Expr> for f64 {
    type Error = anyhow::Error;
    fn try_from(value: ast::Expr) -> std::result::Result<f64, Self::Error> {
        match value {
            ast::Expr::Leaf(ast::LeafExpr::Constant(ast::Constant::Double(ast::DoubleValue(
                val,
            )))) => Ok(val),
            _ => bail!("No default conversion from {:?} to f64", value),
        }
    }
}

impl TryFrom<ast::Expr> for String {
    type Error = anyhow::Error;

    fn try_from(value: ast::Expr) -> std::result::Result<String, Self::Error> {
        match value {
            ast::Expr::Leaf(ast::LeafExpr::Constant(const_)) => match const_ {
                ast::Constant::Null(_) => {
                    bail!("No default conversion from {:?} to String", const_)
                }
                ast::Constant::Bool(_) => {
                    bail!("No default conversion from {:?} to String", const_)
                }
                ast::Constant::Int(_) => bail!("No default conversion from {:?} to String", const_),
                ast::Constant::Double(_) => {
                    bail!("No default conversion from {:?} to String", const_)
                }
                ast::Constant::Str(ast::StrValue(val)) => Ok(val.clone()),
                ast::Constant::SnapTime(_) => {
                    bail!("No default conversion from {:?} to String", const_)
                }
                ast::Constant::SplSpan(_) => {
                    bail!("No default conversion from {:?} to String", const_)
                }
                ast::Constant::Field(ast::Field(val)) => Ok(val.clone()),
                ast::Constant::Wildcard(ast::Wildcard(val)) => Ok(val.clone()),
                ast::Constant::Variable(ast::Variable(val)) => Ok(val.clone()),
                ast::Constant::IPv4CIDR(ast::IPv4CIDR(val)) => Ok(val.clone()),
            },
            _ => bail!("Unsupported mvindex start argument: {:?}", value),
        }
    }
}

fn map_arg<E, T>(arg: &ast::Expr) -> Result<T>
where
    E: Into<anyhow::Error>,
    T: TryFrom<ast::Expr, Error = E> + Dealias,
{
    arg.clone()
        .try_into()
        .map(|e: T| e.unaliased())
        .map_err(|e: E| e.into())
}

fn map_args<E, T>(args: Vec<ast::Expr>) -> Result<Vec<T>>
where
    E: Into<anyhow::Error>,
    T: TryFrom<ast::Expr, Error = E> + Dealias,
{
    args.iter().map(|arg| map_arg(arg)).collect()
}

static SIMPLE_FUNC_MAP: phf::Map<&'static str, &'static str> = phf_map! {
    "len" => "length",
    "mvcount" => "size",
    "mvappend" => "concat",
    "min" => "least",
    "max" => "greatest",
};

pub fn eval_fn(call: ast::Call) -> Result<ColumnLike> {
    let ast::Call { name, args } = call;

    match (name.as_str(), args.len()) {
        // if(condition, then_expr, else_expr) -> when(condition, then_expr).otherwise(else_expr)
        ("if", 3) => {
            let condition: Expr = map_arg(&args[0])?;
            let then_expr: Expr = map_arg(&args[1])?;
            let else_expr: Expr = map_arg(&args[2])?;

            Ok(column_like!([when([condition], [then_expr])].otherwise([else_expr])).into())
        }

        // coalesce(a, b, ...) -> coalesce(col(a), col(b), ...)
        // ("coalesce", _) => {
        //     let cols: Vec<Expr> = map_args(args)?;
        //     let coalesce = ColumnLike::FunctionCall {
        //         func: "coalesce".to_string(),
        //         args: cols,
        //     };
        //     Ok(column_like!([coalesce].alias("coalesce")).into())
        // }

        // mvcount(d) -> size(col(d))
        // ("mvcount", 1) => {
        //     let column: Expr = map_arg(&args[0])?;
        //     Ok(column_like!([size([column])].alias("mvcount")).into())
        // }

        // mvindex(a, b, c) -> slice(a, b+1, c+1)
        ("mvindex", 3) => {
            let x: Expr = map_arg(&args[0])?;
            let start: i64 = map_arg(&args[1])?;
            let end: i64 = map_arg(&args[2])?;
            let length = end - start + 1;
            Ok(
                column_like!([slice([x], [py_lit(start + 1)], [py_lit(length)])].alias("mvindex"))
                    .into(),
            )
        }

        // mvappend(a, b) -> concat(a, b)
        // ("mvappend", 2) => {
        //     let left: Expr = map_arg(&args[0])?;
        //     let right: Expr = map_arg(&args[1])?;
        //     Ok(column_like!([concat([left], [right])].alias("mvappend")).into())
        // }

        // mvfilter(condition_about_d) -> filter(d, lambda d_: condition)
        // TODO
        // ast::Expr::Call(ast::Call { name, args }) if name == "mvfilter" && args.len() == 2 => {
        //     let condition: Expr = map_arg(&args[0])?;
        //     let lambda: Expr = map_arg(&args[1])?;
        //     Ok(column_like!([filter([condition], [lambda])].alias("mvfilter")).into())
        // },

        // strftime(d, c_fmt) -> date_format(d, spark_fmt)
        ("strftime", 2) => {
            let date: Expr = map_arg(&args[0])?;
            let format: String = map_arg(&args[1])?;
            let format = convert_time_format(format);
            Ok(column_like!([date_format([date], [py_lit(format)])].alias("strftime")).into())
        }

        // min(d, val) -> least(d, val)
        // ("min", 2) => {
        //     let column: Expr = map_arg(&args[0])?;
        //     let value: Expr = map_arg(&args[1])?;
        //     Ok(column_like!([least([column], [value])].alias("min")).into())
        // }

        // max(d, val) -> greatest(d, val)
        // ("max", 2) => {
        //     let column: Expr = map_arg(&args[0])?;
        //     let value: Expr = map_arg(&args[1])?;
        //     Ok(column_like!([greatest([column], [value])].alias("max")).into())
        // }

        // round(f, n) -> round(f, n)
        ("round", 2) => {
            let column: Expr = map_arg(&args[0])?;
            let precision: i64 = map_arg(&args[1])?;
            Ok(column_like!([round([column], [py_lit(precision)])].alias("round")).into())
        }

        // substr(s, start, len) -> substring(s, start, len)
        ("substr", 3) => {
            let column: Expr = map_arg(&args[0])?;
            let start: i64 = map_arg(&args[1])?;
            let length: i64 = map_arg(&args[2])?;
            Ok(column_like!(
                [substring([column], [py_lit(start)], [py_lit(length)])].alias("substr")
            )
            .into())
        }

        // cidrmatch(cidr, c) => expr("cidr_match({}, {})")
        ("cidrmatch", 2) => {
            let cidr: String = map_arg(&args[0])?;
            let col: String = map_arg(&args[1])?;
            Ok(column_like!(
                [expr([py_lit(format!("cidr_match('{}', {})", cidr, col))])].alias("cidrmatch")
            )
            .into())
        }

        // memk(v) => ...
        // Converts a string like "10k", "3g" or "7" (implicit "k") to kilobytes
        ("memk", 1) => {
            let col: Expr = map_arg(&args[0])?;
            const REGEX: &str = r#"(?i)^(\d*\.?\d+)([kmg])$"#;
            let num_val = column_like!(regexp_extract([col.clone()], [py_lit(REGEX)], [py_lit(1)]));
            let num_val = column_like!([num_val].cast([py_lit("double")]));
            let unit = column_like!(upper([regexp_extract(
                [col.clone()],
                [py_lit(REGEX)],
                [py_lit(2)]
            )]));
            let is_k = column_like!([unit.clone()] == [lit("K")]);
            let is_m = column_like!([unit.clone()] == [lit("M")]);
            let is_g = column_like!([unit.clone()] == [lit("G")]);
            let scaling_factor = column_like!(when([is_k], [lit(1.0)]));
            let scaling_factor = column_like!([scaling_factor].when([is_m], [lit(1024.0)]));
            let scaling_factor = column_like!([scaling_factor].when([is_g], [lit(1048576.0)]));
            let scaling_factor = column_like!([scaling_factor].otherwise([lit(1.0)]));
            Ok(column_like!([[num_val] * [scaling_factor]].alias("memk")).into())
        }

        // rmunit(s) => ...
        // Given a string like "10k", "3g", or "7", return just the numeric component
        ("rmunit", 1) => {
            let col: Expr = map_arg(&args[0])?;
            const REGEX: &str = r#"(?i)^(\d*\.?\d+)(\w*)$"#;
            let num_val = column_like!(regexp_extract([col.clone()], [py_lit(REGEX)], [py_lit(1)]));
            let num_val = column_like!([num_val].cast([py_lit("double")]));
            Ok(column_like!([num_val].alias("rmunit")).into())
        }

        // rmcomma(s) => ...
        // Given a string, remove all commas from it and cast it to a double
        ("rmcomma", 1) => {
            let col: Expr = map_arg(&args[0])?;
            Ok(column_like!(
                [[regexp_replace([col.clone()], [py_lit(",")], [py_lit("")])]
                    .cast([py_lit("double")])]
                .alias("rmcomma")
            )
            .into())
        }

        ("replace", 3) => {
            let input: Expr = map_arg(&args[0])?;
            let regex: String = map_arg(&args[1])?;
            let replacement: String = map_arg(&args[2])?;
            Ok(column_like!(regexp_replace(
                [input.clone()],
                [py_lit(regex)],
                [py_lit(replacement)]
            )))
        }

        // Fallback
        // (n, l) => bail!("Unsupported function: {}([{} args])", n, l),
        (name, _) => {
            let args: Vec<Expr> = map_args(args)?;
            let spark_name = SIMPLE_FUNC_MAP.get(name).cloned().unwrap_or(name);
            Ok(ColumnLike::Aliased {
                name: name.into(),
                col: Box::new(
                    ColumnLike::FunctionCall {
                        func: spark_name.to_string(),
                        args,
                    }
                    .into(),
                ),
            })
        }
    }
}
