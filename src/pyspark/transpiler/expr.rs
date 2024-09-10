use crate::functions::eval_fns::eval_fn;
use crate::pyspark::ast::*;
use crate::pyspark::transpiler::utils::join_as_binaries;
use crate::spl::ast;
use anyhow::{anyhow, bail};
use phf::phf_map;

static SIMPLE_OP_MAP: phf::Map<&'static str, &'static str> = phf_map! {
    "=" => "==",
    "AND" => "&",
    "OR" => "|",
    "NOT" => "~",
};

impl TryFrom<ast::Expr> for Expr {
    type Error = anyhow::Error;

    fn try_from(expr: ast::Expr) -> anyhow::Result<Self> {
        match expr {
            // Binary operation -> Binary op
            ast::Expr::Binary(ast::Binary {
                left,
                symbol,
                right,
            }) => match (*left, symbol.as_str(), *right) {
                // src_ip = 10.0.0.0/16 -> F.expr("cidr_match('10.0.0.0/16', src_ip)",
                (
                    ast::Expr::Leaf(ast::LeafExpr::Constant(ast::Constant::Field(ast::Field(col)))),
                    "=",
                    ast::Expr::Leaf(ast::LeafExpr::Constant(ast::Constant::IPv4CIDR(
                        ast::IPv4CIDR(cidr),
                    ))),
                ) => Ok(
                    column_like!(expr([py_lit(format!("cidr_match('{}', {})", cidr, col))])).into(),
                ),
                // a [op] b -> a [op] b
                (left, op, right) => Ok(ColumnLike::BinaryOp {
                    left: Box::new(left.try_into()?),
                    op: SIMPLE_OP_MAP.get(op).cloned().unwrap_or(op).to_string(),
                    right: Box::new(right.try_into()?),
                }
                .into()),
            },

            // Unary operation -> Unary op
            ast::Expr::Unary(ast::Unary { symbol, right }) if symbol == "NOT" => {
                Ok(ColumnLike::UnaryNot {
                    right: Box::new((*right).try_into()?),
                }
                .into())
            }

            // Field -> Named column
            ast::Expr::Leaf(ast::LeafExpr::Constant(ast::Constant::Field(ast::Field(name)))) => {
                Ok(ColumnLike::Named { name: name.clone() }.into())
            }

            // Int constant -> Int literal column
            ast::Expr::Leaf(ast::LeafExpr::Constant(ast::Constant::Int(ast::IntValue(val)))) => {
                Ok(ColumnLike::Literal {
                    code: val.to_string(),
                }
                .into())
            }

            // Double constant -> Double literal column
            ast::Expr::Leaf(ast::LeafExpr::Constant(ast::Constant::Double(ast::DoubleValue(
                val,
            )))) => Ok(ColumnLike::Literal {
                code: val.to_string(),
            }
            .into()),

            // String constant -> String literal column
            ast::Expr::Leaf(ast::LeafExpr::Constant(ast::Constant::Str(ast::StrValue(val)))) => {
                Ok(ColumnLike::Literal {
                    code: format!("'{}'", val),
                }
                .into())
            }

            // Boolean constant -> Boolean literal column
            ast::Expr::Leaf(ast::LeafExpr::Constant(ast::Constant::Bool(ast::BoolValue(val)))) => {
                Ok(ColumnLike::Literal {
                    code: val.to_string(),
                }
                .into())
            }

            // IPv4 cidr -> String literal column
            ast::Expr::Leaf(ast::LeafExpr::Constant(ast::Constant::IPv4CIDR(ast::IPv4CIDR(
                val,
            )))) => Ok(ColumnLike::Literal {
                code: format!("'{}'", val),
            }
            .into()),

            // 'x in ("a", "b*", c)' -> Binary op tree of OR's of individual checks
            // "a" -> { col("x") == "a" }
            // "b*" -> { col("x").like("b%") }
            // ...
            ast::Expr::FieldIn(ast::FieldIn { field, exprs }) => {
                let c: Expr = column_like!(col(field.clone())).into();
                let checks: anyhow::Result<Vec<ColumnLike>> = exprs
                    .iter()
                    .map(|expr| match expr {
                        // Handle wildcard case
                        ast::Expr::Leaf(ast::LeafExpr::Constant(const_)) => match const_ {
                            ast::Constant::Wildcard(ast::Wildcard(val)) => {
                                Ok(column_like!(
                                    [c.clone()].like([Raw::from(val.replace("*", "%"))])
                                ))
                            }
                            ast::Constant::Null(_) => {
                                bail!("Unimplemented value in field-in rhs: {:?}", const_)
                            }
                            ast::Constant::Bool(ast::BoolValue(val)) => {
                                Ok(column_like!([c.clone()] == [Raw::from(*val)]))
                            }
                            ast::Constant::Int(ast::IntValue(val)) => {
                                Ok(column_like!([c.clone()] == [Raw::from(*val)]))
                            }
                            ast::Constant::Double(ast::DoubleValue(val)) => {
                                Ok(column_like!([c.clone()] == [Raw::from(*val)]))
                            }
                            ast::Constant::Str(ast::StrValue(val)) => {
                                Ok(column_like!([c.clone()] == [Raw::from(val.clone())]))
                            }
                            ast::Constant::SnapTime(ast::SnapTime { .. }) => {
                                bail!("Unimplemented value in field-in rhs: {:?}", const_)
                            }
                            ast::Constant::SplSpan(ast::SplSpan::TimeSpan(ast::TimeSpan {
                                ..
                            })) => bail!("Unimplemented value in field-in rhs: {:?}", const_),
                            ast::Constant::Field(ast::Field(val)) => {
                                Ok(column_like!([c.clone()] == [Raw::from(val.clone())]))
                            }
                            ast::Constant::Variable(ast::Variable(_value)) => {
                                bail!("Unimplemented value in field-in rhs: {:?}", const_)
                            }
                            ast::Constant::IPv4CIDR(ast::IPv4CIDR(val)) => {
                                Ok(column_like!([c.clone()] == [Raw::from(val.clone())]))
                            }
                        },
                        // Handle exact case
                        _ => Err(anyhow!("Unsupported field-in rhs value: {:?}", expr)),
                    })
                    .collect();
                let checks = checks?;
                Ok(join_as_binaries("|", checks, column_like!(lit(true))).into())
                // Ok(match checks.len() {
                //     0 => column_like!(lit(true)).into(),
                //     1 => checks[0].clone().into(),
                //     2 => column_like!([checks[0].clone()] | [checks[1].clone()]).into(),
                //     _ => {
                //         let mut left = checks[0].clone();
                //         for check in &checks[1..] {
                //             left = column_like!([left] | [check.clone()]).into();
                //         }
                //         left
                //     }
                // }.into())
            }

            ast::Expr::Call(call @ ast::Call { .. }) => Ok(eval_fn(call)?.into()),

            _ => Err(anyhow!("Unsupported expression: {:?}", expr)),
        }
    }
}
