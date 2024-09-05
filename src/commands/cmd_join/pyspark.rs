use crate::commands::cmd_join::spl::JoinCommand;
use crate::pyspark::transpiler::{PipelineTransformState, PipelineTransformer};

impl PipelineTransformer for JoinCommand {
    fn transform(&self, state: PipelineTransformState) -> anyhow::Result<PipelineTransformState> {
        let df = state.df;

        unimplemented!();

        Ok(PipelineTransformState { df })
    }
}
