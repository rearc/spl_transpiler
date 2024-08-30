use crate::pyspark::ast::DataFrame;

pub struct PipelineTransformState {
    pub df: DataFrame,
}

pub trait PipelineTransformer {
    fn transform(&self, state: PipelineTransformState) -> anyhow::Result<PipelineTransformState>;
}
