use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::ast::{Field, ParsedCommandOptions};
use crate::spl::parser::{field, ws};
use crate::spl::python::*;
use nom::combinator::map;
use nom::multi::many0;
use nom::sequence::pair;
use nom::IResult;
use pyo3::prelude::*;
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

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct AddTotalsCommand {
    #[pyo3(get)]
    pub fields: Vec<Field>,
    #[pyo3(get)]
    pub row: bool,
    #[pyo3(get)]
    pub col: bool,
    #[pyo3(get)]
    pub field_name: String,
    #[pyo3(get)]
    pub label_field: Option<String>,
    #[pyo3(get)]
    pub label: String,
}
impl_pyclass!(AddTotalsCommand {
    fields: Vec<Field>,
    row: bool,
    col: bool,
    field_name: String,
    label_field: Option<String>,
    label: String
});

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

impl SplCommand<AddTotalsCommand> for AddTotalsParser {
    type RootCommand = crate::commands::AddTotalsCommandRoot;
    type Options = AddTotalsCommandOptions;

    fn parse_body(input: &str) -> IResult<&str, AddTotalsCommand> {
        map(
            pair(Self::Options::match_options, many0(ws(field))),
            |(options, fields)| AddTotalsCommand {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spl::ast;
    use crate::spl::parser::command;
    use rstest::rstest;

    //
    //   test("addtotals row=t col=f fieldname=num_total num_1 num_2") {
    //     p(command(_), AddTotals(
    //       fields = Seq(Field("num_1"), Field("num_2")),
    //       row = true,
    //       col = false,
    //       fieldName = "num_total",
    //       labelField = null,
    //       label = "Total"
    //     ))
    //   }
    #[rstest]
    fn test_command_addtotals_1() {
        assert_eq!(
            command(r#"addtotals row=t col=f fieldname=num_total num_1 num_2"#),
            Ok((
                "",
                AddTotalsCommand {
                    fields: vec![ast::Field::from("num_1"), ast::Field::from("num_2")],
                    row: true,
                    col: false,
                    field_name: "num_total".into(),
                    label_field: None,
                    label: "Total".into(),
                }
                .into()
            ))
        )
    }
}
