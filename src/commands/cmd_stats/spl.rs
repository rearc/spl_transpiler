use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::ast::{Expr, Field, ParsedCommandOptions};
use crate::spl::parser::{bool_, field_list, stats_call, ws};
use crate::spl::python::impl_pyclass;
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
    pub partitions: i64,
    #[pyo3(get)]
    pub all_num: bool,
    #[pyo3(get)]
    pub delim: String,
    #[pyo3(get)]
    pub funcs: Vec<Expr>,
    #[pyo3(get)]
    pub by: Vec<Field>,
    #[pyo3(get)]
    pub dedup_split_vals: bool,
}
impl_pyclass!(StatsCommand { partitions: i64, all_num: bool, delim: String, funcs: Vec<Expr>, by: Vec<Field>, dedup_split_vals: bool });

#[derive(Debug, Default)]
pub struct StatsParser {}
pub struct StatsCommandOptions {
    partitions: i64,
    all_num: bool,
    delim: String,
}

impl SplCommandOptions for StatsCommandOptions {}

impl TryFrom<ParsedCommandOptions> for StatsCommandOptions {
    type Error = anyhow::Error;

    fn try_from(value: ParsedCommandOptions) -> Result<Self, Self::Error> {
        Ok(Self {
            partitions: value.get_int("partitions", 1)?,
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
                opt(preceded(ws(tag_no_case("by")), field_list)),
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
                by: fields.unwrap_or(vec![]),
                dedup_split_vals: dedup.map(|b| b.0).unwrap_or(false),
            },
        )(input)
    }
}
