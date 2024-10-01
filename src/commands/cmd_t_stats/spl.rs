use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::ast::{
    Binary, Call, Constant, Expr, Field, LeafExpr, ParsedCommandOptions, TimeSpan,
};
use crate::spl::operators;
use crate::spl::operators::OperatorSymbolTrait;
use crate::spl::parser::{
    comma_or_space_separated_list1, field, field_in, logical_expression, space_separated_list1,
    time_span, token, unwrapped_option, ws,
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
    pub by_fields: Option<Vec<MaybeSpannedField>>,
    #[pyo3(get)]
    pub by_prefix: Option<String>,
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
    by_fields: Option<Vec<MaybeSpannedField>>,
    by_prefix: Option<String>
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
                into(recognize(separated_list1(tag("."), token))),
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
                    preceded(opt(ws(tag_no_case("AND"))), logical_expression),
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
    fields: Option<Vec<MaybeSpannedField>>,
    prefix: Option<String>,
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

fn _parse_by(input: &str) -> IResult<&str, ByClause> {
    map(
        preceded(
            ws(tag_no_case("by")),
            alt((
                // Freaking seriously? This can be either a space-delimited or comma-delimited list
                // WHY?!?
                map(
                    comma_or_space_separated_list1(alt((
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
                    ))),
                    |fields| (Some(fields), None),
                ),
                map(
                    ws(preceded(
                        tag_no_case("PREFIX"),
                        delimited(tag("("), field, tag(")")),
                    )),
                    |Field(prefix_field)| (None, Some(prefix_field)),
                ),
            )),
        ),
        |(fields, prefix)| ByClause { fields, prefix },
    )(input)
}

fn _stats_function_count(input: &str) -> IResult<&str, Call> {
    map(
        preceded(
            tag_no_case("count"),
            opt(delimited(
                multispace1,
                verify(field, |Field(name)| {
                    !matches!(name.to_ascii_lowercase().as_str(), "from" | "where" | "by")
                }),
                alt((multispace1, eof)),
            )),
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
                space_separated_list1(_stats_function),
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
            },
        )(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spl::ast;
    use crate::spl::utils::test::*;
    use nom::combinator::all_consuming;

    #[test]
    fn test_xsl_script_execution_with_wmic_1() {
        let query = r#"tstats summariesonly=false allow_old_summaries=true fillnull_value=null
                count min(_time) as firstTime max(_time) as lastTime
                from datamodel=Endpoint.Processes
                where (Processes.process_name=wmic.exe OR Processes.original_file_name=wmic.exe) Processes.process = "*os get*" Processes.process="*/format:*" Processes.process = "*.xsl*"
                by Processes.parent_process_name Processes.parent_process Processes.process_name Processes.process_id Processes.process Processes.dest Processes.user"#;
        assert_eq!(
            all_consuming(TStatsParser::parse)(query),
            Ok((
                "",
                TStatsCommand {
                    prestats: false,
                    local: false,
                    append: false,
                    summaries_only: false,
                    include_reduced_buckets: false,
                    allow_old_summaries: true,
                    chunk_size: 10000000,
                    fillnull_value: Some("null".into()),
                    exprs: vec![
                        _call!(count()).into(),
                        _alias("firstTime", _call!(min(ast::Field::from("_time")))).into(),
                        _alias("lastTime", _call!(max(ast::Field::from("_time")))).into(),
                    ],
                    datamodel: Some("Endpoint.Processes".into()),
                    nodename: None,
                    where_condition: Some(_and(
                        _and(
                            _and(
                                _or(
                                    _eq(
                                        ast::Field::from("Processes.process_name"),
                                        ast::StrValue::from("wmic.exe")
                                    ),
                                    _eq(
                                        ast::Field::from("Processes.original_file_name"),
                                        ast::StrValue::from("wmic.exe")
                                    )
                                ),
                                _eq(
                                    ast::Field::from("Processes.process"),
                                    ast::Wildcard::from("*os get*")
                                ),
                            ),
                            _eq(
                                ast::Field::from("Processes.process"),
                                ast::Wildcard::from("*/format:*")
                            ),
                        ),
                        _eq(
                            ast::Field::from("Processes.process"),
                            ast::Wildcard::from("*.xsl*")
                        )
                    )),
                    by_fields: Some(vec![
                        ast::Field::from("Processes.parent_process_name").into(),
                        ast::Field::from("Processes.parent_process").into(),
                        ast::Field::from("Processes.process_name").into(),
                        ast::Field::from("Processes.process_id").into(),
                        ast::Field::from("Processes.process").into(),
                        ast::Field::from("Processes.dest").into(),
                        ast::Field::from("Processes.user").into(),
                    ]),
                    by_prefix: None,
                }
            ))
        );
    }

    #[test]
    fn test_tstats_2() {
        let query = r#"tstats
            count min(_time) as firstTime max(_time) as lastTime
            from datamodel=Web
            where Web.url IN ("/AHT/AhtApiService.asmx/AuthUser") Web.status=200 Web.http_method=POST
            by Web.http_user_agent, Web.status Web.http_method, Web.url, Web.url_length, Web.src, Web.dest, sourcetype"#;
        assert_eq!(
            all_consuming(TStatsParser::parse)(query),
            Ok((
                "",
                TStatsCommand {
                    prestats: false,
                    local: false,
                    append: false,
                    summaries_only: false,
                    include_reduced_buckets: false,
                    allow_old_summaries: false,
                    chunk_size: 10000000,
                    fillnull_value: None,
                    exprs: vec![
                        _call!(count()).into(),
                        _alias("firstTime", _call!(min(ast::Field::from("_time")))).into(),
                        _alias("lastTime", _call!(max(ast::Field::from("_time")))).into(),
                    ],
                    datamodel: Some("Web".into()),
                    nodename: None,
                    where_condition: Some(_and(
                        _and(
                            ast::FieldIn {
                                field: "Web.url".into(),
                                exprs: vec![ast::StrValue::from(
                                    "/AHT/AhtApiService.asmx/AuthUser"
                                )
                                .into()],
                            },
                            _eq(ast::Field::from("Web.status"), ast::IntValue::from(200)),
                        ),
                        _eq(
                            ast::Field::from("Web.http_method"),
                            ast::StrValue::from("POST")
                        )
                    )),
                    by_fields: Some(vec![
                        ast::Field::from("Web.http_user_agent").into(),
                        ast::Field::from("Web.status").into(),
                        ast::Field::from("Web.http_method").into(),
                        ast::Field::from("Web.url").into(),
                        ast::Field::from("Web.url_length").into(),
                        ast::Field::from("Web.src").into(),
                        ast::Field::from("Web.dest").into(),
                        ast::Field::from("sourcetype").into(),
                    ]),
                    by_prefix: None,
                }
            ))
        );
    }

    #[test]
    fn test_tstats_3() {
        let query = r#"tstats
        summariesonly=false allow_old_summaries=true fillnull_value=null
        count min(_time) AS firstTime max(_time) AS lastTime
        FROM datamodel=Endpoint.Processes
        BY _time span=1h Processes.user Processes.process_id Processes.process_name Processes.process Processes.process_path Processes.dest Processes.parent_process_name Processes.parent_process Processes.process_guid"#;

        assert_eq!(
            all_consuming(TStatsParser::parse)(query),
            Ok((
                "",
                TStatsCommand {
                    prestats: false,
                    local: false,
                    append: false,
                    summaries_only: false,
                    include_reduced_buckets: false,
                    allow_old_summaries: true,
                    chunk_size: 10000000,
                    fillnull_value: Some("null".into()),
                    exprs: vec![
                        _call!(count()).into(),
                        _alias("firstTime", _call!(min(ast::Field::from("_time")))).into(),
                        _alias("lastTime", _call!(max(ast::Field::from("_time")))).into(),
                    ],
                    datamodel: Some("Endpoint.Processes".into()),
                    nodename: None,
                    where_condition: None,
                    by_fields: Some(vec![
                        MaybeSpannedField {
                            field: ast::Field::from("_time"),
                            span: Some(TimeSpan {
                                value: 1,
                                scale: "hours".to_string(),
                            }),
                        },
                        ast::Field::from("Processes.user").into(),
                        ast::Field::from("Processes.process_id").into(),
                        ast::Field::from("Processes.process_name").into(),
                        ast::Field::from("Processes.process").into(),
                        ast::Field::from("Processes.process_path").into(),
                        ast::Field::from("Processes.dest").into(),
                        ast::Field::from("Processes.parent_process_name").into(),
                        ast::Field::from("Processes.parent_process").into(),
                        ast::Field::from("Processes.process_guid").into(),
                    ]),
                    by_prefix: None,
                }
            ))
        );
    }

    #[test]
    fn test_tstats_4() {
        let query = r#"tstats
        summariesonly=false allow_old_summaries=true fillnull_value=null
        count min(_time) as firstTime max(_time) as lastTime
        from datamodel=Endpoint.Processes
        where (Processes.process_name ="7z.exe" OR Processes.process_name = "7za.exe" OR Processes.original_file_name = "7z.exe" OR Processes.original_file_name =  "7za.exe") AND (Processes.process="*\\C$\\*" OR Processes.process="*\\Admin$\\*" OR Processes.process="*\\IPC$\\*")
        by Processes.original_file_name Processes.parent_process_name Processes.parent_process Processes.process_name Processes.process Processes.parent_process_id Processes.process_id  Processes.dest Processes.user"#;

        assert_eq!(
            all_consuming(TStatsParser::parse)(query),
            Ok((
                "",
                TStatsCommand {
                    prestats: false,
                    local: false,
                    append: false,
                    summaries_only: false,
                    include_reduced_buckets: false,
                    allow_old_summaries: true,
                    chunk_size: 10000000,
                    fillnull_value: Some("null".into()),
                    exprs: vec![
                        _call!(count()).into(),
                        _alias("firstTime", _call!(min(ast::Field::from("_time")))).into(),
                        _alias("lastTime", _call!(max(ast::Field::from("_time")))).into(),
                    ],
                    datamodel: Some("Endpoint.Processes".into()),
                    nodename: None,
                    where_condition: Some(_and(
                        _or(
                            _or(
                                _or(
                                    _eq(
                                        ast::Field::from("Processes.process_name"),
                                        ast::StrValue::from("7z.exe")
                                    ),
                                    _eq(
                                        ast::Field::from("Processes.process_name"),
                                        ast::StrValue::from("7za.exe")
                                    ),
                                ),
                                _eq(
                                    ast::Field::from("Processes.original_file_name"),
                                    ast::StrValue::from("7z.exe")
                                ),
                            ),
                            _eq(
                                ast::Field::from("Processes.original_file_name"),
                                ast::StrValue::from("7za.exe")
                            )
                        ),
                        _or(
                            _or(
                                _eq(
                                    ast::Field::from("Processes.process"),
                                    ast::Wildcard::from(r#"*\\C$\\*"#)
                                ),
                                _eq(
                                    ast::Field::from("Processes.process"),
                                    ast::Wildcard::from(r#"*\\Admin$\\*"#)
                                ),
                            ),
                            _eq(
                                ast::Field::from("Processes.process"),
                                ast::Wildcard::from(r#"*\\IPC$\\*"#)
                            )
                        )
                    )),
                    by_fields: Some(vec![
                        ast::Field::from("Processes.original_file_name").into(),
                        ast::Field::from("Processes.parent_process_name").into(),
                        ast::Field::from("Processes.parent_process").into(),
                        ast::Field::from("Processes.process_name").into(),
                        ast::Field::from("Processes.process").into(),
                        ast::Field::from("Processes.parent_process_id").into(),
                        ast::Field::from("Processes.process_id").into(),
                        ast::Field::from("Processes.dest").into(),
                        ast::Field::from("Processes.user").into(),
                    ]),
                    by_prefix: None,
                }
            ))
        );
    }

    #[test]
    fn test_tstats_5() {
        let query = r#"tstats
        summariesonly=false allow_old_summaries=true fillnull_value=null
        count
        FROM datamodel=Endpoint.Registry
        WHERE Registry.registry_path= "*\\AppX82a6gwre4fdg3bt635tn5ctqjf8msdd2\\Shell\\open\\command*" AND (Registry.registry_value_name = "(Default)" OR Registry.registry_value_name = "DelegateExecute")
        by _time span=1h Registry.dest Registry.user Registry.registry_path Registry.registry_key_name Registry.registry_value_name Registry.registry_value_data Registry.process_guid"#;

        assert_eq!(
            all_consuming(TStatsParser::parse)(query),
            Ok((
                "",
                TStatsCommand {
                    prestats: false,
                    local: false,
                    append: false,
                    summaries_only: false,
                    include_reduced_buckets: false,
                    allow_old_summaries: true,
                    chunk_size: 10000000,
                    fillnull_value: Some("null".into()),
                    exprs: vec![_call!(count()).into(),],
                    datamodel: Some("Endpoint.Registry".into()),
                    nodename: None,
                    where_condition: Some(_and(
                        _eq(
                            ast::Field::from("Registry.registry_path"),
                            ast::Wildcard::from(
                                r#"*\\AppX82a6gwre4fdg3bt635tn5ctqjf8msdd2\\Shell\\open\\command*"#
                            )
                        ),
                        _or(
                            _eq(
                                ast::Field::from("Registry.registry_value_name"),
                                ast::StrValue::from("(Default)")
                            ),
                            _eq(
                                ast::Field::from("Registry.registry_value_name"),
                                ast::StrValue::from("DelegateExecute")
                            )
                        )
                    )),
                    by_fields: Some(vec![
                        MaybeSpannedField {
                            field: ast::Field::from("_time"),
                            span: Some(TimeSpan {
                                value: 1,
                                scale: "hours".to_string(),
                            }),
                        },
                        ast::Field::from("Registry.dest").into(),
                        ast::Field::from("Registry.user").into(),
                        ast::Field::from("Registry.registry_path").into(),
                        ast::Field::from("Registry.registry_key_name").into(),
                        ast::Field::from("Registry.registry_value_name").into(),
                        ast::Field::from("Registry.registry_value_data").into(),
                        ast::Field::from("Registry.process_guid").into(),
                    ]),
                    by_prefix: None,
                }
            ))
        );
    }

    #[test]
    fn test_tstats_6() {
        let query = r#"tstats
        summariesonly=false allow_old_summaries=true fillnull_value=null
        count
        FROM datamodel=Network_Traffic.All_Traffic
        where All_Traffic.dest_port != 0 NOT (All_Traffic.dest IN (127.0.0.1,10.0.0.0/8,172.16.0.0/12, 192.168.0.0/16, 0:0:0:0:0:0:0:1))
        by All_Traffic.process_id All_Traffic.dest All_Traffic.dest_port"#;

        assert_eq!(
            all_consuming(TStatsParser::parse)(query),
            Ok((
                "",
                TStatsCommand {
                    prestats: false,
                    local: false,
                    append: false,
                    summaries_only: false,
                    include_reduced_buckets: false,
                    allow_old_summaries: true,
                    chunk_size: 10000000,
                    fillnull_value: Some("null".into()),
                    exprs: vec![_call!(count()).into(),],
                    datamodel: Some("Network_Traffic.All_Traffic".into()),
                    nodename: None,
                    where_condition: Some(_and(
                        _neq(
                            ast::Field::from("All_Traffic.dest_port"),
                            ast::IntValue::from(0)
                        ),
                        _not(_isin(
                            "All_Traffic.dest",
                            vec![
                                ast::StrValue::from("127.0.0.1").into(),
                                ast::IPv4CIDR::from("10.0.0.0/8").into(),
                                ast::IPv4CIDR::from("172.16.0.0/12").into(),
                                ast::IPv4CIDR::from("192.168.0.0/16").into(),
                                ast::StrValue::from("0:0:0:0:0:0:0:1").into(),
                            ]
                        ))
                    )),
                    by_fields: Some(vec![
                        ast::Field::from("All_Traffic.process_id").into(),
                        ast::Field::from("All_Traffic.dest").into(),
                        ast::Field::from("All_Traffic.dest_port").into(),
                    ]),
                    by_prefix: None,
                }
            ))
        );
    }

    #[test]
    fn test_tstats_7() {
        let query = r#"tstats
        summariesonly=false allow_old_summaries=true fillnull_value=null
        dc(Updates.dest) as count
        FROM datamodel=Updates
        where Updates.vendor_product="Microsoft Windows" AND Updates.status=failure
        by _time span=1d"#;

        assert_eq!(
            all_consuming(TStatsParser::parse)(query),
            Ok((
                "",
                TStatsCommand {
                    prestats: false,
                    local: false,
                    append: false,
                    summaries_only: false,
                    include_reduced_buckets: false,
                    allow_old_summaries: true,
                    chunk_size: 10000000,
                    fillnull_value: Some("null".into()),
                    exprs: vec![_alias("count", _call!(dc(Field::from("Updates.dest")))).into(),],
                    datamodel: Some("Updates".into()),
                    nodename: None,
                    where_condition: Some(_and(
                        _eq(
                            ast::Field::from("Updates.vendor_product"),
                            ast::StrValue::from("Microsoft Windows")
                        ),
                        _eq(
                            ast::Field::from("Updates.status"),
                            ast::StrValue::from("failure")
                        )
                    )),
                    by_fields: Some(vec![MaybeSpannedField {
                        field: Field::from("_time"),
                        span: Some(TimeSpan {
                            value: 1,
                            scale: "days".to_string(),
                        }),
                    }]),
                    by_prefix: None,
                }
            ))
        );
    }
}
