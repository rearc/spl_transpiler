use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::ast::*;
use crate::spl::parser::{aliased_field, field};
use crate::spl::python::*;
use nom::branch::alt;
use nom::combinator::{into, map};
use nom::sequence::pair;
use nom::IResult;
use pyo3::prelude::*;
//
//   // bin [<bin-options>...] <field> [AS <newfield>]
//   def bin[_: P]: P[BinCommand] = "bin" ~ commandOptions ~ (aliasedField | field) map {
//     case (options, field) => BinCommand(field,
//       span = options.getSpanOption("span"),
//       minSpan = options.getSpanOption("minspan"),
//       bins = options.getIntOption("bins"),
//       start = options.getIntOption("start"),
//       end = options.getIntOption("end"),
//       alignTime = options.getStringOption("aligntime")
//     )
//   }

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct BinCommand {
    #[pyo3(get)]
    pub field: FieldOrAlias,
    #[pyo3(get)]
    pub span: Option<TimeSpan>,
    #[pyo3(get)]
    pub min_span: Option<TimeSpan>,
    #[pyo3(get)]
    pub bins: Option<i64>,
    #[pyo3(get)]
    pub start: Option<i64>,
    #[pyo3(get)]
    pub end: Option<i64>,
    #[pyo3(get)]
    pub align_time: Option<String>,
}
impl_pyclass!(BinCommand { field: FieldOrAlias, span: Option<TimeSpan>, min_span: Option<TimeSpan>, bins: Option<i64>, start: Option<i64>, end: Option<i64>, align_time: Option<String> });

#[derive(Debug, Default)]
pub struct BinParser {}
pub struct BinCommandOptions {
    span: Option<TimeSpan>,
    min_span: Option<TimeSpan>,
    bins: Option<i64>,
    start: Option<i64>,
    end: Option<i64>,
    align_time: Option<String>,
}

impl SplCommandOptions for BinCommandOptions {}

impl TryFrom<ParsedCommandOptions> for BinCommandOptions {
    type Error = anyhow::Error;

    fn try_from(value: ParsedCommandOptions) -> Result<Self, Self::Error> {
        Ok(Self {
            span: value.get_span_option("span")?,
            min_span: value.get_span_option("minspan")?,
            bins: value.get_int_option("bins")?,
            start: value.get_int_option("start")?,
            end: value.get_int_option("end")?,
            align_time: value.get_string_option("aligntime")?,
        })
    }
}

impl SplCommand<BinCommand> for BinParser {
    type RootCommand = crate::commands::BinCommandRoot;
    type Options = BinCommandOptions;

    fn parse_body(input: &str) -> IResult<&str, BinCommand> {
        map(
            pair(
                Self::Options::match_options,
                alt((into(aliased_field), into(field))),
            ),
            |(options, field)| BinCommand {
                field,
                span: options.span,
                min_span: options.min_span,
                bins: options.bins,
                start: options.start,
                end: options.end,
                align_time: options.align_time,
            },
        )(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spl::parser::command;
    use crate::spl::utils::test::*;
    use rstest::rstest;

    //
    //   test("bin span=30m minspan=5m bins=20 start=0 end=20 aligntime=latest foo AS bar") {
    //     p(command(_), BinCommand(
    //       Alias(Field("foo"), "bar"),
    //       Some(TimeSpan(30, "minutes")),
    //       Some(TimeSpan(5, "minutes")),
    //       Some(20),
    //       Some(0),
    //       Some(20),
    //       Some("latest")))
    //   }
    #[rstest]
    fn test_command_bin_1() {
        assert_eq!(
            command(
                r#"bin span=30m minspan=5m bins=20 start=0 end=20 aligntime=latest foo AS bar"#
            ),
            Ok((
                "",
                BinCommand {
                    field: _alias("bar", Field::from("foo")).into(),
                    span: Some(TimeSpan {
                        value: 30,
                        scale: "minutes".into()
                    }),
                    min_span: Some(TimeSpan {
                        value: 5,
                        scale: "minutes".into()
                    }),
                    bins: Some(20),
                    start: Some(0),
                    end: Some(20),
                    align_time: Some("latest".into()),
                }
                .into()
            ))
        )
    }
}
