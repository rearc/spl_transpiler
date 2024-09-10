use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::ast::{Expr, ParsedCommandOptions};
use crate::spl::parser::{expr, token, ws};
use crate::spl::python::impl_pyclass;
use nom::bytes::complete::tag_no_case;
use nom::combinator::{map, opt};
use nom::sequence::{preceded, tuple};
use nom::IResult;
use pyo3::prelude::*;
//
//   def inputLookup[_: P]: P[InputLookup] =
//     ("inputlookup" ~ commandOptions ~ token ~ ("where" ~ expr).?) map {
//       case (options, tableName, whereOption) =>
//         InputLookup(
//           options.getBoolean("append"),
//           options.getBoolean("strict"),
//           options.getInt("start"),
//           options.getInt("max", 1000000000),
//           tableName,
//           whereOption)
//     }

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct InputLookup {
    #[pyo3(get)]
    pub append: bool,
    #[pyo3(get)]
    pub strict: bool,
    #[pyo3(get)]
    pub start: i64,
    #[pyo3(get)]
    pub max: i64,
    #[pyo3(get)]
    pub table_name: String,
    #[pyo3(get)]
    pub where_expr: Option<Expr>,
}
impl_pyclass!(InputLookup { append: bool, strict: bool, start: i64, max: i64, table_name: String, where_expr: Option<Expr> });

#[derive(Debug, Default)]
pub struct InputLookupParser {}
pub struct InputLookupCommandOptions {
    append: bool,
    strict: bool,
    start: i64,
    max: i64,
}

impl SplCommandOptions for InputLookupCommandOptions {}

impl TryFrom<ParsedCommandOptions> for InputLookupCommandOptions {
    type Error = anyhow::Error;

    fn try_from(value: ParsedCommandOptions) -> Result<Self, Self::Error> {
        Ok(Self {
            append: value.get_boolean("append", false)?,
            strict: value.get_boolean("strict", false)?,
            start: value.get_int("start", 0)?,
            max: value.get_int("max", 1000000000)?,
        })
    }
}

impl SplCommand<InputLookup> for InputLookupParser {
    type RootCommand = crate::commands::InputLookupCommandRoot;
    type Options = InputLookupCommandOptions;

    fn parse_body(input: &str) -> IResult<&str, InputLookup> {
        map(
            tuple((
                Self::Options::match_options,
                ws(token),
                ws(opt(preceded(ws(tag_no_case("where")), expr))),
            )),
            |(options, table_name, where_options)| InputLookup {
                append: options.append,
                strict: options.strict,
                start: options.start,
                max: options.max,
                table_name: table_name.into(),
                where_expr: where_options,
            },
        )(input)
    }
}
