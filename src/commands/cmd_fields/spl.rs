use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::ast::{Field, ParsedCommandOptions};
use crate::spl::parser::{field, ws};
use crate::spl::python::impl_pyclass;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::{map, opt};
use nom::multi::separated_list1;
use nom::sequence::tuple;
use nom::IResult;
use pyo3::prelude::*;
//
//   /*
//    * Function is missing wildcard fields (except when discarding fields ie. fields - myField, ...)
//    */
//   def fields[_: P]: P[FieldsCommand] =
//     "fields" ~ ("+" | "-").!.? ~ field.rep(min = 1, sep = ",") map {
//       case (op, fields) =>
//         if (op.getOrElse("+").equals("-")) {
//           FieldsCommand(removeFields = true, fields)
//         } else {
//           FieldsCommand(removeFields = false, fields)
//         }
//     }

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct FieldsCommand {
    #[pyo3(get)]
    pub remove_fields: bool,
    #[pyo3(get)]
    pub fields: Vec<Field>,
}
impl_pyclass!(FieldsCommand { remove_fields: bool, fields: Vec<Field> });

#[derive(Debug, Default)]
pub struct FieldsParser {}
pub struct FieldsCommandOptions {}

impl SplCommandOptions for FieldsCommandOptions {}

impl TryFrom<ParsedCommandOptions> for FieldsCommandOptions {
    type Error = anyhow::Error;

    fn try_from(_value: ParsedCommandOptions) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl SplCommand<FieldsCommand> for FieldsParser {
    type RootCommand = crate::commands::FieldsCommandRoot;
    type Options = FieldsCommandOptions;

    fn parse_body(input: &str) -> IResult<&str, FieldsCommand> {
        map(
            tuple((
                opt(ws(alt((tag("+"), tag("-"))))),
                separated_list1(ws(tag(",")), field),
            )),
            |(remove_fields_opt, fields)| FieldsCommand {
                remove_fields: remove_fields_opt.unwrap_or("+") == "-",
                fields,
            },
        )(input)
    }
}
