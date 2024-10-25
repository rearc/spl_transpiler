use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::ast::ParsedCommandOptions;
use crate::spl::python::*;
use nom::combinator::map;
use nom::IResult;
use pyo3::prelude::*;
//
//   def makeResults[_: P]: P[MakeResults] = ("makeresults" ~ commandOptions) map {
//     options =>
//       MakeResults(
//         count = options.getInt("count", 1),
//         annotate = options.getBoolean("annotate"),
//         server = options.getString("splunk_server", "local"),
//         serverGroup = options.getString("splunk_server_group", null)
//       )
//   }

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct MakeResultsCommand {
    #[pyo3(get)]
    pub count: i64,
    #[pyo3(get)]
    pub annotate: bool,
    #[pyo3(get)]
    pub server: String,
    #[pyo3(get)]
    pub server_group: Option<String>,
}
impl_pyclass!(MakeResultsCommand { count: i64, annotate: bool, server: String, server_group: Option<String> });

#[derive(Debug, Default)]
pub struct MakeResultsParser {}
pub struct MakeResultsCommandOptions {
    count: i64,
    annotate: bool,
    server: String,
    server_group: Option<String>,
}

impl SplCommandOptions for MakeResultsCommandOptions {}

impl TryFrom<ParsedCommandOptions> for MakeResultsCommandOptions {
    type Error = anyhow::Error;

    fn try_from(value: ParsedCommandOptions) -> Result<Self, Self::Error> {
        Ok(Self {
            count: value.get_int("count", 1)?,
            annotate: value.get_boolean("annotate", false)?,
            server: value.get_string("splunk_server", "local")?,
            server_group: value.get_string_option("splunk_server_group")?,
        })
    }
}

impl SplCommand<MakeResultsCommand> for MakeResultsParser {
    type RootCommand = crate::commands::MakeResultsCommandRoot;
    type Options = MakeResultsCommandOptions;

    fn parse_body(input: &str) -> IResult<&str, MakeResultsCommand> {
        map(Self::Options::match_options, |options| MakeResultsCommand {
            count: options.count,
            annotate: options.annotate,
            server: options.server,
            server_group: options.server_group,
        })(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spl::parser::command;
    use rstest::rstest;

    //
    //   test("makeresults") {
    //     p(command(_), MakeResults(
    //       count = 1,
    //       annotate = false,
    //       server = "local",
    //       serverGroup = null))
    //   }
    #[rstest]
    fn test_command_makeresults_1() {
        assert_eq!(
            command(r#"makeresults"#),
            Ok((
                "",
                MakeResultsCommand {
                    count: 1,
                    annotate: false,
                    server: "local".into(),
                    server_group: None,
                }
                .into()
            ))
        )
    }

    //
    //   test("makeresults count=10 annotate=t splunk_server_group=group0") {
    //     p(command(_), MakeResults(
    //       count = 10,
    //       annotate = true,
    //       server = "local",
    //       serverGroup = "group0"))
    //   }
    #[rstest]
    fn test_command_makeresults_2() {
        assert_eq!(
            command(r#"makeresults count=10 annotate=t splunk_server_group=group0"#),
            Ok((
                "",
                MakeResultsCommand {
                    count: 10,
                    annotate: true,
                    server: "local".into(),
                    server_group: Some("group0".into()),
                }
                .into()
            ))
        )
    }
}
