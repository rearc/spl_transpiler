use crate::ast::ast;
use crate::pyspark::ast::DataFrame::Source;
use crate::pyspark::ast::*;
use anyhow::Result;
use base::{PipelineTransformState, PipelineTransformer};
mod base;
mod command;
mod expr;
mod utils;

impl TryFrom<ast::Pipeline> for TransformedPipeline {
    type Error = anyhow::Error;

    fn try_from(value: ast::Pipeline) -> Result<Self> {
        let mut state = PipelineTransformState {
            df: Source {
                name: "main".to_string(),
            },
        };
        for command in value.commands {
            state = command.transform(state)?;
        }
        Ok(TransformedPipeline {
            dataframes: vec![state.df],
        })
    }
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::pyspark::ast::column_like;

    #[test]
    fn test_field() {
        assert_eq!(
            // Expr::Column(ColumnLike::Named { name: "x".to_string() }),
            Expr::Column(column_like!(col("x"))),
            ast::Expr::Leaf(ast::LeafExpr::Constant(ast::Constant::Field(
                ast::Field::from("x")
            )))
            .try_into()
            .unwrap(),
        )
    }

    #[test]
    fn test_binary_op() {
        assert_eq!(
            Expr::Column(ColumnLike::BinaryOp {
                left: Box::new(Expr::Column(ColumnLike::Named {
                    name: "x".to_string()
                })),
                op: ">".to_string(),
                right: Box::new(Expr::Column(ColumnLike::Named {
                    name: "y".to_string()
                })),
            }),
            ast::Expr::Binary(ast::Binary {
                left: Box::new(ast::Expr::Leaf(ast::LeafExpr::Constant(
                    ast::Constant::Field(ast::Field::from("x"))
                ))),
                symbol: ">".to_string(),
                right: Box::new(ast::Expr::Leaf(ast::LeafExpr::Constant(
                    ast::Constant::Field(ast::Field::from("y"))
                ))),
            })
            .try_into()
            .unwrap(),
        )
    }

    #[test]
    fn test_field_in_0() {
        assert_eq!(
            Expr::Column(ColumnLike::Literal {
                code: "True".to_string()
            }),
            ast::Expr::FieldIn(ast::FieldIn {
                field: "junk".to_string(),
                exprs: vec![]
            })
            .try_into()
            .unwrap()
        );
    }

    #[test]
    fn test_field_in_1_wildcard() {
        assert_eq!(
            Expr::Column(column_like!([col("c")].like([py_lit("abc%")]))),
            ast::Expr::FieldIn(ast::FieldIn {
                field: "c".to_string(),
                exprs: vec![ast::Wildcard::from("abc*").into()]
            })
            .try_into()
            .unwrap()
        );
    }

    #[test]
    fn test_field_in_1_exact() {
        assert_eq!(
            Expr::Column(column_like!([col("c")] == [py_lit("abc")])),
            ast::Expr::FieldIn(ast::FieldIn {
                field: "c".to_string(),
                exprs: vec![ast::StrValue::from("abc").into()]
            })
            .try_into()
            .unwrap()
        );
    }

    #[test]
    fn test_field_in_2() {
        assert_eq!(
            Expr::Column(column_like!(
                [[col("c")] == [py_lit("abc")]] | [[col("c")].like([py_lit("abc%")])]
            )),
            ast::Expr::FieldIn(ast::FieldIn {
                field: "c".to_string(),
                exprs: vec![
                    ast::StrValue::from("abc").into(),
                    ast::Wildcard::from("abc*").into(),
                ]
            })
            .try_into()
            .unwrap()
        );
    }

    #[test]
    fn test_field_in_3() {
        assert_eq!(
            Expr::Column(column_like!(
                [[[col("c")] == [py_lit("abc")]] | [[col("c")].like([py_lit("abc%")])]]
                    | [[col("c")] == [py_lit("xyz")]]
            )),
            ast::Expr::FieldIn(ast::FieldIn {
                field: "c".to_string(),
                exprs: vec![
                    ast::StrValue::from("abc").into(),
                    ast::Wildcard::from("abc*").into(),
                    ast::StrValue::from("xyz").into(),
                ]
            })
            .try_into()
            .unwrap()
        );
    }
}
