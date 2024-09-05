use crate::ast::ast;
use crate::pyspark::transpiler::{PipelineTransformState, PipelineTransformer};

impl PipelineTransformer for ast::SortCommand {
    fn transform(&self, state: PipelineTransformState) -> anyhow::Result<PipelineTransformState> {
        let mut df = state.df;

        unimplemented!();

        Ok(PipelineTransformState { df })
    }
}
