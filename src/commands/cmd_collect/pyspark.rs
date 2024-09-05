use super::spl::*;
use crate::pyspark::transpiler::{PipelineTransformState, PipelineTransformer};

impl PipelineTransformer for CollectCommand {
    fn transform(&self, state: PipelineTransformState) -> anyhow::Result<PipelineTransformState> {
        let df = state.df;

        unimplemented!();

        Ok(PipelineTransformState { df })
    }
}
