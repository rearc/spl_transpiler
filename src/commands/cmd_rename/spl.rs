use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::ast::{Alias, ParsedCommandOptions};
use crate::spl::parser::{aliased_field, ws};
use crate::spl::python::impl_pyclass;
use nom::bytes::complete::tag;
use nom::combinator::map;
use nom::multi::separated_list1;
use nom::IResult;
use pyo3::prelude::*;
//
//   def rename[_: P]: P[RenameCommand] =
//     "rename" ~ aliasedField.rep(min = 1, sep = ",") map RenameCommand

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct RenameCommand {
    #[pyo3(get)]
    pub alias: Vec<Alias>,
}
impl_pyclass!(RenameCommand {
    alias: Vec<Alias>
});

#[derive(Debug, Default)]
pub struct RenameParser {}
pub struct RenameCommandOptions {}

impl SplCommandOptions for RenameCommandOptions {}

impl TryFrom<ParsedCommandOptions> for RenameCommandOptions {
    type Error = anyhow::Error;

    fn try_from(_value: ParsedCommandOptions) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl SplCommand<RenameCommand> for RenameParser {
    type RootCommand = crate::commands::RenameCommandRoot;
    type Options = RenameCommandOptions;

    fn parse_body(input: &str) -> IResult<&str, RenameCommand> {
        map(separated_list1(ws(tag(",")), aliased_field), |alias| {
            RenameCommand { alias }
        })(input)
    }
}
