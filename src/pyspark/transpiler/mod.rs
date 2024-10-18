use crate::pyspark::ast::DataFrame::Source;
use crate::pyspark::ast::*;
use crate::spl::ast;
use anyhow::{anyhow, bail, Result};
use std::fmt::Debug;
pub(crate) mod expr;
pub(crate) mod utils;

#[derive(Debug, Clone)]
pub struct PipelineTransformState {
    pub df: DataFrame,
}

pub trait PipelineTransformer: Debug {
    fn transform(
        &self,
        state: PipelineTransformState,
        allow_runtime: bool,
    ) -> Result<PipelineTransformState> {
        if allow_runtime {
            self.transform_for_runtime(state.clone())
                .or_else(|_| self.transform_standalone(state))
        } else {
            self.transform_standalone(state)
        }
    }

    fn transform_standalone(
        &self,
        _state: PipelineTransformState,
    ) -> Result<PipelineTransformState> {
        bail!(
            "UNIMPLEMENTED: transform_standalone is not implemented for {:?}",
            self
        )
    }

    fn transform_for_runtime(
        &self,
        _state: PipelineTransformState,
    ) -> Result<PipelineTransformState> {
        bail!(
            "UNIMPLEMENTED: transform_standalone is not implemented for {:?}",
            self
        )
    }
}

impl PipelineTransformer for ast::Command {
    fn transform(
        &self,
        state: PipelineTransformState,
        allow_runtime: bool,
    ) -> Result<PipelineTransformState> {
        #[allow(unreachable_patterns)]
        match self {
            ast::Command::AddTotalsCommand(command) => command.transform(state, allow_runtime),
            ast::Command::BinCommand(command) => command.transform(state, allow_runtime),
            ast::Command::CollectCommand(command) => command.transform(state, allow_runtime),
            ast::Command::ConvertCommand(command) => command.transform(state, allow_runtime),
            ast::Command::DataModelCommand(command) => command.transform(state, allow_runtime),
            ast::Command::DedupCommand(command) => command.transform(state, allow_runtime),
            ast::Command::EvalCommand(command) => command.transform(state, allow_runtime),
            ast::Command::EventStatsCommand(command) => command.transform(state, allow_runtime),
            ast::Command::FieldsCommand(command) => command.transform(state, allow_runtime),
            ast::Command::FillNullCommand(command) => command.transform(state, allow_runtime),
            ast::Command::FormatCommand(command) => command.transform(state, allow_runtime),
            ast::Command::HeadCommand(command) => command.transform(state, allow_runtime),
            ast::Command::InputLookupCommand(command) => command.transform(state, allow_runtime),
            ast::Command::JoinCommand(command) => command.transform(state, allow_runtime),
            ast::Command::LookupCommand(command) => command.transform(state, allow_runtime),
            ast::Command::MakeResultsCommand(command) => command.transform(state, allow_runtime),
            ast::Command::MapCommand(command) => command.transform(state, allow_runtime),
            ast::Command::MultiSearchCommand(command) => command.transform(state, allow_runtime),
            ast::Command::MvCombineCommand(command) => command.transform(state, allow_runtime),
            ast::Command::MvExpandCommand(command) => command.transform(state, allow_runtime),
            ast::Command::RareCommand(command) => command.transform(state, allow_runtime),
            ast::Command::RegexCommand(command) => command.transform(state, allow_runtime),
            ast::Command::RenameCommand(command) => command.transform(state, allow_runtime),
            ast::Command::ReturnCommand(command) => command.transform(state, allow_runtime),
            ast::Command::RexCommand(command) => command.transform(state, allow_runtime),
            ast::Command::SearchCommand(command) => command.transform(state, allow_runtime),
            ast::Command::SortCommand(command) => command.transform(state, allow_runtime),
            ast::Command::SPathCommand(command) => command.transform(state, allow_runtime),
            ast::Command::StatsCommand(command) => command.transform(state, allow_runtime),
            ast::Command::StreamStatsCommand(command) => command.transform(state, allow_runtime),
            ast::Command::TableCommand(command) => command.transform(state, allow_runtime),
            ast::Command::TailCommand(command) => command.transform(state, allow_runtime),
            ast::Command::TopCommand(command) => command.transform(state, allow_runtime),
            ast::Command::TStatsCommand(command) => command.transform(state, allow_runtime),
            ast::Command::WhereCommand(command) => command.transform(state, allow_runtime),
            _ => Err(anyhow!("Unsupported command in pipeline: {:?}", self)),
        }
    }
}

impl TransformedPipeline {
    pub fn transform(value: ast::Pipeline, allow_runtime: bool) -> Result<Self> {
        let mut state = PipelineTransformState {
            df: Source {
                name: "main".to_string(),
            },
        };
        for command in value.commands {
            state = command.transform(state, allow_runtime)?;
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
            Expr::Column(ColumnLike::binary_op(
                Expr::Column(ColumnLike::named("x")),
                ">",
                Expr::Column(ColumnLike::named("y")),
            )),
            ast::Expr::Binary(ast::Binary::new(
                ast::Expr::Leaf(ast::LeafExpr::Constant(ast::Constant::Field(
                    ast::Field::from("x")
                ))),
                ">",
                ast::Expr::Leaf(ast::LeafExpr::Constant(ast::Constant::Field(
                    ast::Field::from("y")
                ))),
            ))
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
