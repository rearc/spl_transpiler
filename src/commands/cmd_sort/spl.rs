use crate::ast::ast::{Expr, ParsedCommandOptions};
use crate::ast::python::impl_pyclass;
use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::{expr, ws};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::{map, opt};
use nom::multi::separated_list1;
use nom::sequence::pair;
use nom::IResult;
use pyo3::prelude::*;
//
//   def sort[_: P]: P[SortCommand] =
//     "sort" ~ (("+"|"-").!.? ~~ expr).rep(min = 1, sep = ",") map SortCommand

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct SortCommand {
    #[pyo3(get)]
    pub fields_to_sort: Vec<(Option<String>, Expr)>,
}
impl_pyclass!(SortCommand { fields_to_sort: Vec<(Option<String>, Expr)> });

#[derive(Debug, Default)]
pub struct SortParser {}
pub struct SortCommandOptions {}

impl SplCommandOptions for SortCommandOptions {}

impl TryFrom<ParsedCommandOptions> for SortCommandOptions {
    type Error = anyhow::Error;

    fn try_from(_value: ParsedCommandOptions) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl SplCommand<SortCommand> for SortParser {
    type RootCommand = crate::commands::SortCommandRoot;
    type Options = SortCommandOptions;

    fn parse_body(input: &str) -> IResult<&str, SortCommand> {
        map(
            separated_list1(
                ws(tag(",")),
                pair(opt(map(alt((tag("+"), tag("-"))), String::from)), expr),
            ),
            |fields_to_sort| SortCommand { fields_to_sort },
        )(input)
    }
}
