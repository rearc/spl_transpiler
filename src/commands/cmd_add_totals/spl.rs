use crate::ast::ast;
use crate::ast::ast::ParsedCommandOptions;
use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::{field, ws};
use nom::combinator::map;
use nom::multi::many0;
use nom::sequence::pair;
use nom::{IResult, Parser};

//
//   def cAddtotals[_: P]: P[AddTotals] = "addtotals" ~ commandOptions ~ field.rep(1).? map {
//     case (options: CommandOptions, fields: Option[Seq[Field]]) =>
//       AddTotals(
//         fields.getOrElse(Seq.empty[Field]),
//         row = options.getBoolean("row", default = true),
//         col = options.getBoolean("col"),
//         fieldName = options.getString("fieldname", "Total"),
//         labelField = options.getString("labelfield", null),
//         label = options.getString("label", "Total")
//       )
//   }

#[derive(Debug, Default)]
pub struct AddTotalsParser {}
pub struct AddTotalsCommandOptions {
    row: bool,
    col: bool,
    field_name: String,
    label_field: Option<String>,
    label: String,
}

impl SplCommandOptions for AddTotalsCommandOptions {}

impl TryFrom<ParsedCommandOptions> for AddTotalsCommandOptions {
    type Error = anyhow::Error;

    fn try_from(value: ParsedCommandOptions) -> Result<Self, Self::Error> {
        Ok(Self {
            row: value.get_boolean("row", true)?,
            col: value.get_boolean("col", false)?,
            field_name: value.get_string("fieldname", "Total")?,
            label_field: value.get_string_option("labelfield")?,
            label: value.get_string("label", "Total")?,
        })
    }
}

impl SplCommand<ast::AddTotals> for AddTotalsParser {
    type RootCommand = crate::commands::AddTotalsCommand;
    type Options = AddTotalsCommandOptions;

    fn parse_body(input: &str) -> IResult<&str, ast::AddTotals> {
        map(
            pair(Self::Options::match_options, many0(ws(field))),
            |(options, fields)| ast::AddTotals {
                fields,
                row: options.row,
                col: options.col,
                field_name: options.field_name,
                label_field: options.label_field,
                label: options.label,
            },
        )(input)
    }
}
