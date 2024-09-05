use crate::ast::ast;
use crate::ast::ast::ParsedCommandOptions;
use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::{field_rep, token, ws};
use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::character::complete::multispace1;
use nom::combinator::{map, opt};
use nom::sequence::{separated_pair, tuple};
use nom::{IResult, Parser};

//
//   def lookupOutput[_: P]: P[LookupOutput] =
//     (W("OUTPUT")|W("OUTPUTNEW")).! ~ fieldRep map LookupOutput.tupled
fn lookup_output(input: &str) -> IResult<&str, ast::LookupOutput> {
    map(
        separated_pair(
            alt((tag_no_case("OUTPUT"), tag_no_case("OUTPUTNEW"))),
            multispace1,
            field_rep,
        ),
        |(kv, fields)| ast::LookupOutput {
            kv: kv.into(),
            fields,
        },
    )(input)
}

//
//   def lookup[_: P]: P[LookupCommand] =
//     "lookup" ~ token ~ fieldRep ~ lookupOutput.? map LookupCommand.tupled

#[derive(Debug, Default)]
pub struct LookupParser {}
pub struct LookupCommandOptions {}

impl SplCommandOptions for LookupCommandOptions {}

impl TryFrom<ParsedCommandOptions> for LookupCommandOptions {
    type Error = anyhow::Error;

    fn try_from(value: ParsedCommandOptions) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl SplCommand<ast::LookupCommand> for LookupParser {
    type RootCommand = crate::commands::LookupCommand;
    type Options = LookupCommandOptions;

    fn parse_body(input: &str) -> IResult<&str, ast::LookupCommand> {
        map(
            tuple((ws(token), ws(field_rep), ws(opt(lookup_output)))),
            |(token, fields, output)| ast::LookupCommand {
                dataset: token.to_string(),
                fields,
                output,
            },
        )(input)
    }
}
