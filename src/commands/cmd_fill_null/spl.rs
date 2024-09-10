use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::ast::{Field, ParsedCommandOptions};
use crate::spl::parser::{double_quoted, field, token, ws};
use crate::spl::python::impl_pyclass;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::{into, map, opt};
use nom::multi::many1;
use nom::sequence::{preceded, tuple};
use nom::IResult;
use pyo3::prelude::*;
//
//   def fillNull[_: P]: P[FillNullCommand] = ("fillnull" ~ ("value=" ~~ (doubleQuoted|token)).?
//     ~ field.rep(1).?) map FillNullCommand.tupled

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct FillNullCommand {
    #[pyo3(get)]
    pub value: Option<String>,
    #[pyo3(get)]
    pub fields: Option<Vec<Field>>,
}
impl_pyclass!(FillNullCommand {
    value: Option<String>,
    fields: Option<Vec<Field>>
});

#[derive(Debug, Default)]
pub struct FillNullParser {}
pub struct FillNullCommandOptions {}

impl SplCommandOptions for FillNullCommandOptions {}

impl TryFrom<ParsedCommandOptions> for FillNullCommandOptions {
    type Error = anyhow::Error;

    fn try_from(_value: ParsedCommandOptions) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl SplCommand<FillNullCommand> for FillNullParser {
    type RootCommand = crate::commands::FillNullCommandRoot;
    type Options = FillNullCommandOptions;

    fn parse_body(input: &str) -> IResult<&str, FillNullCommand> {
        map(
            tuple((
                opt(preceded(tag("value="), alt((double_quoted, token)))),
                opt(many1(into(ws(field)))),
            )),
            |(maybe_value, fields)| FillNullCommand {
                value: maybe_value.map(|v| v.to_string()),
                fields,
            },
        )(input)
    }
}
