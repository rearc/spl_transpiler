use crate::ast::ast;
use crate::ast::ast::ParsedCommandOptions;
use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::{double_quoted, field, token, ws};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::{map, opt};
use nom::multi::many1;
use nom::sequence::{preceded, tuple};
use nom::{IResult, Parser};
//
//   def fillNull[_: P]: P[FillNullCommand] = ("fillnull" ~ ("value=" ~~ (doubleQuoted|token)).?
//     ~ field.rep(1).?) map FillNullCommand.tupled

#[derive(Debug, Default)]
pub struct FillNullParser {}
pub struct FillNullCommandOptions {}

impl SplCommandOptions for FillNullCommandOptions {}

impl TryFrom<ParsedCommandOptions> for FillNullCommandOptions {
    type Error = anyhow::Error;

    fn try_from(value: ParsedCommandOptions) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl SplCommand<ast::FillNullCommand> for FillNullParser {
    type RootCommand = crate::commands::FillNullCommand;
    type Options = FillNullCommandOptions;

    fn parse_body(input: &str) -> IResult<&str, ast::FillNullCommand> {
        map(
            tuple((
                opt(preceded(tag("value="), alt((double_quoted, token)))),
                opt(many1(map(ws(field), |v| v.into()))),
            )),
            |(maybe_value, fields)| ast::FillNullCommand {
                value: maybe_value.map(|v| v.to_string()),
                fields,
            },
        )(input)
    }
}
