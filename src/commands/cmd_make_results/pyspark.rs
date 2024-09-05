use crate::pyspark::transpiler::{PipelineTransformState, PipelineTransformer};

impl PipelineTransformer for super::spl::MakeResults {
    fn transform(&self, state: PipelineTransformState) -> anyhow::Result<PipelineTransformState> {
        let df = state.df;

        unimplemented!();

        Ok(PipelineTransformState { df })
    }
}
