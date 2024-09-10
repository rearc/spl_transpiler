use crate::pyspark::ast::DataFrame::Source;
use crate::pyspark::ast::*;
use crate::spl::ast;
use anyhow::{anyhow, Result};
pub(crate) mod expr;
pub(crate) mod utils;

pub struct PipelineTransformState {
    pub df: DataFrame,
}

pub trait PipelineTransformer {
    fn transform(&self, state: PipelineTransformState) -> Result<PipelineTransformState>;
}

impl PipelineTransformer for ast::Command {
    fn transform(&self, state: PipelineTransformState) -> Result<PipelineTransformState> {
        match self {
            ast::Command::AddTotals(command) => command.transform(state),
            ast::Command::BinCommand(command) => command.transform(state),
            ast::Command::CollectCommand(command) => command.transform(state),
            ast::Command::ConvertCommand(command) => command.transform(state),
            ast::Command::DedupCommand(command) => command.transform(state),
            ast::Command::EvalCommand(command) => command.transform(state),
            ast::Command::EventStatsCommand(command) => command.transform(state),
            ast::Command::FieldsCommand(command) => command.transform(state),
            ast::Command::FillNullCommand(command) => command.transform(state),
            ast::Command::FormatCommand(command) => command.transform(state),
            ast::Command::HeadCommand(command) => command.transform(state),
            ast::Command::InputLookup(command) => command.transform(state),
            ast::Command::JoinCommand(command) => command.transform(state),
            ast::Command::LookupCommand(command) => command.transform(state),
            ast::Command::MakeResults(command) => command.transform(state),
            ast::Command::MapCommand(command) => command.transform(state),
            ast::Command::MultiSearch(command) => command.transform(state),
            ast::Command::MvCombineCommand(command) => command.transform(state),
            ast::Command::MvExpandCommand(command) => command.transform(state),
            ast::Command::RegexCommand(command) => command.transform(state),
            ast::Command::RenameCommand(command) => command.transform(state),
            ast::Command::ReturnCommand(command) => command.transform(state),
            ast::Command::RexCommand(command) => command.transform(state),
            ast::Command::SearchCommand(command) => command.transform(state),
            ast::Command::SortCommand(command) => command.transform(state),
            ast::Command::StatsCommand(command) => command.transform(state),
            ast::Command::StreamStatsCommand(command) => command.transform(state),
            ast::Command::TableCommand(command) => command.transform(state),
            ast::Command::TopCommand(command) => command.transform(state),
            ast::Command::WhereCommand(command) => command.transform(state),
            _ => Err(anyhow!("Unsupported command in pipeline: {:?}", self)),
        }
    }
}

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
