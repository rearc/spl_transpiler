use crate::ast::ast;
use crate::ast::ast::ParsedCommandOptions;
use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::commands::ConvertCommand;
use crate::spl::{field, token, ws};
use nom::bytes::complete::{tag, tag_no_case};
use nom::combinator::{map, opt};
use nom::multi::many0;
use nom::sequence::{delimited, pair, preceded, tuple};
use nom::IResult;
//
//   // | convert dur2sec(*delay)
//   // convert (timeformat=<string>)? ( (auto|dur2sec|mstime|memk|none|
//   // num|rmunit|rmcomma|ctime|mktime) "(" <field>? ")" (as <field>)?)+
//   def convert[_: P]: P[ConvertCommand] = ("convert" ~
//     commandOptions ~ (token ~~ "(" ~ field ~ ")" ~
//     (W("AS") ~ field).?).map(FieldConversion.tupled).rep) map {
//       case (options, fcs) =>
//         ConvertCommand(
//           options.getString("timeformat", "%m/%d/%Y %H:%M:%S"),
//           fcs
//         )
//     }

#[derive(Debug, Default)]
pub struct ConvertParser {}
pub struct ConvertCommandOptions {
    timeformat: String,
}

impl SplCommandOptions for ConvertCommandOptions {}

impl TryFrom<ParsedCommandOptions> for ConvertCommandOptions {
    type Error = anyhow::Error;

    fn try_from(value: ParsedCommandOptions) -> Result<Self, Self::Error> {
        Ok(Self {
            timeformat: value.get_string("timeformat", "%m/%d/%Y %H:%M:%S")?,
        })
    }
}

impl SplCommand<ast::ConvertCommand> for ConvertParser {
    type RootCommand = ConvertCommand;
    type Options = ConvertCommandOptions;

    fn parse_body(input: &str) -> IResult<&str, ast::ConvertCommand> {
        map(
            pair(
                Self::Options::match_options,
                many0(map(
                    ws(tuple((
                        token,
                        delimited(tag("("), ws(field), tag(")")),
                        ws(opt(preceded(ws(tag_no_case("AS")), field))),
                    ))),
                    |(token, field, as_field)| ast::FieldConversion {
                        func: token.into(),
                        field,
                        alias: as_field,
                    },
                )),
            ),
            |(ConvertCommandOptions { timeformat }, convs)| ast::ConvertCommand {
                timeformat,
                convs,
            },
        )(input)
    }
}
