use crate::ast::ast;
use crate::ast::ast::ParsedCommandOptions;
use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::{field, int, ws};
use nom::bytes::complete::{tag, tag_no_case};
use nom::combinator::{map, opt};
use nom::sequence::{pair, preceded};
use nom::{IResult, Parser};

//
//   def mvexpand[_: P]: P[MvExpandCommand] = ("mvexpand" ~ field ~ ("limit" ~ "=" ~ int).?) map {
//     case (field, None) => MvExpandCommand(field, None)
//     case (field, Some(limit)) => MvExpandCommand(field, Some(limit.value))
//   }

#[derive(Debug, Default)]
pub struct MvExpandParser {}
pub struct MvExpandCommandOptions {}

impl SplCommandOptions for MvExpandCommandOptions {}

impl TryFrom<ParsedCommandOptions> for MvExpandCommandOptions {
    type Error = anyhow::Error;

    fn try_from(value: ParsedCommandOptions) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl SplCommand<ast::MvExpandCommand> for MvExpandParser {
    type RootCommand = crate::commands::MvExpandCommand;
    type Options = MvExpandCommandOptions;

    fn parse_body(input: &str) -> IResult<&str, ast::MvExpandCommand> {
        map(
            pair(
                ws(field),
                opt(preceded(
                    pair(ws(tag_no_case("limit")), ws(tag("="))),
                    ws(int),
                )),
            ),
            |(field, limit_opt)| ast::MvExpandCommand {
                field,
                limit: limit_opt.map(|v| v.0),
            },
        )(input)
    }
}
