use crate::ast::ast;
use crate::ast::ast::ParsedCommandOptions;
use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::{aliased_field, ws};
use nom::bytes::complete::tag;
use nom::combinator::map;
use nom::multi::separated_list1;
use nom::{IResult, Parser};

//
//   def rename[_: P]: P[RenameCommand] =
//     "rename" ~ aliasedField.rep(min = 1, sep = ",") map RenameCommand

#[derive(Debug, Default)]
pub struct RenameParser {}
pub struct RenameCommandOptions {}

impl SplCommandOptions for RenameCommandOptions {}

impl TryFrom<ParsedCommandOptions> for RenameCommandOptions {
    type Error = anyhow::Error;

    fn try_from(value: ParsedCommandOptions) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl SplCommand<ast::RenameCommand> for RenameParser {
    type RootCommand = crate::commands::RenameCommand;
    type Options = RenameCommandOptions;

    fn parse_body(input: &str) -> IResult<&str, ast::RenameCommand> {
        map(separated_list1(ws(tag(",")), aliased_field), |alias| {
            ast::RenameCommand { alias }
        })(input)
    }
}
