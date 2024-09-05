use super::spl::*;
use crate::ast::ast;
use crate::pyspark::ast::*;
use crate::pyspark::transpiler::{PipelineTransformState, PipelineTransformer};
use anyhow::bail;

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

pub fn convert_fn(
    cmd: &ConvertCommand,
    conversion: &FieldConversion,
) -> anyhow::Result<ColumnLike> {
    let ConvertCommand { timeformat, .. } = cmd;
    let timeformat = crate::pyspark::transpiler::utils::convert_time_format(timeformat);
    let FieldConversion {
        func,
        field: ast::Field(field_name),
        ..
    } = conversion;

    match func.as_str() {
        "ctime" => Ok(column_like!(date_format(
            [col(field_name)],
            [py_lit(timeformat)]
        ))),

        _ => bail!("Unsupported conversion function: {}", func),
    }
}
