use crate::commands::cmd_dedup::spl::DedupCommand;
use crate::pyspark::transpiler::{PipelineTransformState, PipelineTransformer};
use anyhow::bail;

impl PipelineTransformer for DedupCommand {
    #[allow(unused_variables, unreachable_code)]
    fn transform(&self, state: PipelineTransformState) -> anyhow::Result<PipelineTransformState> {
        let df = state.df;

        bail!("UNIMPLEMENTED");

        Ok(PipelineTransformState { df })
    }
}
