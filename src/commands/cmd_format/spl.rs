use crate::ast::ast;
use crate::ast::ast::ParsedCommandOptions;
use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::{double_quoted, ws};
use nom::combinator::{map, opt};
use nom::sequence::{pair, tuple};
use nom::{IResult, Parser};

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

impl SplCommand<ast::FormatCommand> for FormatParser {
    type RootCommand = crate::commands::FormatCommand;
    type Options = FormatCommandOptions;

    fn parse_body(input: &str) -> IResult<&str, ast::FormatCommand> {
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
                ast::FormatCommand {
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
