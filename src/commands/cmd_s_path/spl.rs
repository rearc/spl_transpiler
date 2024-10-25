use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::ast::ParsedCommandOptions;
use crate::spl::parser::{field, string, ws};
use crate::spl::python::*;
use nom::bytes::complete::tag_no_case;
use nom::combinator::{map, opt};
use nom::sequence::{preceded, tuple};
use nom::IResult;
use pyo3::prelude::*;

/*
spath [input=<field>] [output=<field>] [path=<datapath> | <datapath>]

input
    Syntax: input=<field>
    Default: _raw

output
    Syntax: output=<field>
    Default: If you do not specify an output argument, the value for the path argument becomes the field name for the extracted value.

path
    Syntax: path=<datapath> | <datapath>
    Description: The location path to the value that you want to extract. The location path can be specified as path=<datapath> or as just datapath. If you do not specify the path=, the first unlabeled argument is used as the location path. A location path is composed of one or more location steps, separated by periods. An example of this is vendorProductSet.product.desc. A location step is composed of a field name and an optional index surrounded by curly brackets. The index can be an integer, to refer to the position of the data in an array (this differs between JSON and XML), or a string, to refer to an XML attribute. If the index refers to an XML attribute, specify the attribute name with an @ symbol.
 */

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct SPathCommand {
    #[pyo3(get)]
    pub path: String,
    #[pyo3(get)]
    pub input: String,
    #[pyo3(get)]
    pub output: Option<String>,
}
impl_pyclass!(SPathCommand {
    path: String,
    input: String,
    output: Option<String>
});

#[derive(Debug, Default)]
pub struct SPathParser {}
pub struct SPathCommandOptions {
    // pub input: String,
    // pub output: Option<String>,
    // pub path: Option<String>,
}

impl SplCommandOptions for SPathCommandOptions {}

impl TryFrom<ParsedCommandOptions> for SPathCommandOptions {
    type Error = anyhow::Error;

    fn try_from(_value: ParsedCommandOptions) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl SplCommand<SPathCommand> for SPathParser {
    type RootCommand = crate::commands::SPathCommandRoot;
    type Options = SPathCommandOptions;

    fn parse_body(input: &str) -> IResult<&str, SPathCommand> {
        map(
            tuple((
                // Self::Options::match_options,
                opt(ws(preceded(tag_no_case("input="), field))),
                opt(ws(preceded(tag_no_case("output="), field))),
                ws(preceded(opt(tag_no_case("path=")), string)),
            )),
            |(input, output, path)| SPathCommand {
                path: path.into(),
                input: input.map_or("_raw".to_string(), |f| f.0),
                output: output.map(|f| f.0),
            },
        )(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    fn test_spath_1() {
        assert_eq!(
            SPathParser::parse(r#"spath output=myfield path=vendorProductSet.product.desc"#),
            Ok((
                "",
                SPathCommand {
                    input: "_raw".into(),
                    output: Some("myfield".into()),
                    path: "vendorProductSet.product.desc".into(),
                }
            ))
        );
        assert_eq!(
            crate::spl::parser::command(
                r#"spath output=myfield path=vendorProductSet.product.desc"#
            )
            .unwrap()
            .1,
            SPathParser::parse(r#"spath output=myfield path=vendorProductSet.product.desc"#)
                .unwrap()
                .1
                .into(),
        );
    }

    #[rstest]
    fn test_spath_2() {
        assert_eq!(
            SPathParser::parse(r#"spath input=x output=y key.subkey"#),
            Ok((
                "",
                SPathCommand {
                    input: "x".into(),
                    output: Some("y".into()),
                    path: "key.subkey".into(),
                }
            ))
        );
    }
}
