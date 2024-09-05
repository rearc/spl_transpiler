use crate::ast::ast;
use crate::ast::ast::ParsedCommandOptions;
use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::{sub_search, ws};
use nom::combinator::map;
use nom::multi::many_m_n;
use nom::IResult;
/*
//   def multiSearch[_: P]: P[MultiSearch] = "multisearch" ~ subSearch.rep(2) map MultiSearch
pub fn multi_search(input: &str) -> IResult<&str, ast::MultiSearch> {
}
 */

#[derive(Debug, Default)]
pub struct MultiSearchParser {}
pub struct MultiSearchCommandOptions {}

impl SplCommandOptions for MultiSearchCommandOptions {}

impl TryFrom<ParsedCommandOptions> for MultiSearchCommandOptions {
    type Error = anyhow::Error;

    fn try_from(value: ParsedCommandOptions) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl SplCommand<ast::MultiSearch> for MultiSearchParser {
    type RootCommand = crate::commands::MultiSearchCommand;
    type Options = MultiSearchCommandOptions;

    fn parse_body(input: &str) -> IResult<&str, ast::MultiSearch> {
        map(many_m_n(2, usize::MAX, ws(sub_search)), |pipelines| {
            ast::MultiSearch { pipelines }
        })(input)
    }
}
