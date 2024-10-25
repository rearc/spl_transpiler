use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::ast::{Field, ParsedCommandOptions};
use crate::spl::parser::{double_quoted, field, token, ws};
use crate::spl::python::*;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::{into, map, opt};
use nom::multi::many1;
use nom::sequence::{preceded, tuple};
use nom::IResult;
use pyo3::prelude::*;
//
//   def fillNull[_: P]: P[FillNullCommand] = ("fillnull" ~ ("value=" ~~ (doubleQuoted|token)).?
//     ~ field.rep(1).?) map FillNullCommand.tupled

const DEFAULT_VALUE: &str = "0";

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct FillNullCommand {
    #[pyo3(get)]
    pub value: String,
    #[pyo3(get)]
    pub fields: Option<Vec<Field>>,
}
impl_pyclass!(FillNullCommand {
    value: String,
    fields: Option<Vec<Field>>
});

#[derive(Debug, Default)]
pub struct FillNullParser {}
pub struct FillNullCommandOptions {}

impl SplCommandOptions for FillNullCommandOptions {}

impl TryFrom<ParsedCommandOptions> for FillNullCommandOptions {
    type Error = anyhow::Error;

    fn try_from(_value: ParsedCommandOptions) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl SplCommand<FillNullCommand> for FillNullParser {
    type RootCommand = crate::commands::FillNullCommandRoot;
    type Options = FillNullCommandOptions;

    fn parse_body(input: &str) -> IResult<&str, FillNullCommand> {
        map(
            tuple((
                opt(preceded(tag("value="), alt((double_quoted, token)))),
                opt(many1(into(ws(field)))),
            )),
            |(maybe_value, fields)| FillNullCommand {
                value: maybe_value
                    .map(|v| v.to_string())
                    .unwrap_or(DEFAULT_VALUE.into()),
                fields,
            },
        )(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spl::ast;
    use rstest::rstest;

    //
    //   test("fillnull") {
    //     p(fillNull(_), FillNullCommand(None, None))
    //   }
    #[rstest]
    fn test_fill_null_1() {
        assert_eq!(
            FillNullParser::parse(r#"fillnull"#),
            Ok((
                "",
                FillNullCommand {
                    value: DEFAULT_VALUE.into(),
                    fields: None,
                }
            ))
        )
    }

    //
    //   test("fillnull value=NA") {
    //     p(fillNull(_), FillNullCommand(Some("NA"), None))
    //   }
    #[rstest]
    fn test_fill_null_2() {
        assert_eq!(
            FillNullParser::parse(r#"fillnull value="NA""#),
            Ok((
                "",
                FillNullCommand {
                    value: "NA".into(),
                    fields: None,
                }
            ))
        )
    }

    //
    //   test("fillnull value=\"NULL\" host port") {
    //     p(fillNull(_), FillNullCommand(
    //       Some("NULL"),
    //       Some(Seq(
    //         Field("host"),
    //         Field("port")
    //       ))))
    //   }
    #[rstest]
    fn test_fill_null_3() {
        assert_eq!(
            FillNullParser::parse(r#"fillnull value="NULL" host port"#),
            Ok((
                "",
                FillNullCommand {
                    value: "NULL".into(),
                    fields: Some(vec![ast::Field::from("host"), ast::Field::from("port"),]),
                }
            ))
        )
    }
}
