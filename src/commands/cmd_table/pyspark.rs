use crate::commands::cmd_table::spl::TableCommand;
use crate::pyspark::transpiler::{PipelineTransformState, PipelineTransformer};

impl PipelineTransformer for TableCommand {
    fn transform(&self, state: PipelineTransformState) -> anyhow::Result<PipelineTransformState> {
        let df = state.df;

        unimplemented!();

        Ok(PipelineTransformState { df })
    }
}
