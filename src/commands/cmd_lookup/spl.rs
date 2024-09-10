use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::ast::{FieldLike, ParsedCommandOptions};
use crate::spl::parser::{field_rep, token, ws};
use crate::spl::python::impl_pyclass;
use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::character::complete::multispace1;
use nom::combinator::{map, opt};
use nom::sequence::{separated_pair, tuple};
use nom::IResult;
use pyo3::prelude::*;

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct LookupOutput {
    #[pyo3(get)]
    pub kv: String,
    #[pyo3(get)]
    pub fields: Vec<FieldLike>,
}

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct LookupCommand {
    #[pyo3(get)]
    pub dataset: String,
    #[pyo3(get)]
    pub fields: Vec<FieldLike>,
    #[pyo3(get)]
    pub output: Option<LookupOutput>,
}
impl_pyclass!(LookupOutput { kv: String, fields: Vec<FieldLike> });
impl_pyclass!(LookupCommand { dataset: String, fields: Vec<FieldLike>, output: Option<LookupOutput> });

//
//   def lookupOutput[_: P]: P[LookupOutput] =
//     (W("OUTPUT")|W("OUTPUTNEW")).! ~ fieldRep map LookupOutput.tupled
fn lookup_output(input: &str) -> IResult<&str, LookupOutput> {
    map(
        separated_pair(
            alt((tag_no_case("OUTPUT"), tag_no_case("OUTPUTNEW"))),
            multispace1,
            field_rep,
        ),
        |(kv, fields)| LookupOutput {
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

    fn try_from(_value: ParsedCommandOptions) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl SplCommand<LookupCommand> for LookupParser {
    type RootCommand = crate::commands::LookupCommandRoot;
    type Options = LookupCommandOptions;

    fn parse_body(input: &str) -> IResult<&str, LookupCommand> {
        map(
            tuple((ws(token), ws(field_rep), ws(opt(lookup_output)))),
            |(token, fields, output)| LookupCommand {
                dataset: token.to_string(),
                fields,
                output,
            },
        )(input)
    }
}
