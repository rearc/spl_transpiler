use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::ast::{Alias, ParsedCommandOptions};
use crate::spl::parser::{aliased_field, comma_separated_list1};
use crate::spl::python::*;
use nom::combinator::map;
use nom::IResult;
use pyo3::prelude::*;
//
//   def rename[_: P]: P[RenameCommand] =
//     "rename" ~ aliasedField.rep(min = 1, sep = ",") map RenameCommand

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct RenameCommand {
    #[pyo3(get)]
    pub alias: Vec<Alias>,
}
impl_pyclass!(RenameCommand {
    alias: Vec<Alias>
});

#[derive(Debug, Default)]
pub struct RenameParser {}
pub struct RenameCommandOptions {}

impl SplCommandOptions for RenameCommandOptions {}

impl TryFrom<ParsedCommandOptions> for RenameCommandOptions {
    type Error = anyhow::Error;

    fn try_from(_value: ParsedCommandOptions) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl SplCommand<RenameCommand> for RenameParser {
    type RootCommand = crate::commands::RenameCommandRoot;
    type Options = RenameCommandOptions;

    fn parse_body(input: &str) -> IResult<&str, RenameCommand> {
        map(comma_separated_list1(aliased_field), |alias| {
            RenameCommand {
                alias: alias.into_iter().map(Into::into).collect(),
            }
        })(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spl::ast;
    use crate::spl::utils::test::*;
    use rstest::rstest;

    //
    //   test("rename _ip AS IPAddress") {
    //     p(rename(_),
    //       RenameCommand(
    //         Seq(Alias(
    //           Field("_ip"),
    //           "IPAddress"
    //         )))
    //     )
    //   }
    #[rstest]
    fn test_rename_1() {
        assert_eq!(
            RenameParser::parse(r#"rename _ip AS IPAddress"#),
            Ok((
                "",
                RenameCommand {
                    alias: vec![_alias("IPAddress", ast::Field::from("_ip"))],
                }
            ))
        )
    }

    //
    //   test("rename _ip AS IPAddress, _host AS host, _port AS port") {
    //     p(rename(_),
    //       RenameCommand(Seq(
    //         Alias(
    //           Field("_ip"),
    //           "IPAddress"
    //         ),
    //         Alias(
    //           Field("_host"),
    //           "host"
    //         ),
    //         Alias(
    //           Field("_port"),
    //           "port"
    //         )))
    //     )
    //   }
    #[rstest]
    fn test_rename_2() {
        assert_eq!(
            RenameParser::parse(r#"rename _ip AS IPAddress, _host AS host, _port AS port"#),
            Ok((
                "",
                RenameCommand {
                    alias: vec![
                        _alias("IPAddress", ast::Field::from("_ip")),
                        _alias("host", ast::Field::from("_host")),
                        _alias("port", ast::Field::from("_port")),
                    ],
                }
            ))
        )
    }

    //
    //   // Regex not taken into account
    //   test("rename foo* AS bar*") {
    //     p(rename(_),
    //       RenameCommand(
    //         Seq(Alias(
    //           Field("foo*"),
    //           "bar*"
    //         )))
    //     )
    //   }
    #[rstest]
    fn test_rename_3() {
        assert_eq!(
            RenameParser::parse(r#"rename foo* AS bar*"#),
            Ok((
                "",
                RenameCommand {
                    alias: vec![_alias("bar*", ast::Field::from("foo*"))],
                }
            ))
        )
    }

    //
    //   test("rename count AS \"Count of Events\"") {
    //     p(rename(_),
    //       RenameCommand(
    //         Seq(Alias(
    //           Field("count"),
    //           "Count of Events"
    //         )))
    //     )
    //   }
    #[rstest]
    fn test_rename_4() {
        assert_eq!(
            RenameParser::parse(r#"rename count AS "Count of Events""#),
            Ok((
                "",
                RenameCommand {
                    alias: vec![_alias("Count of Events", ast::Field::from("count"))],
                }
            ))
        )
    }

    #[rstest]
    fn test_rename_5() {
        assert_eq!(
            RenameParser::parse(r#"rename "Web".* AS *"#),
            Ok((
                "",
                RenameCommand {
                    alias: vec![_alias("*", ast::Field::from("\"Web\".*")),],
                }
            ))
        )
    }
}
