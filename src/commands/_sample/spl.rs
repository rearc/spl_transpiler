use crate::ast::ast;
use crate::ast::ast::ParsedCommandOptions;
use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::commands::ConvertCommand;
use crate::spl::{field, token, ws};
use nom::bytes::complete::{tag, tag_no_case};
use nom::combinator::{map, opt};
use nom::multi::many0;
use nom::sequence::{delimited, pair, preceded, tuple};
use nom::{IResult, Parser};

#[derive(Debug, Default)]
pub struct SAMPLEParser {}
pub struct SAMPLECommandOptions {}

impl SplCommandOptions for SAMPLECommandOptions {}

impl TryFrom<ParsedCommandOptions> for SAMPLECommandOptions {
    type Error = anyhow::Error;

    fn try_from(value: ParsedCommandOptions) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl SplCommand<ast::SAMPLECommand> for SAMPLEParser {
    type RootCommand = crate::commands::SAMPLECommand;
    type Options = SAMPLECommandOptions;

    fn parse_body(input: &str) -> IResult<&str, ast::SAMPLECommand> {
        unimplemented!()
    }
}
