use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::ast::{Expr, ParsedCommandOptions};
use crate::spl::parser::{expr, token, ws};
use crate::spl::python::*;
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
pub struct InputLookupCommand {
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
impl_pyclass!(InputLookupCommand { append: bool, strict: bool, start: i64, max: i64, table_name: String, where_expr: Option<Expr> });

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

impl SplCommand<InputLookupCommand> for InputLookupParser {
    type RootCommand = crate::commands::InputLookupCommandRoot;
    type Options = InputLookupCommandOptions;

    fn parse_body(input: &str) -> IResult<&str, InputLookupCommand> {
        map(
            tuple((
                Self::Options::match_options,
                ws(token),
                ws(opt(preceded(ws(tag_no_case("where")), expr))),
            )),
            |(options, table_name, where_options)| InputLookupCommand {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spl::ast;
    use crate::spl::utils::test::*;
    use rstest::rstest;

    //
    //   test("inputlookup append=t strict=f myTable where test_id=11") {
    //     p(inputLookup(_), InputLookup(
    //       append = true,
    //       strict = false,
    //       start = 0,
    //       max = 1000000000,
    //       "myTable",
    //       Some(
    //         Binary(
    //           Field("test_id"),
    //           Equals,
    //           IntValue(11)
    //         )
    //     )))
    //   }
    #[rstest]
    fn test_input_lookup_1() {
        assert_eq!(
            InputLookupParser::parse(r#"inputlookup append=t strict=f myTable where test_id=11"#),
            Ok((
                "",
                InputLookupCommand {
                    append: true,
                    strict: false,
                    start: 0,
                    max: 1000000000,
                    table_name: "myTable".into(),
                    where_expr: Some(_eq(ast::Field::from("test_id"), ast::IntValue(11),)),
                }
            ))
        )
    }

    //
    //   test("inputlookup myTable") {
    //     p(inputLookup(_), InputLookup(
    //       append = false,
    //       strict = false,
    //       start = 0,
    //       max = 1000000000,
    //       "myTable",
    //       None
    //     ))
    //   }
    #[rstest]
    fn test_input_lookup_2() {
        assert_eq!(
            InputLookupParser::parse(r#"inputlookup myTable"#),
            Ok((
                "",
                InputLookupCommand {
                    append: false,
                    strict: false,
                    start: 0,
                    max: 1000000000,
                    table_name: "myTable".into(),
                    where_expr: None,
                }
            ))
        )
    }
}
