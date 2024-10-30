use crate::functions::stat_fns::stats_fn;
use crate::pyspark::alias::Aliasable;
use crate::pyspark::ast::ColumnLike::FunctionCall;
use crate::pyspark::ast::*;
use crate::pyspark::base::{PysparkTranspileContext, ToSparkExpr};
use crate::spl::ast;
use crate::spl::ast::{Field, TimeSpan};
use crate::spl::parser::{comma_or_space_separated_list1, field, time_span, ws};
use anyhow::{bail, ensure};
use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::character::complete::multispace1;
use nom::combinator::map;
use nom::sequence::{preceded, separated_pair};
use nom::IResult;
use nom::Parser;
use pyo3::pyclass;

pub fn transform_by_field(f: MaybeSpannedField) -> ColumnOrName {
    match f.clone() {
        MaybeSpannedField {
            field: ast::Field(field),
            span: None,
        } => ColumnOrName::Name(field),
        MaybeSpannedField {
            field: ast::Field(field),
            span: Some(span),
        } => {
            let span_text = format!("{} {}", span.value, span.scale);
            column_like!(window([py_lit(field)], [py_lit(span_text)])).into()
        }
    }
}

pub fn transform_stats_expr(
    df: DataFrame,
    expr: &ast::Expr,
    ctx: &PysparkTranspileContext,
) -> anyhow::Result<(DataFrame, ColumnLike)> {
    let (e, maybe_name) = expr.clone().unaliased_with_name();
    ensure!(
        matches!(e, ast::Expr::Call(_)),
        "All `stats` aggregations must be function calls"
    );
    let (df_, e) = stats_fn(e, df, ctx)?;
    Ok((df_, e.maybe_with_alias(maybe_name)))
}

pub fn transform_stats_runtime_expr(
    expr: &ast::Expr,
    ctx: &PysparkTranspileContext,
) -> anyhow::Result<(String, RuntimeExpr)> {
    let (e, maybe_name) = expr.clone().unaliased_with_name();
    let (name, args) = match e {
        ast::Expr::Call(ast::Call { name, args }) => (name.to_string(), args),
        _ => bail!(
            "All `stats` aggregations must be function calls, got {:?}",
            expr
        ),
    };
    let alias = maybe_name.unwrap_or(name.clone());
    let args: anyhow::Result<Vec<Expr>> = args
        .into_iter()
        .map(|e| e.with_context(ctx).try_into())
        .collect();
    let expr: Expr = FunctionCall {
        func: format!("functions.stats.{}", name).to_string(),
        args: args?,
    }
    .into();
    Ok((alias, RuntimeExpr::from(expr)))
}

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct MaybeSpannedField {
    pub field: Field,
    pub span: Option<TimeSpan>,
}

impl From<Field> for MaybeSpannedField {
    fn from(field: Field) -> Self {
        MaybeSpannedField { field, span: None }
    }
}

pub fn maybe_spanned_field(input: &str) -> IResult<&str, MaybeSpannedField> {
    alt((
        map(
            separated_pair(
                field,
                multispace1,
                preceded(ws(tag_no_case("span=")), time_span),
            ),
            |(field, span)| MaybeSpannedField {
                field,
                span: Some(span),
            },
        ),
        map(field, |field| MaybeSpannedField { field, span: None }),
    ))(input)
}

pub fn maybe_spanned_field_list1(input: &str) -> IResult<&str, Vec<MaybeSpannedField>> {
    comma_or_space_separated_list1(maybe_spanned_field).parse(input)
}
