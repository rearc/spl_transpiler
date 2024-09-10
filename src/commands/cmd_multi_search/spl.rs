use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::ast::{ParsedCommandOptions, Pipeline};
use crate::spl::parser::{sub_search, ws};
use crate::spl::python::impl_pyclass;
use nom::combinator::map;
use nom::multi::many_m_n;
use nom::IResult;
use pyo3::prelude::*;
/*
//   def multiSearch[_: P]: P[MultiSearch] = "multisearch" ~ subSearch.rep(2) map MultiSearch
pub fn multi_search(input: &str) -> IResult<&str, MultiSearch> {
}
 */

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct MultiSearch {
    #[pyo3(get)]
    pub pipelines: Vec<Pipeline>,
}
impl_pyclass!(MultiSearch {
    pipelines: Vec<Pipeline>
});

#[derive(Debug, Default)]
pub struct MultiSearchParser {}
pub struct MultiSearchCommandOptions {}

impl SplCommandOptions for MultiSearchCommandOptions {}

impl TryFrom<ParsedCommandOptions> for MultiSearchCommandOptions {
    type Error = anyhow::Error;

    fn try_from(_value: ParsedCommandOptions) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl SplCommand<MultiSearch> for MultiSearchParser {
    type RootCommand = crate::commands::MultiSearchCommandRoot;
    type Options = MultiSearchCommandOptions;

    fn parse_body(input: &str) -> IResult<&str, MultiSearch> {
        map(many_m_n(2, usize::MAX, ws(sub_search)), |pipelines| {
            MultiSearch { pipelines }
        })(input)
    }
}
