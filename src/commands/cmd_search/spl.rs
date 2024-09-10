use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::ast::{Expr, ParsedCommandOptions};
use crate::spl::operators::OperatorSymbolTrait;
use crate::spl::parser::{expr, ws};
use crate::spl::python::impl_pyclass;
use crate::spl::{ast, operators};
use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::character::complete::{multispace0, multispace1};
use nom::combinator::{eof, map, verify};
use nom::multi::fold_many_m_n;
use nom::sequence::tuple;
use nom::IResult;
use pyo3::prelude::*;
//   def impliedSearch[_: P]: P[SearchCommand] =
//     "search".? ~ expr.rep(max = 100) map(_.reduce((a, b) => Binary(a, And, b))) map SearchCommand

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct SearchCommand {
    #[pyo3(get)]
    pub expr: Expr,
}
impl_pyclass!(SearchCommand { expr: Expr });

#[derive(Debug, Default)]
pub struct SearchParser {}
pub struct SearchCommandOptions {}

impl SplCommandOptions for SearchCommandOptions {}

impl TryFrom<ParsedCommandOptions> for SearchCommandOptions {
    type Error = anyhow::Error;

    fn try_from(_value: ParsedCommandOptions) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl SplCommand<SearchCommand> for SearchParser {
    type RootCommand = crate::commands::SearchCommandRoot;
    type Options = SearchCommandOptions;

    fn parse_body(input: &str) -> IResult<&str, SearchCommand> {
        map(
            verify(
                fold_many_m_n(
                    1, // <-- differs from original code, but I don't see how 0 makes sense
                    100,
                    ws(expr),
                    || None,
                    |a, b| match a {
                        None => Some(b),
                        Some(a) => Some(Expr::Binary(ast::Binary {
                            left: Box::new(a),
                            symbol: operators::And::SYMBOL.into(),
                            right: Box::new(b),
                        })),
                    },
                ),
                |v| v.is_some(),
            ),
            |v| SearchCommand { expr: v.unwrap() },
        )(input)
    }

    fn match_name(input: &str) -> IResult<&str, ()> {
        alt((
            map(
                tuple((tag_no_case("search"), alt((multispace1, eof)))),
                |_| (),
            ),
            map(multispace0, |_| ()),
        ))(input)
    }
}
