use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::ast::{Field, ParsedCommandOptions};
use crate::spl::parser::{double_quoted, field, ws};
use crate::spl::python::*;
use nom::bytes::complete::{tag, tag_no_case};
use nom::combinator::{map, opt};
use nom::sequence::{pair, preceded};
use nom::IResult;
use pyo3::prelude::*;
//
//   def mvcombine[_: P]: P[MvCombineCommand] = ("mvcombine" ~ ("delim" ~ "=" ~ doubleQuoted).?
//     ~ field) map MvCombineCommand.tupled

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct MvCombineCommand {
    #[pyo3(get)]
    pub delim: Option<String>,
    #[pyo3(get)]
    pub field: Field,
}
impl_pyclass!(MvCombineCommand {
    delim: Option<String>,
    field: Field
});

#[derive(Debug, Default)]
pub struct MvCombineParser {}
pub struct MvCombineCommandOptions {}

impl SplCommandOptions for MvCombineCommandOptions {}

impl TryFrom<ParsedCommandOptions> for MvCombineCommandOptions {
    type Error = anyhow::Error;

    fn try_from(_value: ParsedCommandOptions) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl SplCommand<MvCombineCommand> for MvCombineParser {
    type RootCommand = crate::commands::MvCombineCommandRoot;
    type Options = MvCombineCommandOptions;

    fn parse_body(input: &str) -> IResult<&str, MvCombineCommand> {
        map(
            pair(
                opt(preceded(
                    pair(ws(tag_no_case("delim")), ws(tag("="))),
                    ws(double_quoted),
                )),
                field,
            ),
            |(delim_opt, field)| MvCombineCommand {
                delim: delim_opt.map(Into::into),
                field,
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
    //   test("mvcombine host") {
    //     p(mvcombine(_), MvCombineCommand(
    //       None,
    //       Field("host")
    //     ))
    //   }
    #[rstest]
    fn test_mvcombine_1() {
        assert_eq!(
            MvCombineParser::parse(r#"mvcombine host"#),
            Ok((
                "",
                MvCombineCommand {
                    delim: None,
                    field: ast::Field::from("host"),
                }
            ))
        )
    }

    //
    //   test("mvcombine delim=\",\" host") {
    //     p(mvcombine(_), MvCombineCommand(
    //       Some(","),
    //       Field("host")
    //     ))
    //   }
    #[rstest]
    fn test_mvcombine_2() {
        assert_eq!(
            MvCombineParser::parse(r#"mvcombine delim="," host"#),
            Ok((
                "",
                MvCombineCommand {
                    delim: Some(",".into()),
                    field: ast::Field::from("host"),
                }
            ))
        )
    }
}
