use crate::commands::cmd_join::spl::JoinCommand;
use crate::pyspark::transpiler::{PipelineTransformState, PipelineTransformer};
use anyhow::bail;

impl PipelineTransformer for JoinCommand {
    #[allow(unused_variables, unreachable_code)]
    fn transform(&self, state: PipelineTransformState) -> anyhow::Result<PipelineTransformState> {
        let df = state.df;

        bail!("UNIMPLEMENTED");

        Ok(PipelineTransformState { df })
    }
}
