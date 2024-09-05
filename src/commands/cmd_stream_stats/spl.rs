use crate::ast::ast;
use crate::ast::ast::ParsedCommandOptions;
use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::{field_list, stats_call, ws};
use nom::bytes::complete::tag_no_case;
use nom::combinator::{map, opt};
use nom::sequence::{preceded, tuple};
use nom::{IResult, Parser};

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

impl SplCommand<ast::StreamStatsCommand> for StreamStatsParser {
    type RootCommand = crate::commands::StreamStatsCommand;
    type Options = StreamStatsCommandOptions;

    fn parse_body(input: &str) -> IResult<&str, ast::StreamStatsCommand> {
        map(
            tuple((
                Self::Options::match_options,
                ws(stats_call),
                opt(preceded(ws(tag_no_case("by")), field_list)),
            )),
            |(options, funcs, by)| ast::StreamStatsCommand {
                funcs,
                by: by.unwrap_or(vec![]),
                current: options.current,
                window: options.window,
            },
        )(input)
    }
}
