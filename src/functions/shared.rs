use crate::pyspark::ast::*;

pub fn ctime(c: impl Into<Expr>, timeformat: String) -> ColumnLike {
    column_like!(date_format([c.into()], [py_lit(timeformat)]))
}

/// Converts a string like "10k", "3g" or "7" (implicit "k") to kilobytes
pub fn memk(c: impl Into<Expr>) -> ColumnLike {
    let c = c.into();
    const REGEX: &str = r#"(?i)^(\d*\.?\d+)([kmg])$"#;
    let num_val = column_like!(regexp_extract([c.clone()], [py_lit(REGEX)], [py_lit(1)]));
    let num_val = column_like!([num_val].cast([py_lit("double")]));
    let unit = column_like!(upper([regexp_extract([c], [py_lit(REGEX)], [py_lit(2)])]));
    let is_k = column_like!([unit.clone()] == [lit("K")]);
    let is_m = column_like!([unit.clone()] == [lit("M")]);
    let is_g = column_like!([unit.clone()] == [lit("G")]);
    let scaling_factor = column_like!(when([is_k], [lit(1.0)]));
    let scaling_factor = column_like!([scaling_factor].when([is_m], [lit(1024.0)]));
    let scaling_factor = column_like!([scaling_factor].when([is_g], [lit(1048576.0)]));
    let scaling_factor = column_like!([scaling_factor].otherwise([lit(1.0)]));
    column_like!([num_val] * [scaling_factor])
}

/// Given a string, remove all commas from it and cast it to a double
pub fn rmcomma(c: impl Into<Expr>) -> ColumnLike {
    column_like!([regexp_replace([c.into()], [py_lit(",")], [py_lit("")])].cast([py_lit("double")]))
}

/// Given a string like "10k", "3g", or "7", return just the numeric component
pub fn rmunit(c: impl Into<Expr>) -> ColumnLike {
    const REGEX: &str = r#"(?i)^(\d*\.?\d+)(\w*)$"#;
    column_like!([regexp_extract([c.into()], [py_lit(REGEX)], [py_lit(1)])].cast([py_lit("double")]))
}
