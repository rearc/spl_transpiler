use crate::commands::cmd_tail::spl::TailCommand;
use crate::pyspark::transpiler::{PipelineTransformState, PipelineTransformer};

impl PipelineTransformer for TailCommand {
    fn transform_standalone(
        &self,
        state: PipelineTransformState,
    ) -> anyhow::Result<PipelineTransformState> {
        let mut df = state.df;

        df = df.tail(self.n);

        Ok(PipelineTransformState { df })
    }
}
