use crate::pyspark::ast::*;
use crate::pyspark::transpiler::{PipelineTransformState, PipelineTransformer};

impl PipelineTransformer for super::spl::MultiSearchCommand {
    fn transform_standalone(
        &self,
        state: PipelineTransformState,
    ) -> anyhow::Result<PipelineTransformState> {
        if self.pipelines.is_empty() {
            return Ok(state);
        }

        let mut cur_df = None;

        for pipeline in self.pipelines.iter() {
            let transformed_pipeline: TransformedPipeline =
                TransformedPipeline::transform(pipeline.clone(), false)?;
            let other_df: DataFrame = transformed_pipeline.try_into()?;
            cur_df = match (cur_df, other_df) {
                (None, other) => Some(other),
                (Some(ref mut cur_df), other) => Some(cur_df.union_by_name(other)),
            };
        }

        Ok(PipelineTransformState {
            df: cur_df.unwrap(),
        })
    }
}
