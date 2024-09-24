use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::ast::{Field, ParsedCommandOptions};
use crate::spl::parser::{field, ws};
use crate::spl::python::impl_pyclass;
use nom::combinator::map;
use nom::multi::many1;
use nom::IResult;
use pyo3::prelude::*;
//   def table[_: P]: P[TableCommand] = "table" ~ field.rep(1) map TableCommand

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct TableCommand {
    #[pyo3(get)]
    pub fields: Vec<Field>,
}
impl_pyclass!(TableCommand { fields: Vec<Field> });

#[derive(Debug, Default)]
pub struct TableParser {}
pub struct TableCommandOptions {}

impl SplCommandOptions for TableCommandOptions {}

impl TryFrom<ParsedCommandOptions> for TableCommandOptions {
    type Error = anyhow::Error;

    fn try_from(_value: ParsedCommandOptions) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl SplCommand<TableCommand> for TableParser {
    type RootCommand = crate::commands::TableCommandRoot;
    type Options = TableCommandOptions;

    fn parse_body(input: &str) -> IResult<&str, TableCommand> {
        map(many1(ws(field)), |fields| TableCommand { fields })(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spl::ast;
    use crate::spl::parser::pipeline;

    //
    //   test("table foo bar baz*") {
    //     p(pipeline(_), Pipeline(Seq(
    //       TableCommand(Seq(
    //         Field("foo"),
    //         Field("bar"),
    //         Field("baz*")
    //       ))
    //     )))
    //   }
    #[test]
    fn test_pipeline_table_7() {
        assert_eq!(
            pipeline("table foo bar baz*"),
            Ok((
                "",
                ast::Pipeline {
                    commands: vec![TableCommand {
                        fields: vec![
                            ast::Field::from("foo"),
                            ast::Field::from("bar"),
                            ast::Field::from("baz*")
                        ]
                    }
                    .into()],
                }
            ))
        )
    }
}
