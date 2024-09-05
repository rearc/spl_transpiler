use crate::ast::ast;
use crate::ast::ast::ParsedCommandOptions;
use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::{double_quoted, field, ws};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::{map, opt};
use nom::sequence::pair;
use nom::{IResult, Parser};

//   def _regex[_: P]: P[RegexCommand] =
//     "regex" ~ (field ~ ("="|"!=").!).? ~ doubleQuoted map RegexCommand.tupled

#[derive(Debug, Default)]
pub struct RegexParser {}
pub struct RegexCommandOptions {}

impl SplCommandOptions for RegexCommandOptions {}

impl TryFrom<ParsedCommandOptions> for RegexCommandOptions {
    type Error = anyhow::Error;

    fn try_from(value: ParsedCommandOptions) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl SplCommand<ast::RegexCommand> for RegexParser {
    type RootCommand = crate::commands::RegexCommand;
    type Options = RegexCommandOptions;

    fn parse_body(input: &str) -> IResult<&str, ast::RegexCommand> {
        map(
            pair(
                opt(pair(
                    ws(field),
                    map(ws(alt((tag("="), tag("!=")))), |v| v.into()),
                )),
                double_quoted,
            ),
            |(item, regex)| ast::RegexCommand {
                item,
                regex: regex.to_string(),
            },
        )(input)
    }
}
