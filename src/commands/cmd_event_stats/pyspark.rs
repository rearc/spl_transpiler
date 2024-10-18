use crate::commands::cmd_event_stats::spl::EventStatsCommand;
use crate::pyspark::transpiler::{PipelineTransformState, PipelineTransformer};
use anyhow::bail;

impl PipelineTransformer for EventStatsCommand {
    #[allow(unused_variables, unreachable_code)]
    fn transform_standalone(
        &self,
        state: PipelineTransformState,
    ) -> anyhow::Result<PipelineTransformState> {
        let df = state.df;

        bail!("UNIMPLEMENTED");

        Ok(PipelineTransformState { df })
    }
}
