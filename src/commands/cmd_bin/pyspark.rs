use super::spl::*;
use crate::pyspark::ast::*;
use crate::pyspark::transpiler::{PipelineTransformState, PipelineTransformer};
use crate::spl::ast;
use crate::spl::ast::FieldOrAlias;

impl PipelineTransformer for BinCommand {
    fn transform(&self, state: PipelineTransformState) -> anyhow::Result<PipelineTransformState> {
        let mut df = state.df;
        let col_name = match self.field.clone() {
            FieldOrAlias::Field(ast::Field(name)) => name,
            FieldOrAlias::Alias(ast::Alias { name, .. }) => name,
        };
        if let Some(ast::TimeSpan { value, scale }) = self.span.clone() {
            let span = format!("{} {}", value, scale);
            df = df.with_column(
                col_name.clone(),
                column_like!(window([col("n")], [py_lit(span)])),
            );
        }
        let subfield = "start";
        df = df.with_column(
            col_name.clone(),
            column_like!(col(format!("{}.{}", col_name, subfield))),
        );
        Ok(PipelineTransformState { df })
    }
}
