use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::ast::{Expr, ParsedCommandOptions};
use crate::spl::parser::{expr, int, ws};
use crate::spl::python::impl_pyclass;
use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case};
use nom::combinator::{map, opt};
use nom::multi::separated_list1;
use nom::sequence::{pair, preceded, tuple};
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
    #[pyo3(get)]
    pub count: i64,
    #[pyo3(get)]
    pub descending: bool,
}
impl_pyclass!(SortCommand { fields_to_sort: Vec<(Option<String>, Expr)>, count: i64, descending: bool });
impl Default for SortCommand {
    fn default() -> Self {
        SortCommand {
            fields_to_sort: Vec::new(),
            count: 10000,
            descending: false,
        }
    }
}
impl SortCommand {
    /// Simple constructor for creating an unlimited sort command. Note that `.default()` uses the `sort` command's default limit of 10,000.
    pub fn new_simple(fields_to_sort: Vec<(Option<String>, Expr)>) -> Self {
        SortCommand {
            fields_to_sort,
            count: 0,
            descending: false,
        }
    }
}

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
            tuple((
                opt(preceded(opt(ws(tag_no_case("limit="))), ws(int))),
                separated_list1(
                    ws(tag(",")),
                    pair(opt(map(alt((tag("+"), tag("-"))), String::from)), expr),
                ),
                opt(ws(tag_no_case("desc"))),
            )),
            |(count, fields_to_sort, desc)| SortCommand {
                fields_to_sort,
                count: count.map(|v| v.0).unwrap_or(10000),
                descending: desc.is_some(),
            },
        )(input)
    }
}
