use crate::ast::ast;
use crate::ast::ast::ParsedCommandOptions;
use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::{field, ws};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::{map, opt};
use nom::multi::separated_list1;
use nom::sequence::tuple;
use nom::{IResult, Parser};

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

#[derive(Debug, Default)]
pub struct FieldsParser {}
pub struct FieldsCommandOptions {}

impl SplCommandOptions for FieldsCommandOptions {}

impl TryFrom<ParsedCommandOptions> for FieldsCommandOptions {
    type Error = anyhow::Error;

    fn try_from(value: ParsedCommandOptions) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl SplCommand<ast::FieldsCommand> for FieldsParser {
    type RootCommand = crate::commands::FieldsCommand;
    type Options = FieldsCommandOptions;

    fn parse_body(input: &str) -> IResult<&str, ast::FieldsCommand> {
        map(
            tuple((
                opt(ws(alt((tag("+"), tag("-"))))),
                separated_list1(ws(tag(",")), field),
            )),
            |(remove_fields_opt, fields)| ast::FieldsCommand {
                remove_fields: remove_fields_opt.unwrap_or("+") == "-",
                fields,
            },
        )(input)
    }
}
