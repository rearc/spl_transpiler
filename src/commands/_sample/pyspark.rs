use super::spl::*;
use crate::ast::ast;
use crate::pyspark::ast::*;
use crate::pyspark::transpiler::{PipelineTransformState, PipelineTransformer};
use anyhow::bail;

impl PipelineTransformer for SAMPLECommand {
    fn transform(&self, state: PipelineTransformState) -> anyhow::Result<PipelineTransformState> {
        let mut df = state.df;

        unimplemented!();

        Ok(PipelineTransformState { df })
    }
}
