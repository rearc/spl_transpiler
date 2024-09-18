use super::spl::*;
use crate::pyspark::ast::*;
use crate::pyspark::transpiler::{PipelineTransformState, PipelineTransformer};

impl PipelineTransformer for SPathCommand {
    fn transform(&self, state: PipelineTransformState) -> anyhow::Result<PipelineTransformState> {
        let mut df = state.df;

        let output_name = self.output.clone().unwrap_or(self.path.clone());

        df = df.with_column(
            output_name,
            column_like!(get_json_object(
                [col(self.input.clone())],
                [py_lit(format!("$.{}", self.path.clone()))]
            )),
        );

        Ok(PipelineTransformState { df })
    }
}
