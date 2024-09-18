use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::ast::{
    Binary, Call, Constant, Expr, Field, LeafExpr, ParsedCommandOptions, TimeSpan,
};
use crate::spl::operators;
use crate::spl::operators::OperatorSymbolTrait;
use crate::spl::parser::{
    field, field_in, logical_expression, time_span, token, unwrapped_option, ws,
};
use crate::spl::python::impl_pyclass;
use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case};
use nom::character::complete::multispace1;
use nom::combinator::{eof, into, map, opt, recognize, verify};
use nom::multi::{fold_many1, separated_list1};
use nom::sequence::{delimited, pair, preceded, separated_pair, tuple};
use nom::IResult;
use pyo3::prelude::*;
/*
tstats
[prestats=<bool>]
[local=<bool>]
[append=<bool>]
[summariesonly=<bool>]
[include_reduced_buckets=<bool>]
[allow_old_summaries=<bool>]
[chunk_size=<unsigned int>]
[fillnull_value=<string>]
<stats-func>...
[ FROM datamodel=<data_model_name>.<root_dataset_name> [where nodename = <root_dataset_name>.<...>.<target_dataset_name>]]
[ WHERE <search-query> | <field> IN (<value-list>)]
[ BY (<field-list> | (PREFIX(<field>))) [span=<timespan>]]


 */

//
//   def stats[_: P]: P[TStatsCommand] = ("stats" ~ commandOptions ~ statsCall ~
//     (W("by") ~ fieldList).?.map(fields => fields.getOrElse(Seq())) ~
//     ("dedup_splitvals" ~ "=" ~ bool).?.map(v => v.exists(_.value)))
//     .map {
//       case (options, exprs, fields, dedup) =>
//         TStatsCommand(
//           partitions = options.getInt("partitions", 1),
//           allNum = options.getBoolean("allnum"),
//           delim = options.getString("delim", default = " "),
//           funcs = exprs,
//           by = fields,
//           dedupSplitVals = dedup
//         )
//     }

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct TStatsCommand {
    #[pyo3(get)]
    pub prestats: bool,
    #[pyo3(get)]
    pub local: bool,
    #[pyo3(get)]
    pub append: bool,
    #[pyo3(get)]
    pub summaries_only: bool,
    #[pyo3(get)]
    pub include_reduced_buckets: bool,
    #[pyo3(get)]
    pub allow_old_summaries: bool,
    #[pyo3(get)]
    pub chunk_size: i64,
    #[pyo3(get)]
    pub fillnull_value: Option<String>,
    #[pyo3(get)]
    pub exprs: Vec<Expr>,
    #[pyo3(get)]
    pub datamodel: Option<String>,
    #[pyo3(get)]
    pub nodename: Option<String>,
    #[pyo3(get)]
    pub where_condition: Option<Expr>,
    #[pyo3(get)]
    pub by_fields: Option<Vec<Field>>,
    #[pyo3(get)]
    pub by_prefix: Option<String>,
    #[pyo3(get)]
    pub span: Option<TimeSpan>,
}
impl_pyclass!(TStatsCommand {
    prestats: bool,
    local: bool,
    append: bool,
    summaries_only: bool,
    include_reduced_buckets: bool,
    allow_old_summaries: bool,
    chunk_size: i64,
    exprs: Vec<Expr>,
    where_condition: Option<Expr>,
    fillnull_value: Option<String>,
    datamodel: Option<String>,
    nodename: Option<String>,
    by_fields: Option<Vec<Field>>,
    by_prefix: Option<String>,
    span: Option<TimeSpan>
});

#[derive(Debug, Default)]
pub struct TStatsParser {}
pub struct TStatsCommandOptions {
    prestats: bool,
    local: bool,
    append: bool,
    summaries_only: bool,
    include_reduced_buckets: bool,
    allow_old_summaries: bool,
    chunk_size: i64,
    fillnull_value: Option<String>,
}

impl SplCommandOptions for TStatsCommandOptions {}

impl TryFrom<ParsedCommandOptions> for TStatsCommandOptions {
    type Error = anyhow::Error;

    fn try_from(value: ParsedCommandOptions) -> Result<Self, Self::Error> {
        Ok(Self {
            prestats: value.get_boolean("prestats", false)?,
            local: value.get_boolean("local", false)?,
            append: value.get_boolean("append", false)?,
            summaries_only: value.get_boolean("summariesonly", false)?,
            include_reduced_buckets: value.get_boolean("include_reduced_buckets", false)?,
            allow_old_summaries: value.get_boolean("allow_old_summaries", false)?,
            chunk_size: value.get_int("chunk_size", 10000000)?,
            fillnull_value: value.get_string_option("fillnull_value")?,
        })
    }
}

fn _parse_from_datamodel(input: &str) -> IResult<&str, (String, Option<String>)> {
    preceded(
        ws(tag_no_case("from")),
        tuple((
            ws(preceded(
                ws(tag_no_case("datamodel=")),
                into(recognize(separated_pair(token, tag("."), token))),
            )),
            opt(ws(preceded(
                ws(tag_no_case("where")),
                ws(preceded(
                    tag_no_case("nodename="),
                    into(recognize(separated_list1(tag("."), token))),
                )),
            ))),
        )),
    )(input)
}

fn _parse_where(input: &str) -> IResult<&str, Expr> {
    preceded(
        ws(tag_no_case("where")),
        ws(unwrapped_option(alt((
            fold_many1(
                ws(verify(
                    logical_expression,
                    |e| !matches!(e, Expr::Leaf(LeafExpr::Constant(Constant::Field(Field(name)))) if name.to_ascii_lowercase() == "by"),
                )),
                || None,
                |a, b| match a {
                    None => Some(b),
                    Some(a) => Some(Expr::Binary(Binary {
                        left: Box::new(a),
                        symbol: operators::And::SYMBOL.into(),
                        right: Box::new(b),
                    })),
                },
            ),
            map(field_in, |expr| Some(expr.into())),
        )))),
    )(input)
}

#[derive(Debug, Default)]
struct ByClause {
    fields: Option<Vec<Field>>,
    prefix: Option<String>,
    span: Option<TimeSpan>,
}

fn _parse_by(input: &str) -> IResult<&str, ByClause> {
    map(
        preceded(
            ws(tag_no_case("by")),
            tuple((
                alt((
                    map(separated_list1(multispace1, field), |fields| {
                        (Some(fields), None)
                    }),
                    map(
                        ws(preceded(
                            tag_no_case("PREFIX"),
                            delimited(tag("("), field, tag(")")),
                        )),
                        |Field(prefix_field)| (None, Some(prefix_field)),
                    ),
                )),
                opt(ws(preceded(ws(tag_no_case("span=")), time_span))),
            )),
        ),
        |((fields, prefix), span)| ByClause {
            fields,
            prefix,
            span,
        },
    )(input)
}

fn _stats_function_count(input: &str) -> IResult<&str, Call> {
    map(
        preceded(
            tag_no_case("count"),
            opt(delimited(multispace1, field, alt((multispace1, eof)))),
        ),
        |name| Call {
            name: "count".into(),
            args: name.map_or_else(std::vec::Vec::new, |f| vec![f.into()]),
        },
    )(input)
}

fn _stats_function_call(input: &str) -> IResult<&str, Call> {
    map(
        pair(
            token,
            // TODO: Support PREFIX(field)
            delimited(tag("("), field, tag(")")),
        ),
        |(func, field)| Call {
            name: func.into(),
            args: vec![field.into()],
        },
    )(input)
}

/// (count [<field>] | <function>(PREFIX(<string>) | <field>))... [AS<string>]
fn _stats_function(input: &str) -> IResult<&str, Expr> {
    map(
        pair(
            alt((_stats_function_call, _stats_function_count)),
            opt(preceded(
                delimited(multispace1, tag_no_case("AS"), multispace1),
                into(token),
            )),
        ),
        |(call, alias)| {
            let expr: Expr = call.into();
            expr.maybe_with_alias(alias)
        },
    )(input)
}

impl SplCommand<TStatsCommand> for TStatsParser {
    type RootCommand = crate::commands::TStatsCommandRoot;
    type Options = TStatsCommandOptions;

    fn parse_body(input: &str) -> IResult<&str, TStatsCommand> {
        map(
            tuple((
                Self::Options::match_options,
                separated_list1(multispace1, _stats_function),
                map(ws(opt(_parse_from_datamodel)), |res| match res {
                    None => (None, None),
                    Some((datamodel, nodename)) => (Some(datamodel), nodename),
                }),
                ws(opt(_parse_where)),
                map(ws(opt(_parse_by)), |res| res.unwrap_or_default()),
            )),
            |(options, exprs, (datamodel, nodename), where_condition, by_clause)| TStatsCommand {
                prestats: options.prestats,
                local: options.local,
                append: options.append,
                summaries_only: options.summaries_only,
                include_reduced_buckets: options.include_reduced_buckets,
                allow_old_summaries: options.allow_old_summaries,
                chunk_size: options.chunk_size,
                fillnull_value: options.fillnull_value,
                exprs,
                datamodel,
                nodename,
                where_condition,
                by_fields: by_clause.fields,
                by_prefix: by_clause.prefix,
                span: by_clause.span,
            },
        )(input)
    }
}
