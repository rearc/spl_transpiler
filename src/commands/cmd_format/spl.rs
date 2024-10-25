use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::ast::ParsedCommandOptions;
use crate::spl::parser::{double_quoted, ws};
use crate::spl::python::*;
use nom::combinator::{map, opt};
use nom::sequence::{pair, tuple};
use nom::IResult;
use pyo3::prelude::*;
//
//   def format[_: P]: P[FormatCommand] = ("format" ~ commandOptions ~ doubleQuoted.rep(6).?) map {
//     case (kv, options) =>
//       val arguments = options match {
//         case Some(args) => args
//         case _ => Seq("(", "(", "AND", ")", "OR", ")")
//       }
//       FormatCommand(
//         mvSep = kv.getString("mvsep", "OR"),
//         maxResults = kv.getInt("maxresults"),
//         rowPrefix = arguments.head,
//         colPrefix = arguments(1),
//         colSep = arguments(2),
//         colEnd = arguments(3),
//         rowSep = arguments(4),
//         rowEnd = arguments(5)
//       )
//   }

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct FormatCommand {
    #[pyo3(get)]
    pub mv_sep: String,
    #[pyo3(get)]
    pub max_results: i64,
    #[pyo3(get)]
    pub row_prefix: String,
    #[pyo3(get)]
    pub col_prefix: String,
    #[pyo3(get)]
    pub col_sep: String,
    #[pyo3(get)]
    pub col_end: String,
    #[pyo3(get)]
    pub row_sep: String,
    #[pyo3(get)]
    pub row_end: String,
}
impl_pyclass!(FormatCommand {
    mv_sep: String,
    max_results: i64,
    row_prefix: String,
    col_prefix: String,
    col_sep: String,
    col_end: String,
    row_sep: String,
    row_end: String
});

#[derive(Debug, Default)]
pub struct FormatParser {}
pub struct FormatCommandOptions {
    mv_sep: String,
    max_results: i64,
}

impl SplCommandOptions for FormatCommandOptions {}

impl TryFrom<ParsedCommandOptions> for FormatCommandOptions {
    type Error = anyhow::Error;

    fn try_from(value: ParsedCommandOptions) -> Result<Self, Self::Error> {
        Ok(Self {
            mv_sep: value.get_string("mvsep", "OR")?,
            max_results: value.get_int("maxresults", 0)?,
        })
    }
}

impl SplCommand<FormatCommand> for FormatParser {
    type RootCommand = crate::commands::FormatCommandRoot;
    type Options = FormatCommandOptions;

    fn parse_body(input: &str) -> IResult<&str, FormatCommand> {
        map(
            pair(
                Self::Options::match_options,
                opt(tuple((
                    ws(double_quoted),
                    ws(double_quoted),
                    ws(double_quoted),
                    ws(double_quoted),
                    ws(double_quoted),
                    ws(double_quoted),
                ))),
            ),
            |(options, delimiters)| {
                let delimiters = delimiters.unwrap_or(("(", "(", "AND", ")", "OR", ")"));
                FormatCommand {
                    mv_sep: options.mv_sep,
                    max_results: options.max_results,
                    row_prefix: delimiters.0.into(),
                    col_prefix: delimiters.1.into(),
                    col_sep: delimiters.2.into(),
                    col_end: delimiters.3.into(),
                    row_sep: delimiters.4.into(),
                    row_end: delimiters.5.into(),
                }
            },
        )(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    //
    //   test("format maxresults=10") {
    //     p(format(_), FormatCommand(
    //       mvSep = "OR",
    //       maxResults = 10,
    //       rowPrefix = "(",
    //       colPrefix =  "(",
    //       colSep = "AND",
    //       colEnd = ")",
    //       rowSep = "OR",
    //       rowEnd = ")"
    //     ))
    //   }
    #[rstest]
    fn test_format_1() {
        assert_eq!(
            FormatParser::parse(r#"format maxresults=10"#),
            Ok((
                "",
                FormatCommand {
                    mv_sep: "OR".into(),
                    max_results: 10,
                    row_prefix: "(".into(),
                    col_prefix: "(".into(),
                    col_sep: "AND".into(),
                    col_end: ")".into(),
                    row_sep: "OR".into(),
                    row_end: ")".into(),
                }
            ))
        )
    }

    //
    //   test("format mvsep=\"||\" \"[\" \"[\" \"&&\" \"]\" \"||\" \"]\"") {
    //     p(format(_), FormatCommand(
    //       mvSep = "||",
    //       maxResults = 0,
    //       rowPrefix = "[",
    //       colPrefix = "[",
    //       colSep = "&&",
    //       colEnd = "]",
    //       rowSep = "||",
    //       rowEnd = "]"
    //     ))
    //   }
    #[rstest]
    fn test_format_2() {
        assert_eq!(
            FormatParser::parse(r#"format mvsep="||" "[" "[" "&&" "]" "||" "]""#),
            Ok((
                "",
                FormatCommand {
                    mv_sep: "||".into(),
                    max_results: 0,
                    row_prefix: "[".into(),
                    col_prefix: "[".into(),
                    col_sep: "&&".into(),
                    col_end: "]".into(),
                    row_sep: "||".into(),
                    row_end: "]".into(),
                }
            ))
        )
    }
}
