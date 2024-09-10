use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::ast::{Expr, Field, ParsedCommandOptions};
use crate::spl::parser::{expr, field, ws};
use crate::spl::python::impl_pyclass;
use nom::bytes::complete::tag;
use nom::combinator::map;
use nom::multi::separated_list0;
use nom::sequence::separated_pair;
use nom::IResult;
use pyo3::prelude::*;
//
//   def eval[_: P]: P[EvalCommand] = "eval" ~ (field ~ "=" ~ expr).rep(sep = ",") map EvalCommand

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct EvalCommand {
    #[pyo3(get)]
    pub fields: Vec<(Field, Expr)>,
}
impl_pyclass!(EvalCommand { fields: Vec<(Field, Expr)> });

#[derive(Debug, Default)]
pub struct EvalParser {}
pub struct EvalCommandOptions {}

impl SplCommandOptions for EvalCommandOptions {}

impl TryFrom<ParsedCommandOptions> for EvalCommandOptions {
    type Error = anyhow::Error;

    fn try_from(_value: ParsedCommandOptions) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl SplCommand<EvalCommand> for EvalParser {
    type RootCommand = crate::commands::EvalCommandRoot;
    type Options = EvalCommandOptions;

    fn parse_body(input: &str) -> IResult<&str, EvalCommand> {
        map(
            separated_list0(
                ws(tag(",")),
                separated_pair(ws(field), ws(tag("=")), ws(expr)),
            ),
            |assignments| EvalCommand {
                fields: assignments,
            },
        )(input)
    }
}
