use crate::ast::ast;
use crate::ast::ast::ParsedCommandOptions;
use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::{expr, ws};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::{map, opt};
use nom::multi::separated_list1;
use nom::sequence::pair;
use nom::{IResult, Parser};

//
//   def sort[_: P]: P[SortCommand] =
//     "sort" ~ (("+"|"-").!.? ~~ expr).rep(min = 1, sep = ",") map SortCommand

#[derive(Debug, Default)]
pub struct SortParser {}
pub struct SortCommandOptions {}

impl SplCommandOptions for SortCommandOptions {}

impl TryFrom<ParsedCommandOptions> for SortCommandOptions {
    type Error = anyhow::Error;

    fn try_from(value: ParsedCommandOptions) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl SplCommand<ast::SortCommand> for SortParser {
    type RootCommand = crate::commands::SortCommand;
    type Options = SortCommandOptions;

    fn parse_body(input: &str) -> IResult<&str, ast::SortCommand> {
        map(
            separated_list1(
                ws(tag(",")),
                pair(opt(map(alt((tag("+"), tag("-"))), String::from)), expr),
            ),
            |fields_to_sort| ast::SortCommand { fields_to_sort },
        )(input)
    }
}
