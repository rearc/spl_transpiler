use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::ast::{Field, ParsedCommandOptions};
use crate::spl::parser::{field, token, ws};
use crate::spl::python::impl_pyclass;
use nom::bytes::complete::{tag, tag_no_case};
use nom::combinator::{map, opt};
use nom::multi::many0;
use nom::sequence::{delimited, pair, preceded, tuple};
use nom::IResult;
use pyo3::prelude::*;
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

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct FieldConversion {
    #[pyo3(get)]
    pub func: String,
    // TODO: This should be either a Wildcard or a Field
    #[pyo3(get)]
    pub field: Field,
    #[pyo3(get)]
    pub alias: Option<Field>,
}

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct ConvertCommand {
    #[pyo3(get)]
    pub timeformat: String,
    #[pyo3(get)]
    pub convs: Vec<FieldConversion>,
}
impl_pyclass!(FieldConversion { func: String, field: Field, alias: Option<Field> });
impl_pyclass!(ConvertCommand { timeformat: String, convs: Vec<FieldConversion> });

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

impl SplCommand<ConvertCommand> for ConvertParser {
    type RootCommand = crate::commands::ConvertCommandRoot;
    type Options = ConvertCommandOptions;

    fn parse_body(input: &str) -> IResult<&str, ConvertCommand> {
        map(
            pair(
                Self::Options::match_options,
                many0(map(
                    ws(tuple((
                        token,
                        delimited(tag("("), ws(field), tag(")")),
                        ws(opt(preceded(ws(tag_no_case("AS")), field))),
                    ))),
                    |(token, field, as_field)| FieldConversion {
                        func: token.into(),
                        field,
                        alias: as_field,
                    },
                )),
            ),
            |(ConvertCommandOptions { timeformat }, convs)| ConvertCommand { timeformat, convs },
        )(input)
    }
}
