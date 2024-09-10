use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::ast::{Field, ParsedCommandOptions};
use crate::spl::parser::{double_quoted, field, ws};
use crate::spl::python::impl_pyclass;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::{map, opt};
use nom::sequence::pair;
use nom::IResult;
use pyo3::prelude::*;
//   def _regex[_: P]: P[RegexCommand] =
//     "regex" ~ (field ~ ("="|"!=").!).? ~ doubleQuoted map RegexCommand.tupled

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct RegexCommand {
    #[pyo3(get)]
    pub item: Option<(Field, String)>,
    #[pyo3(get)]
    pub regex: String,
}
impl_pyclass!(RegexCommand {
    item: Option<(Field, String)>,
    regex: String
});

#[derive(Debug, Default)]
pub struct RegexParser {}
pub struct RegexCommandOptions {}

impl SplCommandOptions for RegexCommandOptions {}

impl TryFrom<ParsedCommandOptions> for RegexCommandOptions {
    type Error = anyhow::Error;

    fn try_from(_value: ParsedCommandOptions) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl SplCommand<RegexCommand> for RegexParser {
    type RootCommand = crate::commands::RegexCommandRoot;
    type Options = RegexCommandOptions;

    fn parse_body(input: &str) -> IResult<&str, RegexCommand> {
        map(
            pair(
                opt(pair(
                    ws(field),
                    map(ws(alt((tag("="), tag("!=")))), |v| v.into()),
                )),
                double_quoted,
            ),
            |(item, regex)| RegexCommand {
                item,
                regex: regex.to_string(),
            },
        )(input)
    }
}
