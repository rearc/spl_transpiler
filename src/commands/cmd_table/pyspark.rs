use crate::commands::cmd_table::spl::TableCommand;
use crate::pyspark::ast::*;
use crate::pyspark::transpiler::{PipelineTransformState, PipelineTransformer};

impl PipelineTransformer for TableCommand {
    fn transform(&self, state: PipelineTransformState) -> anyhow::Result<PipelineTransformState> {
        let mut df = state.df;

        df = df.select(
            self.fields
                .iter()
                .map(|field| column_like!(col(field.0.clone())))
                .collect(),
        );

        Ok(PipelineTransformState { df })
    }
}
