use crate::pyspark::ast::*;
use crate::pyspark::base::{PysparkTranspileContext, RuntimeSelection};
use crate::spl::ast;
use anyhow::{anyhow, bail, Result};
use std::fmt::Debug;

pub(crate) mod expr;
pub(crate) mod utils;

#[derive(Debug, Clone)]
pub struct PipelineTransformState {
    pub ctx: PysparkTranspileContext,
    pub df: Option<DataFrame>,
}

impl PipelineTransformState {
    pub fn new(ctx: PysparkTranspileContext) -> Self {
        Self { ctx, df: None }
    }

    pub fn with_df(self, df: DataFrame) -> Self {
        Self {
            ctx: self.ctx,
            df: Some(df),
        }
    }
}

pub trait PipelineTransformer: Debug {
    fn transform(&self, state: PipelineTransformState) -> Result<PipelineTransformState> {
        match state.ctx.runtime {
            RuntimeSelection::Disallow => self.transform_standalone(state),
            RuntimeSelection::Allow => self
                .transform_for_runtime(state.clone())
                .or_else(|_| self.transform_standalone(state)),
            RuntimeSelection::Require => self.transform_for_runtime(state.clone()),
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
    fn transform(&self, state: PipelineTransformState) -> Result<PipelineTransformState> {
        #[allow(unreachable_patterns)]
        match self {
            ast::Command::AddTotalsCommand(command) => command.transform(state),
            ast::Command::BinCommand(command) => command.transform(state),
            ast::Command::CollectCommand(command) => command.transform(state),
            ast::Command::ConvertCommand(command) => command.transform(state),
            ast::Command::DataModelCommand(command) => command.transform(state),
            ast::Command::DedupCommand(command) => command.transform(state),
            ast::Command::EvalCommand(command) => command.transform(state),
            ast::Command::EventStatsCommand(command) => command.transform(state),
            ast::Command::FieldsCommand(command) => command.transform(state),
            ast::Command::FillNullCommand(command) => command.transform(state),
            ast::Command::FormatCommand(command) => command.transform(state),
            ast::Command::HeadCommand(command) => command.transform(state),
            ast::Command::InputLookupCommand(command) => command.transform(state),
            ast::Command::JoinCommand(command) => command.transform(state),
            ast::Command::LookupCommand(command) => command.transform(state),
            ast::Command::MakeResultsCommand(command) => command.transform(state),
            ast::Command::MapCommand(command) => command.transform(state),
            ast::Command::MultiSearchCommand(command) => command.transform(state),
            ast::Command::MvCombineCommand(command) => command.transform(state),
            ast::Command::MvExpandCommand(command) => command.transform(state),
            ast::Command::RareCommand(command) => command.transform(state),
            ast::Command::RegexCommand(command) => command.transform(state),
            ast::Command::RenameCommand(command) => command.transform(state),
            ast::Command::ReturnCommand(command) => command.transform(state),
            ast::Command::RexCommand(command) => command.transform(state),
            ast::Command::SearchCommand(command) => command.transform(state),
            ast::Command::SortCommand(command) => command.transform(state),
            ast::Command::SPathCommand(command) => command.transform(state),
            ast::Command::StatsCommand(command) => command.transform(state),
            ast::Command::StreamStatsCommand(command) => command.transform(state),
            ast::Command::TableCommand(command) => command.transform(state),
            ast::Command::TailCommand(command) => command.transform(state),
            ast::Command::TopCommand(command) => command.transform(state),
            ast::Command::TStatsCommand(command) => command.transform(state),
            ast::Command::WhereCommand(command) => command.transform(state),
            _ => Err(anyhow!("Unsupported command in pipeline: {:?}", self)),
        }
    }
}

impl TransformedPipeline {
    pub fn transform(value: ast::Pipeline, ctx: PysparkTranspileContext) -> Result<Self> {
        let mut state = PipelineTransformState::new(ctx);
        for command in value.commands {
            state = command.transform(state)?;
        }
        Ok(TransformedPipeline {
            dataframes: state.df.map_or_else(Vec::new, |df| vec![df]),
        })
    }
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::pyspark::ast::column_like;
    use crate::pyspark::base::ToSparkExpr;
    use rstest::rstest;

    #[rstest]
    fn test_field(#[from(crate::pyspark::base::test::ctx_bare)] ctx: PysparkTranspileContext) {
        assert_eq!(
            // Expr::Column(ColumnLike::Named { name: "x".to_string() }),
            Expr::Column(column_like!(col("x"))),
            ast::Expr::Leaf(ast::LeafExpr::Constant(ast::Constant::Field(
                ast::Field::from("x")
            )))
            .with_context(&ctx)
            .try_into()
            .unwrap(),
        )
    }

    #[rstest]
    fn test_binary_op(#[from(crate::pyspark::base::test::ctx_bare)] ctx: PysparkTranspileContext) {
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
            .with_context(&ctx)
            .try_into()
            .unwrap(),
        )
    }

    #[rstest]
    fn test_field_in_0(#[from(crate::pyspark::base::test::ctx_bare)] ctx: PysparkTranspileContext) {
        assert_eq!(
            Expr::Column(ColumnLike::Literal {
                code: "True".to_string()
            }),
            ast::Expr::FieldIn(ast::FieldIn {
                field: "junk".to_string(),
                exprs: vec![]
            })
            .with_context(&ctx)
            .try_into()
            .unwrap()
        );
    }

    #[rstest]
    fn test_field_in_1_wildcard(
        #[from(crate::pyspark::base::test::ctx_bare)] ctx: PysparkTranspileContext,
    ) {
        assert_eq!(
            Expr::Column(column_like!([col("c")].like([py_lit("abc%")]))),
            ast::Expr::FieldIn(ast::FieldIn {
                field: "c".to_string(),
                exprs: vec![ast::Wildcard::from("abc*").into()]
            })
            .with_context(&ctx)
            .try_into()
            .unwrap()
        );
    }

    #[rstest]
    fn test_field_in_1_exact(
        #[from(crate::pyspark::base::test::ctx_bare)] ctx: PysparkTranspileContext,
    ) {
        assert_eq!(
            Expr::Column(column_like!([col("c")] == [py_lit("abc")])),
            ast::Expr::FieldIn(ast::FieldIn {
                field: "c".to_string(),
                exprs: vec![ast::StrValue::from("abc").into()]
            })
            .with_context(&ctx)
            .try_into()
            .unwrap()
        );
    }

    #[rstest]
    fn test_field_in_2(#[from(crate::pyspark::base::test::ctx_bare)] ctx: PysparkTranspileContext) {
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
            .with_context(&ctx)
            .try_into()
            .unwrap()
        );
    }

    #[rstest]
    fn test_field_in_3(#[from(crate::pyspark::base::test::ctx_bare)] ctx: PysparkTranspileContext) {
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
            .with_context(&ctx)
            .try_into()
            .unwrap()
        );
    }
}
