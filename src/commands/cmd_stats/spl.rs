use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::commands::stats_utils;
use crate::commands::stats_utils::maybe_spanned_field_list1;
use crate::spl::ast::{Expr, ParsedCommandOptions};
use crate::spl::parser::{bool_, stats_call, ws};
use crate::spl::python::*;
use nom::bytes::complete::{tag, tag_no_case};
use nom::combinator::{map, opt};
use nom::sequence::{pair, preceded, tuple};
use nom::IResult;
use pyo3::prelude::*;
//
//   def stats[_: P]: P[StatsCommand] = ("stats" ~ commandOptions ~ statsCall ~
//     (W("by") ~ fieldList).?.map(fields => fields.getOrElse(Seq())) ~
//     ("dedup_splitvals" ~ "=" ~ bool).?.map(v => v.exists(_.value)))
//     .map {
//       case (options, exprs, fields, dedup) =>
//         StatsCommand(
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
pub struct StatsCommand {
    #[pyo3(get)]
    pub partitions: usize,
    #[pyo3(get)]
    pub all_num: bool,
    #[pyo3(get)]
    pub delim: String,
    #[pyo3(get)]
    pub funcs: Vec<Expr>,
    #[pyo3(get)]
    pub by: Option<Vec<stats_utils::MaybeSpannedField>>,
    #[pyo3(get)]
    pub dedup_split_vals: bool,
}
impl_pyclass!(StatsCommand { partitions: usize, all_num: bool, delim: String, funcs: Vec<Expr>, dedup_split_vals: bool, by: Option<Vec<stats_utils::MaybeSpannedField>> });

#[derive(Debug, Default)]
pub struct StatsParser {}
pub struct StatsCommandOptions {
    partitions: usize,
    all_num: bool,
    delim: String,
}

impl SplCommandOptions for StatsCommandOptions {}

impl TryFrom<ParsedCommandOptions> for StatsCommandOptions {
    type Error = anyhow::Error;

    fn try_from(value: ParsedCommandOptions) -> Result<Self, Self::Error> {
        Ok(Self {
            partitions: value.get_int("partitions", 1)? as usize,
            all_num: value.get_boolean("allnum", false)?,
            delim: value.get_string("delim", " ")?,
        })
    }
}

impl SplCommand<StatsCommand> for StatsParser {
    type RootCommand = crate::commands::StatsCommandRoot;
    type Options = StatsCommandOptions;

    fn parse_body(input: &str) -> IResult<&str, StatsCommand> {
        map(
            tuple((
                Self::Options::match_options,
                stats_call,
                opt(preceded(ws(tag_no_case("by")), maybe_spanned_field_list1)),
                opt(preceded(
                    pair(ws(tag_no_case("dedup_splitvals")), ws(tag("="))),
                    bool_,
                )),
            )),
            |(options, exprs, fields, dedup)| StatsCommand {
                partitions: options.partitions,
                all_num: options.all_num,
                delim: options.delim,
                funcs: exprs,
                by: fields,
                dedup_split_vals: dedup.map(|b| b.0).unwrap_or(false),
            },
        )(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spl::ast;
    use crate::spl::parser::pipeline;
    use crate::spl::utils::test::*;
    use rstest::rstest;

    //
    //   test("stats first(startTime) AS startTime, last(histID) AS lastPassHistId BY testCaseId") {
    //     p(pipeline(_), Pipeline(Seq(
    //       StatsCommand(
    //         partitions = 1,
    //         allNum = false,
    //         delim = " ",
    //         funcs = Seq(
    //           Alias(
    //             Call("first", Seq(
    //               Field("startTime")
    //             )),
    //             "startTime"),
    //           Alias(
    //             Call("last", Seq(
    //               Field("histID")
    //             )),
    //             "lastPassHistId")
    //         ),
    //         by = Seq(
    //           Field("testCaseId")
    //         ))
    //     )))
    //   }
    #[rstest]
    fn test_stats_1() {
        assert_eq!(
            pipeline(
                "stats first(startTime) AS startTime, last(histID) AS lastPassHistId BY testCaseId"
            ),
            Ok((
                "",
                ast::Pipeline {
                    commands: vec![StatsCommand {
                        partitions: 1,
                        all_num: false,
                        delim: " ".to_string(),
                        funcs: vec![
                            _alias("startTime", _call!(first(ast::Field::from("startTime"))))
                                .into(),
                            _alias("lastPassHistId", _call!(last(ast::Field::from("histID"))))
                                .into(),
                        ],
                        by: Some(vec![ast::Field::from("testCaseId").into()]),
                        dedup_split_vals: false
                    }
                    .into()],
                }
            ))
        )
    }

    //
    //   test("stats count(eval(status=404))") {
    //     p(pipeline(_), Pipeline(Seq(
    //       StatsCommand(
    //         partitions = 1,
    //         allNum = false,
    //         delim = " ",
    //         funcs = Seq(
    //           Call("count", Seq(
    //             Call("eval", Seq(
    //               Binary(
    //                 Field("status"),
    //                 Equals,
    //                 IntValue(404)
    //               )
    //             ))
    //           ))
    //         )
    //       ))
    //     ))
    //   }
    #[rstest]
    fn test_stats_2() {
        assert_eq!(
            pipeline("stats count(eval(status=404))"),
            Ok((
                "",
                ast::Pipeline {
                    commands: vec![StatsCommand {
                        partitions: 1,
                        all_num: false,
                        delim: " ".to_string(),
                        funcs: vec![_call!(count(_call!(eval(_eq(
                            ast::Field::from("status"),
                            ast::IntValue(404)
                        )))))
                        .into()],
                        by: None,
                        dedup_split_vals: false
                    }
                    .into()],
                }
            ))
        )
    }

    //
    //   test("no-comma stats") {
    //     val query =
    //       """stats allnum=f delim=":" partitions=10 count
    //         |earliest(_time) as earliest latest(_time) as latest
    //         |values(var_2) as var_2
    //         |by var_1
    //         |""".stripMargin
    //     parses(query, stats(_), StatsCommand(
    //       partitions = 10,
    //       allNum = false,
    //       delim = ":",
    //       Seq(
    //         Call("count"),
    //         Alias(Call("earliest", Seq(Field("_time"))), "earliest"),
    //         Alias(Call("latest", Seq(Field("_time"))), "latest"),
    //         Alias(Call("values", Seq(Field("var_2"))), "var_2")
    //       ),
    //       Seq(
    //         Field("var_1")
    //       )
    //     ))
    //   }
    #[rstest]
    fn test_no_comma_stats_1() {
        assert_eq!(
            pipeline(
                r#"stats allnum=f delim=":" partitions=10 count earliest(_time) as earliest latest(_time) as latest values(var_2) as var_2 by var_1"#
            ),
            Ok((
                "",
                ast::Pipeline {
                    commands: vec![StatsCommand {
                        partitions: 10,
                        all_num: false,
                        delim: ":".to_string(),
                        funcs: vec![
                            _call!(count()).into(),
                            _alias("earliest", _call!(earliest(ast::Field::from("_time")))).into(),
                            _alias("latest", _call!(latest(ast::Field::from("_time")))).into(),
                            _alias("var_2", _call!(values(ast::Field::from("var_2")))).into(),
                        ],
                        by: Some(vec![ast::Field::from("var_1").into()]),
                        dedup_split_vals: false
                    }
                    .into()],
                }
            ))
        )
    }

    #[rstest]
    fn test_stats_3() {
        let query = r#"stats
        count min(_time) as firstTime max(_time) as lastTime
        by action deviceowner user urlcategory url src dest"#;

        assert_eq!(
            StatsParser::parse(query),
            Ok((
                "",
                StatsCommand {
                    partitions: 1,
                    all_num: false,
                    delim: " ".to_string(),
                    funcs: vec![
                        _call!(count()).into(),
                        _alias("firstTime", _call!(min(ast::Field::from("_time")))).into(),
                        _alias("lastTime", _call!(max(ast::Field::from("_time")))).into(),
                    ],
                    by: Some(vec![
                        ast::Field::from("action").into(),
                        ast::Field::from("deviceowner").into(),
                        ast::Field::from("user").into(),
                        ast::Field::from("urlcategory").into(),
                        ast::Field::from("url").into(),
                        ast::Field::from("src").into(),
                        ast::Field::from("dest").into(),
                    ]),
                    dedup_split_vals: false,
                }
            ))
        );
    }

    #[rstest]
    fn test_stats_4() {
        let query = "stats count AS instances_launched by _time userName";
        assert_eq!(
            StatsParser::parse(query),
            Ok((
                "",
                StatsCommand {
                    partitions: 1,
                    all_num: false,
                    delim: " ".to_string(),
                    funcs: vec![_alias("instances_launched", _call!(count())).into(),],
                    by: Some(vec![
                        ast::Field::from("_time").into(),
                        ast::Field::from("userName").into(),
                    ]),
                    dedup_split_vals: false,
                }
            ))
        )
    }
}
