use super::spl::*;
use crate::functions::convert_fns::convert_fn;
use crate::pyspark::transpiler::{PipelineTransformState, PipelineTransformer};
use crate::spl::ast;

impl PipelineTransformer for ConvertCommand {
    fn transform(&self, state: PipelineTransformState) -> anyhow::Result<PipelineTransformState> {
        let mut df = state.df;

        for conv in self.convs.iter().cloned() {
            let result = convert_fn(self, &conv)?;
            let FieldConversion {
                field: ast::Field(name),
                alias,
                ..
            } = conv;
            let name = alias.map(|f| f.0).unwrap_or(name);
            df = df.with_column(name, result)
        }
        Ok(PipelineTransformState { df })
    }
}
