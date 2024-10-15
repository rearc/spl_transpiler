//noinspection RsDetachedFile
use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::commands::ConvertCommandRoot;
use crate::spl::ast;
use crate::spl::ast::ParsedCommandOptions;
use crate::spl::parser::{field, token, ws};
use crate::spl::python::*;
use nom::bytes::complete::{tag, tag_no_case};
use nom::combinator::{map, opt};
use nom::multi::many0;
use nom::sequence::{delimited, pair, preceded, tuple};
use nom::{IResult, Parser};
use pyo3::prelude::*;

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct SAMPLECommand {
    // #[pyo3(get)]
    // pub expr: Expr,
}
impl_pyclass!(SAMPLECommand {
    // expr: Expr
});

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

impl SplCommand<SAMPLECommand> for SAMPLEParser {
    type RootCommand = crate::commands::SAMPLECommandRoot;
    type Options = SAMPLECommandOptions;

    fn parse_body(input: &str) -> IResult<&str, SAMPLECommand> {
        bail!("UNIMPLEMENTED")
    }
}
