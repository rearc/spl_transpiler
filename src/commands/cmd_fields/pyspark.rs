use super::spl::*;
use crate::pyspark::ast::*;
use crate::pyspark::transpiler::{PipelineTransformState, PipelineTransformer};
use crate::spl::ast;

impl PipelineTransformer for FieldsCommand {
    fn transform(&self, state: PipelineTransformState) -> anyhow::Result<PipelineTransformState> {
        let mut df = state.df;

        let cols: Vec<_> = self
            .fields
            .iter()
            .map(|ast::Field(name)| column_like!(col(name)))
            .collect();

        df = df.select(cols);

        Ok(PipelineTransformState { df })
    }
}
