use crate::ast::ast;
use crate::ast::ast::ParsedCommandOptions;
use crate::commands::spl::{SplCommand, SplCommandOptions};
use nom::combinator::map;
use nom::{IResult, Parser};

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

impl SplCommand<ast::MakeResults> for MakeResultsParser {
    type RootCommand = crate::commands::MakeResultsCommand;
    type Options = MakeResultsCommandOptions;

    fn parse_body(input: &str) -> IResult<&str, ast::MakeResults> {
        map(Self::Options::match_options, |options| ast::MakeResults {
            count: options.count,
            annotate: options.annotate,
            server: options.server,
            server_group: options.server_group,
        })(input)
    }
}
