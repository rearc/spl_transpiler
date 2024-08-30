use crate::ast::ast;
use crate::pyspark::transpiler::base::{PipelineTransformState, PipelineTransformer};
use crate::pyspark::transpiler::command::convert_fns::convert_fn;

impl PipelineTransformer for ast::ConvertCommand {
    fn transform(&self, state: PipelineTransformState) -> anyhow::Result<PipelineTransformState> {
        let mut df = state.df;

        for conv in self.convs.iter().cloned() {
            let result = convert_fn(self, &conv)?;
            let ast::FieldConversion {
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
