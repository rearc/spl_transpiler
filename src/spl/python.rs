use crate::spl::ast;
use pyo3::prelude::*;

macro_rules! impl_pyclass {
    ($name: ty { $arg_tp: ty }) => {
        #[pymethods]
        impl $name {
            #[allow(clippy::too_many_arguments)]
            #[new]
            fn new(value: $arg_tp) -> Self {
                Self ( value )
            }

            fn __repr__(&self) -> String {
                format!("{:?}", self)
            }
        }
    };
    ($name: path {$($arg: ident : $arg_tp: ty),*}) => {
        #[pymethods]
        impl $name {
            #[allow(clippy::too_many_arguments)]
            #[new]
            fn new($($arg: $arg_tp),*) -> Self {
                Self { $($arg),* }
            }

            fn __repr__(&self) -> String {
                format!("{:?}", self)
            }
        }
    };
    ($name: path {$(|$arg: ident : $arg_tp: ty| $conv: expr),*}) => {
        #[pymethods]
        impl $name {
            #[allow(clippy::too_many_arguments)]
            #[new]
            fn new($( $arg: $arg_tp ),*) -> Self {
                Self { $( $arg : (|$arg: $arg_tp| $conv)($arg) ),* }
            }

            fn __repr__(&self) -> String {
                format!("{:?}", self)
            }

            $other
        }
    };
}

pub(crate) use impl_pyclass;

// impl_pyclass!(ast::NullValue ());
impl_pyclass!(ast::BoolValue { bool });
impl_pyclass!(ast::IntValue { i64 });
impl_pyclass!(ast::StrValue { String });
impl_pyclass!(ast::DoubleValue { f64 });
impl_pyclass!(ast::SnapTime { span: Option<ast::TimeSpan>, snap: String, snap_offset: Option<ast::TimeSpan> });
impl_pyclass!(ast::Field { String });
impl_pyclass!(ast::Wildcard { String });
impl_pyclass!(ast::Variable { String });
impl_pyclass!(ast::IPv4CIDR { String });
impl_pyclass!(ast::FV {
    field: String,
    value: String
});
impl_pyclass!(ast::FB {
    field: String,
    value: bool
});
impl_pyclass!(ast::FC {
    field: String,
    value: ast::Constant
});
impl_pyclass!(ast::CommandOptions { options: Vec<ast::FC> });
impl_pyclass!(ast::AliasedField {
    field: ast::Field,
    alias: String
});
// impl_pyclass!(ast::Binary { |left: ast::Expr| Box::new(left), |symbol: String| symbol, |right: ast::Expr| Box::new(right) } {{}});
// impl_pyclass!(ast::Unary { |symbol: String| symbol, |right: ast::Expr| Box::new(right) });
impl_pyclass!(ast::Call { name: String, args: Vec<ast::Expr> });
impl_pyclass!(ast::FieldIn { field: String, exprs: Vec<ast::Expr> });
// impl_pyclass!(ast::Alias { |expr: ast::Expr| Box::new(expr), |name: String| name });

impl_pyclass!(ast::Pipeline { commands: Vec<ast::Command> });

#[pymethods]
impl ast::Binary {
    #[new]
    fn new(left: ast::Expr, symbol: String, right: ast::Expr) -> Self {
        Self {
            left: Box::new(left),
            symbol,
            right: Box::new(right),
        }
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }

    #[getter]
    fn left(&self) -> ast::Expr {
        (*self.left).clone()
    }

    #[getter]
    fn right(&self) -> ast::Expr {
        (*self.right).clone()
    }
}

#[pymethods]
impl ast::Unary {
    #[new]
    fn new(symbol: String, right: ast::Expr) -> Self {
        Self {
            symbol,
            right: Box::new(right),
        }
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }

    #[getter]
    fn right(&self) -> ast::Expr {
        (*self.right).clone()
    }
}

#[pymethods]
impl ast::Alias {
    #[new]
    fn new(expr: ast::Expr, name: String) -> Self {
        Self {
            expr: Box::new(expr),
            name,
        }
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }

    #[getter]
    fn expr(&self) -> ast::Expr {
        (*self.expr).clone()
    }
}

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

    m.add_class::<crate::commands::cmd_search::spl::SearchCommand>()?;
    m.add_class::<crate::commands::cmd_eval::spl::EvalCommand>()?;
    m.add_class::<crate::commands::cmd_convert::spl::FieldConversion>()?;
    m.add_class::<crate::commands::cmd_convert::spl::ConvertCommand>()?;
    m.add_class::<crate::commands::cmd_lookup::spl::LookupOutput>()?;
    m.add_class::<crate::commands::cmd_lookup::spl::LookupCommand>()?;
    m.add_class::<crate::commands::cmd_collect::spl::CollectCommand>()?;
    m.add_class::<crate::commands::cmd_where::spl::WhereCommand>()?;
    m.add_class::<crate::commands::cmd_table::spl::TableCommand>()?;
    m.add_class::<crate::commands::cmd_head::spl::HeadCommand>()?;
    m.add_class::<crate::commands::cmd_fields::spl::FieldsCommand>()?;
    m.add_class::<crate::commands::cmd_sort::spl::SortCommand>()?;
    m.add_class::<crate::commands::cmd_stats::spl::StatsCommand>()?;
    m.add_class::<crate::commands::cmd_rex::spl::RexCommand>()?;
    m.add_class::<crate::commands::cmd_rename::spl::RenameCommand>()?;
    m.add_class::<crate::commands::cmd_regex::spl::RegexCommand>()?;
    m.add_class::<crate::commands::cmd_join::spl::JoinCommand>()?;
    m.add_class::<crate::commands::cmd_return::spl::ReturnCommand>()?;
    m.add_class::<crate::commands::cmd_fill_null::spl::FillNullCommand>()?;
    m.add_class::<crate::commands::cmd_event_stats::spl::EventStatsCommand>()?;
    m.add_class::<crate::commands::cmd_stream_stats::spl::StreamStatsCommand>()?;
    m.add_class::<crate::commands::cmd_dedup::spl::DedupCommand>()?;
    m.add_class::<crate::commands::cmd_input_lookup::spl::InputLookup>()?;
    m.add_class::<crate::commands::cmd_format::spl::FormatCommand>()?;
    m.add_class::<crate::commands::cmd_mv_combine::spl::MvCombineCommand>()?;
    m.add_class::<crate::commands::cmd_mv_expand::spl::MvExpandCommand>()?;
    m.add_class::<crate::commands::cmd_make_results::spl::MakeResults>()?;
    m.add_class::<crate::commands::cmd_add_totals::spl::AddTotals>()?;
    m.add_class::<crate::commands::cmd_bin::spl::BinCommand>()?;
    m.add_class::<crate::commands::cmd_multi_search::spl::MultiSearch>()?;
    m.add_class::<crate::commands::cmd_map::spl::MapCommand>()?;
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
