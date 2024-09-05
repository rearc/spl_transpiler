use crate::ast::ast;
use crate::ast::ast::ParsedCommandOptions;
use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::expr;
use nom::combinator::map;
use nom::{IResult, Parser};

//   // where <predicate-expression>
//   def where[_: P]: P[WhereCommand] = "where" ~ expr map WhereCommand

#[derive(Debug, Default)]
pub struct WhereParser {}
pub struct WhereCommandOptions {}

impl SplCommandOptions for WhereCommandOptions {}

impl TryFrom<ParsedCommandOptions> for WhereCommandOptions {
    type Error = anyhow::Error;

    fn try_from(value: ParsedCommandOptions) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl SplCommand<ast::WhereCommand> for WhereParser {
    type RootCommand = crate::commands::WhereCommand;
    type Options = WhereCommandOptions;

    fn parse_body(input: &str) -> IResult<&str, ast::WhereCommand> {
        map(expr, |v| ast::WhereCommand { expr: v })(input)
    }
}
