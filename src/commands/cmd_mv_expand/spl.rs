use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::ast::{Field, ParsedCommandOptions};
use crate::spl::parser::{field, int, ws};
use crate::spl::python::impl_pyclass;
use nom::bytes::complete::{tag, tag_no_case};
use nom::combinator::{map, opt};
use nom::sequence::{pair, preceded};
use nom::IResult;
use pyo3::prelude::*;
//
//   def mvexpand[_: P]: P[MvExpandCommand] = ("mvexpand" ~ field ~ ("limit" ~ "=" ~ int).?) map {
//     case (field, None) => MvExpandCommand(field, None)
//     case (field, Some(limit)) => MvExpandCommand(field, Some(limit.value))
//   }

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct MvExpandCommand {
    #[pyo3(get)]
    pub field: Field,
    #[pyo3(get)]
    pub limit: Option<i64>,
}
impl_pyclass!(MvExpandCommand {
    field: Field,
    limit: Option<i64>
});

#[derive(Debug, Default)]
pub struct MvExpandParser {}
pub struct MvExpandCommandOptions {}

impl SplCommandOptions for MvExpandCommandOptions {}

impl TryFrom<ParsedCommandOptions> for MvExpandCommandOptions {
    type Error = anyhow::Error;

    fn try_from(_value: ParsedCommandOptions) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl SplCommand<MvExpandCommand> for MvExpandParser {
    type RootCommand = crate::commands::MvExpandCommandRoot;
    type Options = MvExpandCommandOptions;

    fn parse_body(input: &str) -> IResult<&str, MvExpandCommand> {
        map(
            pair(
                ws(field),
                opt(preceded(
                    pair(ws(tag_no_case("limit")), ws(tag("="))),
                    ws(int),
                )),
            ),
            |(field, limit_opt)| MvExpandCommand {
                field,
                limit: limit_opt.map(|v| v.0),
            },
        )(input)
    }
}
