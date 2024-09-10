use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::ast::{Expr, Field, ParsedCommandOptions};
use crate::spl::parser::{field_list, stats_call, ws};
use crate::spl::python::impl_pyclass;
use nom::bytes::complete::tag_no_case;
use nom::combinator::{map, opt};
use nom::sequence::{preceded, tuple};
use nom::IResult;
use pyo3::prelude::*;
//
//   def streamStats[_: P]: P[StreamStatsCommand] = ("streamstats" ~ commandOptions ~ statsCall
//     ~ (W("by") ~ fieldList).?.map(fields => fields.getOrElse(Seq()))).map {
//     case (options, funcs, by) =>
//       StreamStatsCommand(
//         funcs,
//         by,
//         options.getBoolean("current", default = true),
//         options.getInt("window")
//       )
//   }

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct StreamStatsCommand {
    #[pyo3(get)]
    pub funcs: Vec<Expr>,
    #[pyo3(get)]
    pub by: Vec<Field>,
    #[pyo3(get)]
    pub current: bool,
    #[pyo3(get)]
    pub window: i64,
}
impl_pyclass!(StreamStatsCommand { funcs: Vec<Expr>, by: Vec<Field>, current: bool, window: i64 });

#[derive(Debug, Default)]
pub struct StreamStatsParser {}
pub struct StreamStatsCommandOptions {
    current: bool,
    window: i64,
}

impl SplCommandOptions for StreamStatsCommandOptions {}

impl TryFrom<ParsedCommandOptions> for StreamStatsCommandOptions {
    type Error = anyhow::Error;

    fn try_from(value: ParsedCommandOptions) -> Result<Self, Self::Error> {
        Ok(Self {
            current: value.get_boolean("current", true)?,
            window: value.get_int("window", 0)?,
        })
    }
}

impl SplCommand<StreamStatsCommand> for StreamStatsParser {
    type RootCommand = crate::commands::StreamStatsCommandRoot;
    type Options = StreamStatsCommandOptions;

    fn parse_body(input: &str) -> IResult<&str, StreamStatsCommand> {
        map(
            tuple((
                Self::Options::match_options,
                ws(stats_call),
                opt(preceded(ws(tag_no_case("by")), field_list)),
            )),
            |(options, funcs, by)| StreamStatsCommand {
                funcs,
                by: by.unwrap_or(vec![]),
                current: options.current,
                window: options.window,
            },
        )(input)
    }
}
