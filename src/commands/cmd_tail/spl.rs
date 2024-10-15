use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::ast::ParsedCommandOptions;
use crate::spl::parser::int;
use crate::spl::python::*;
use nom::combinator::map;
use nom::IResult;
use pyo3::prelude::*;

//   tail [<N>]

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct TailCommand {
    #[pyo3(get)]
    pub n: u64,
}
impl_pyclass!(TailCommand { n: u64 });

#[derive(Debug, Default)]
pub struct TailParser {}
pub struct TailCommandOptions {}

impl SplCommandOptions for TailCommandOptions {}

impl TryFrom<ParsedCommandOptions> for TailCommandOptions {
    type Error = anyhow::Error;

    fn try_from(_value: ParsedCommandOptions) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl SplCommand<TailCommand> for TailParser {
    type RootCommand = crate::commands::TailCommandRoot;
    type Options = TailCommandOptions;

    fn parse_body(input: &str) -> IResult<&str, TailCommand> {
        map(int, |n| TailCommand { n: n.0 as u64 })(input)
    }
}
