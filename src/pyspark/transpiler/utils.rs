use crate::pyspark::ast::*;

pub fn join_as_binaries(
    op: impl ToString,
    exprs: Vec<ColumnLike>,
    default: ColumnLike,
) -> ColumnLike {
    match exprs.len() {
        0 => default,
        1 => exprs[0].clone(),
        2 => ColumnLike::BinaryOp {
            op: op.to_string(),
            left: Box::new(exprs[0].clone().into()),
            right: Box::new(exprs[1].clone().into()),
        },
        _ => {
            let mut left = exprs[0].clone();
            for check in &exprs[1..] {
                left = ColumnLike::BinaryOp {
                    left: Box::new(left.into()),
                    op: op.to_string(),
                    right: Box::new(check.clone().into()),
                };
            }
            left
        }
    }
}

pub fn convert_time_format(spl_time_format: impl ToString) -> String {
    spl_time_format
        .to_string()
        .replace("%Y", "yyyy")
        .replace("%m", "MM")
        .replace("%d", "dd")
        .replace("%T", "HH:mm:ss")
}
