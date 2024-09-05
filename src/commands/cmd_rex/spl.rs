use crate::ast::ast;
use crate::ast::ast::ParsedCommandOptions;
use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::double_quoted;
use nom::combinator::map;
use nom::sequence::pair;
use nom::{IResult, Parser};

//
//   // https://docs.splunk.com/Documentation/Splunk/8.2.2/SearchReference/Rex
//   def rex[_: P]: P[RexCommand] = ("rex" ~ commandOptions ~ doubleQuoted) map {
//     case (kv, regex) =>
//       RexCommand(
//         field = kv.getStringOption("field"),
//         maxMatch = kv.getInt("max_match", 1),
//         offsetField = kv.getStringOption("offset_field"),
//         mode = kv.getStringOption("mode"),
//         regex = regex)
//   }

#[derive(Debug, Default)]
pub struct RexParser {}
pub struct RexCommandOptions {
    field: Option<String>,
    max_match: i64,
    offset_field: Option<String>,
    mode: Option<String>,
}

impl SplCommandOptions for RexCommandOptions {}

impl TryFrom<ParsedCommandOptions> for RexCommandOptions {
    type Error = anyhow::Error;

    fn try_from(value: ParsedCommandOptions) -> Result<Self, Self::Error> {
        Ok(Self {
            field: value.get_string_option("field")?,
            max_match: value.get_int("max_match", 1)?,
            offset_field: value.get_string_option("offset_field")?,
            mode: value.get_string_option("mode")?,
        })
    }
}

impl SplCommand<ast::RexCommand> for RexParser {
    type RootCommand = crate::commands::RexCommand;
    type Options = RexCommandOptions;

    fn parse_body(input: &str) -> IResult<&str, ast::RexCommand> {
        map(
            pair(Self::Options::match_options, double_quoted),
            |(options, regex)| ast::RexCommand {
                field: options.field,
                max_match: options.max_match,
                offset_field: options.offset_field,
                mode: options.mode,
                regex: regex.to_string(),
            },
        )(input)
    }
}
