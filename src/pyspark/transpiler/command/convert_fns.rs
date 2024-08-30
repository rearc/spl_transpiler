use crate::ast::ast;
use crate::pyspark::ast::*;
use crate::pyspark::transpiler::utils::convert_time_format;
use anyhow::bail;

pub fn convert_fn(
    cmd: &ast::ConvertCommand,
    conversion: &ast::FieldConversion,
) -> anyhow::Result<ColumnLike> {
    let ast::ConvertCommand { timeformat, .. } = cmd;
    let timeformat = convert_time_format(timeformat);
    let ast::FieldConversion {
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
