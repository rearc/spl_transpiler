use crate::ast::ast;
use crate::ast::ast::ParsedCommandOptions;
use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::{aliased_field, field};
use nom::branch::alt;
use nom::combinator::{into, map};
use nom::sequence::pair;
use nom::{IResult, Parser};

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

#[derive(Debug, Default)]
pub struct BinParser {}
pub struct BinCommandOptions {
    span: Option<ast::TimeSpan>,
    min_span: Option<ast::TimeSpan>,
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
            span: value.get_span_option("span")?.map(|span| match span {
                ast::SplSpan::TimeSpan(s) => s,
            }),
            min_span: value.get_span_option("minspan")?.map(|span| match span {
                ast::SplSpan::TimeSpan(s) => s,
            }),
            bins: value.get_int_option("bins")?,
            start: value.get_int_option("start")?,
            end: value.get_int_option("end")?,
            align_time: value.get_string_option("aligntime")?,
        })
    }
}

impl SplCommand<ast::BinCommand> for BinParser {
    type RootCommand = crate::commands::BinCommand;
    type Options = BinCommandOptions;

    fn parse_body(input: &str) -> IResult<&str, ast::BinCommand> {
        map(
            pair(
                Self::Options::match_options,
                alt((into(aliased_field), into(field))),
            ),
            |(options, field)| ast::BinCommand {
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
