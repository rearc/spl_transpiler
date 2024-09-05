use crate::commands::cmd_stream_stats::spl::StreamStatsCommand;
use crate::pyspark::transpiler::{PipelineTransformState, PipelineTransformer};

impl PipelineTransformer for StreamStatsCommand {
    fn transform(&self, state: PipelineTransformState) -> anyhow::Result<PipelineTransformState> {
        let df = state.df;

        unimplemented!();

        Ok(PipelineTransformState { df })
    }
}
