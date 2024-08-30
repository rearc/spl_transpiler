use crate::ast::ast;
use pyo3::prelude::*;

macro_rules! impl_pyclass {
    ($name: ident ($($arg: ident : $arg_tp: ty),*)) => {
        #[pymethods]
        impl ast::$name {
            #[new]
            fn new($($arg: $arg_tp),*) -> Self {
                ast::$name ( $($arg),* )
            }
        }
    };
    ($name: ident {$($arg: ident : $arg_tp: ty),*}) => {
        #[pymethods]
        impl ast::$name {
            #[new]
            fn new($($arg: $arg_tp),*) -> Self {
                ast::$name { $($arg),* }
            }
        }
    };
    ($name: ident {$(|$arg: ident : $arg_tp: ty| $conv: expr),*}) => {
        #[pymethods]
        impl ast::$name {
            #[new]
            fn new($( $arg: $arg_tp ),*) -> Self {
                ast::$name { $( $arg : (|$arg: $arg_tp| $conv)($arg) ),* }
            }
        }
    };
}

// impl_pyclass!(NullValue ());
impl_pyclass!(BoolValue (value: bool));
impl_pyclass!(IntValue (value: i64));
impl_pyclass!(StrValue (value: String));
impl_pyclass!(DoubleValue (value: f64));
impl_pyclass!(SnapTime { span: Option<ast::TimeSpan>, snap: String, snap_offset: Option<ast::TimeSpan> });
impl_pyclass!(Field (name: String));
impl_pyclass!(Wildcard (value: String));
impl_pyclass!(Variable (value: String));
impl_pyclass!(IPv4CIDR (value: String));
impl_pyclass!(FV {
    field: String,
    value: String
});
impl_pyclass!(FB {
    field: String,
    value: bool
});
impl_pyclass!(FC {
    field: String,
    value: ast::Constant
});
impl_pyclass!(CommandOptions { options: Vec<ast::FC> });
impl_pyclass!(AliasedField {
    field: ast::Field,
    alias: String
});
impl_pyclass!(Binary { |left: ast::Expr| Box::new(left), |symbol: String| symbol, |right: ast::Expr| Box::new(right) });
impl_pyclass!(Unary { |symbol: String| symbol, |right: ast::Expr| Box::new(right) });
impl_pyclass!(Call { name: String, args: Vec<ast::Expr> });
impl_pyclass!(FieldIn { field: String, exprs: Vec<ast::Expr> });
impl_pyclass!(Alias { |expr: ast::Expr| Box::new(expr), |name: String| name });
impl_pyclass!(SearchCommand { expr: ast::Expr });
impl_pyclass!(EvalCommand { fields: Vec<(ast::Field, ast::Expr)> });
impl_pyclass!(FieldConversion { func: String, field: ast::Field, alias: Option<ast::Field> });
impl_pyclass!(ConvertCommand { timeformat: String, convs: Vec<ast::FieldConversion> });
impl_pyclass!(LookupOutput { kv: String, fields: Vec<ast::FieldLike> });
impl_pyclass!(LookupCommand { dataset: String, fields: Vec<ast::FieldLike>, output: Option<ast::LookupOutput> });
impl_pyclass!(CollectCommand { index: String, fields: Vec<ast::Field>, add_time: bool, file: Option<String>, host: Option<String>, marker: Option<String>, output_format: String, run_in_preview: bool, spool: bool, source: Option<String>, source_type: Option<String>, test_mode: bool });
impl_pyclass!(WhereCommand { expr: ast::Expr });
impl_pyclass!(TableCommand { fields: Vec<ast::Field> });
impl_pyclass!(HeadCommand {
    eval_expr: ast::Expr,
    keep_last: ast::BoolValue,
    null_option: ast::BoolValue
});
impl_pyclass!(FieldsCommand { remove_fields: bool, fields: Vec<ast::Field> });
impl_pyclass!(SortCommand { fields_to_sort: Vec<(Option<String>, ast::Expr)> });
impl_pyclass!(StatsCommand { partitions: i64, all_num: bool, delim: String, funcs: Vec<ast::Expr>, by: Vec<ast::Field>, dedup_split_vals: bool });
impl_pyclass!(RexCommand { field: Option<String>, max_match: i64, offset_field: Option<String>, mode: Option<String>, regex: String });
impl_pyclass!(RenameCommand { alias: Vec<ast::Alias> });
impl_pyclass!(RegexCommand { item: Option<(ast::Field, String)>, regex: String });
impl_pyclass!(JoinCommand { join_type: String, use_time: bool, earlier: bool, overwrite: bool, max: i64, fields: Vec<ast::Field>, sub_search: ast::Pipeline });
impl_pyclass!(ReturnCommand { count: ast::IntValue, fields: Vec<ast::FieldOrAlias> });
impl_pyclass!(FillNullCommand { value: Option<String>, fields: Option<Vec<ast::Field>> });
impl_pyclass!(EventStatsCommand { all_num: bool, funcs: Vec<ast::Expr>, by: Vec<ast::Field> });
impl_pyclass!(StreamStatsCommand { funcs: Vec<ast::Expr>, by: Vec<ast::Field>, current: bool, window: i64 });
impl_pyclass!(DedupCommand { num_results: i64, fields: Vec<ast::Field>, keep_events: bool, keep_empty: bool, consecutive: bool, sort_by: ast::SortCommand });
impl_pyclass!(InputLookup { append: bool, strict: bool, start: i64, max: i64, table_name: String, where_expr: Option<ast::Expr> });
impl_pyclass!(FormatCommand {
    mv_sep: String,
    max_results: i64,
    row_prefix: String,
    col_prefix: String,
    col_sep: String,
    col_end: String,
    row_sep: String,
    row_end: String
});
impl_pyclass!(MvCombineCommand { delim: Option<String>, field: ast::Field });
impl_pyclass!(MvExpandCommand { field: ast::Field, limit: Option<i64> });
impl_pyclass!(MakeResults { count: i64, annotate: bool, server: String, server_group: Option<String> });
impl_pyclass!(AddTotals { fields: Vec<ast::Field>, row: bool, col: bool, field_name: String, label_field: Option<String>, label: String });
impl_pyclass!(BinCommand { field: ast::FieldOrAlias, span: Option<ast::TimeSpan>, min_span: Option<ast::TimeSpan>, bins: Option<i64>, start: Option<i64>, end: Option<i64>, align_time: Option<String> });
impl_pyclass!(MultiSearch { pipelines: Vec<ast::Pipeline> });
impl_pyclass!(MapCommand {
    search: ast::Pipeline,
    max_searches: i64
});
impl_pyclass!(Pipeline { commands: Vec<ast::Command> });

pub fn ast(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<ast::NullValue>()?;
    m.add_class::<ast::BoolValue>()?;
    m.add_class::<ast::IntValue>()?;
    m.add_class::<ast::StrValue>()?;
    m.add_class::<ast::DoubleValue>()?;
    m.add_class::<ast::SnapTime>()?;
    m.add_class::<ast::Field>()?;
    m.add_class::<ast::Wildcard>()?;
    m.add_class::<ast::Variable>()?;
    m.add_class::<ast::IPv4CIDR>()?;
    m.add_class::<ast::FV>()?;
    m.add_class::<ast::FB>()?;
    m.add_class::<ast::FC>()?;
    m.add_class::<ast::AliasedField>()?;
    m.add_class::<ast::Binary>()?;
    m.add_class::<ast::Unary>()?;
    m.add_class::<ast::Call>()?;
    m.add_class::<ast::FieldIn>()?;
    m.add_class::<ast::Alias>()?;

    m.add_class::<ast::SearchCommand>()?;
    m.add_class::<ast::EvalCommand>()?;
    m.add_class::<ast::FieldConversion>()?;
    m.add_class::<ast::ConvertCommand>()?;
    m.add_class::<ast::LookupOutput>()?;
    m.add_class::<ast::LookupCommand>()?;
    m.add_class::<ast::CollectCommand>()?;
    m.add_class::<ast::WhereCommand>()?;
    m.add_class::<ast::TableCommand>()?;
    m.add_class::<ast::HeadCommand>()?;
    m.add_class::<ast::FieldsCommand>()?;
    m.add_class::<ast::SortCommand>()?;
    m.add_class::<ast::StatsCommand>()?;
    m.add_class::<ast::RexCommand>()?;
    m.add_class::<ast::RenameCommand>()?;
    m.add_class::<ast::RegexCommand>()?;
    m.add_class::<ast::JoinCommand>()?;
    m.add_class::<ast::ReturnCommand>()?;
    m.add_class::<ast::FillNullCommand>()?;
    m.add_class::<ast::EventStatsCommand>()?;
    m.add_class::<ast::StreamStatsCommand>()?;
    m.add_class::<ast::DedupCommand>()?;
    m.add_class::<ast::InputLookup>()?;
    m.add_class::<ast::FormatCommand>()?;
    m.add_class::<ast::MvCombineCommand>()?;
    m.add_class::<ast::MvExpandCommand>()?;
    m.add_class::<ast::MakeResults>()?;
    m.add_class::<ast::AddTotals>()?;
    m.add_class::<ast::BinCommand>()?;
    m.add_class::<ast::MultiSearch>()?;
    m.add_class::<ast::MapCommand>()?;
    m.add_class::<ast::Pipeline>()?;

    m.add_class::<ast::SplSpan>()?;
    m.add_class::<ast::Constant>()?;
    m.add_class::<ast::LeafExpr>()?;
    m.add_class::<ast::Expr>()?;
    m.add_class::<ast::FieldLike>()?;
    m.add_class::<ast::FieldOrAlias>()?;
    m.add_class::<ast::Command>()?;

    Ok(())
}
