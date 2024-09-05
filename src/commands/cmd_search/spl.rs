use crate::ast::ast::ParsedCommandOptions;
use crate::ast::operators::OperatorSymbolTrait;
use crate::ast::{ast, operators};
use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::{expr, ws};
use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::character::complete::{multispace0, multispace1};
use nom::combinator::{eof, map, verify};
use nom::multi::fold_many_m_n;
use nom::sequence::{terminated, tuple};
use nom::{IResult, Parser};
//   def impliedSearch[_: P]: P[SearchCommand] =
//     "search".? ~ expr.rep(max = 100) map(_.reduce((a, b) => Binary(a, And, b))) map SearchCommand

#[derive(Debug, Default)]
pub struct SearchParser {}
pub struct SearchCommandOptions {}

impl SplCommandOptions for SearchCommandOptions {}

impl TryFrom<ParsedCommandOptions> for SearchCommandOptions {
    type Error = anyhow::Error;

    fn try_from(value: ParsedCommandOptions) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl SplCommand<ast::SearchCommand> for SearchParser {
    type RootCommand = crate::commands::SearchCommand;
    type Options = SearchCommandOptions;

    fn match_name(input: &str) -> nom::IResult<&str, ()> {
        alt((
            map(
                tuple((tag_no_case("search"), alt((multispace1, eof)))),
                |_| (),
            ),
            map(multispace0, |_| ()),
        ))(input)
    }

    fn parse_body(input: &str) -> IResult<&str, ast::SearchCommand> {
        map(
            verify(
                fold_many_m_n(
                    1, // <-- differs from original code, but I don't see how 0 makes sense
                    100,
                    ws(expr),
                    || None,
                    |a, b| match a {
                        None => Some(b),
                        Some(a) => Some(ast::Expr::Binary(ast::Binary {
                            left: Box::new(a),
                            symbol: operators::And::SYMBOL.into(),
                            right: Box::new(b),
                        })),
                    },
                ),
                |v| v.is_some(),
            ),
            |v| ast::SearchCommand { expr: v.unwrap() },
        )(input)
    }
}
