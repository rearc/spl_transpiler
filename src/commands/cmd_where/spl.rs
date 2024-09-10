use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::ast::{Expr, ParsedCommandOptions};
use crate::spl::parser::expr;
use crate::spl::python::impl_pyclass;
use nom::combinator::map;
use nom::IResult;
use pyo3::prelude::*;
//   // where <predicate-expression>
//   def where[_: P]: P[WhereCommand] = "where" ~ expr map WhereCommand

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct WhereCommand {
    #[pyo3(get)]
    pub expr: Expr,
}
impl_pyclass!(WhereCommand { expr: Expr });

#[derive(Debug, Default)]
pub struct WhereParser {}
pub struct WhereCommandOptions {}

impl SplCommandOptions for WhereCommandOptions {}

impl TryFrom<ParsedCommandOptions> for WhereCommandOptions {
    type Error = anyhow::Error;

    fn try_from(_value: ParsedCommandOptions) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl SplCommand<WhereCommand> for WhereParser {
    type RootCommand = crate::commands::WhereCommandRoot;
    type Options = WhereCommandOptions;

    fn parse_body(input: &str) -> IResult<&str, WhereCommand> {
        map(expr, |v| WhereCommand { expr: v })(input)
    }
}
