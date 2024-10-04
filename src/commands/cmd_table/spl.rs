use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::ast::{Field, ParsedCommandOptions};
use crate::spl::parser::{comma_or_space_separated_list1, field};
use crate::spl::python::impl_pyclass;
use nom::combinator::map;
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
        map(comma_or_space_separated_list1(field), |fields| {
            TableCommand { fields }
        })(input)
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
    fn test_table_1() {
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

    #[test]
    fn test_table_2() {
        assert_eq!(
            pipeline("table _time, dest, user, Operation, EventType, Query, Consumer, Filter"),
            Ok((
                "",
                ast::Pipeline {
                    commands: vec![TableCommand {
                        fields: vec![
                            ast::Field::from("_time"),
                            ast::Field::from("dest"),
                            ast::Field::from("user"),
                            ast::Field::from("Operation"),
                            ast::Field::from("EventType"),
                            ast::Field::from("Query"),
                            ast::Field::from("Consumer"),
                            ast::Field::from("Filter")
                        ]
                    }
                    .into()],
                }
            ))
        )
    }

    #[test]
    fn test_table_3() {
        assert_eq!(
            pipeline("table protoPayload.@type protoPayload.status.details{}.@type protoPayload.status.details{}.violations{}.callerIp protoPayload.status.details{}.violations{}.type protoPayload.status.message"),
            Ok((
                "",
                ast::Pipeline {
                    commands: vec![TableCommand {
                        fields: vec![
                            ast::Field::from("protoPayload.@type"),
                            ast::Field::from("protoPayload.status.details{}.@type"),
                            ast::Field::from("protoPayload.status.details{}.violations{}.callerIp"),
                            ast::Field::from("protoPayload.status.details{}.violations{}.type"),
                            ast::Field::from("protoPayload.status.message")
                        ]
                    }
                    .into()],
                }
            ))
        )
    }
}
