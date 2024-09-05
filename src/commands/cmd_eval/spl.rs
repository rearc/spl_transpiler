use crate::ast::ast;
use crate::ast::ast::ParsedCommandOptions;
use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::{expr, field, ws};
use nom::bytes::complete::tag;
use nom::combinator::map;
use nom::multi::separated_list0;
use nom::sequence::separated_pair;
use nom::{IResult, Parser};
//
//   def eval[_: P]: P[EvalCommand] = "eval" ~ (field ~ "=" ~ expr).rep(sep = ",") map EvalCommand

#[derive(Debug, Default)]
pub struct EvalParser {}
pub struct EvalCommandOptions {}

impl SplCommandOptions for EvalCommandOptions {}

impl TryFrom<ParsedCommandOptions> for EvalCommandOptions {
    type Error = anyhow::Error;

    fn try_from(value: ParsedCommandOptions) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl SplCommand<ast::EvalCommand> for EvalParser {
    type RootCommand = crate::commands::EvalCommand;
    type Options = EvalCommandOptions;

    fn parse_body(input: &str) -> IResult<&str, ast::EvalCommand> {
        map(
            separated_list0(
                ws(tag(",")),
                separated_pair(ws(field), ws(tag("=")), ws(expr)),
            ),
            |assignments| ast::EvalCommand {
                fields: assignments,
            },
        )(input)
    }
}
