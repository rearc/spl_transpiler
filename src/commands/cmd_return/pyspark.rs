use crate::commands::cmd_return::spl::ReturnCommand;
use crate::pyspark::transpiler::{PipelineTransformState, PipelineTransformer};
use anyhow::bail;

impl PipelineTransformer for ReturnCommand {
    #[allow(unused_variables, unreachable_code)]
    fn transform_standalone(
        &self,
        state: PipelineTransformState,
    ) -> anyhow::Result<PipelineTransformState> {
        let df = state.df.clone().unwrap_or_default();

        bail!("UNIMPLEMENTED");

        Ok(state.with_df(df))
    }
}
