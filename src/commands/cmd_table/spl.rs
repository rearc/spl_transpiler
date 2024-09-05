use crate::ast::ast;
use crate::ast::ast::ParsedCommandOptions;
use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::{field, ws};
use nom::combinator::map;
use nom::multi::many1;
use nom::{IResult, Parser};

//   def table[_: P]: P[TableCommand] = "table" ~ field.rep(1) map TableCommand

#[derive(Debug, Default)]
pub struct TableParser {}
pub struct TableCommandOptions {}

impl SplCommandOptions for TableCommandOptions {}

impl TryFrom<ParsedCommandOptions> for TableCommandOptions {
    type Error = anyhow::Error;

    fn try_from(value: ParsedCommandOptions) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl SplCommand<ast::TableCommand> for TableParser {
    type RootCommand = crate::commands::TableCommand;
    type Options = TableCommandOptions;

    fn parse_body(input: &str) -> IResult<&str, ast::TableCommand> {
        map(many1(ws(field)), |fields| ast::TableCommand { fields })(input)
    }
}
