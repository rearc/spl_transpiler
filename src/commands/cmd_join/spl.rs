use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::ast::{Field, ParsedCommandOptions, Pipeline};
use crate::spl::parser::{comma_separated_list1, field, sub_search};
use crate::spl::python::impl_pyclass;
use nom::combinator::map;
use nom::sequence::tuple;
use nom::IResult;
use pyo3::prelude::*;
//   def join[_: P]: P[JoinCommand] =
//     ("join" ~ commandOptions ~ field.rep(min = 1, sep = ",") ~ subSearch) map {
//       case (options, fields, pipeline) => JoinCommand(
//         joinType = options.getString("type", "inner"),
//         useTime = options.getBoolean("usetime"),
//         earlier = options.getBoolean("earlier", default = true),
//         overwrite = options.getBoolean("overwrite"),
//         max = options.getInt("max", 1),
//         fields = fields,
//         subSearch = pipeline)
//     }

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct JoinCommand {
    #[pyo3(get)]
    pub join_type: String,
    #[pyo3(get)]
    pub use_time: bool,
    #[pyo3(get)]
    pub earlier: bool,
    #[pyo3(get)]
    pub overwrite: bool,
    #[pyo3(get)]
    pub max: i64,
    #[pyo3(get)]
    pub fields: Vec<Field>,
    #[pyo3(get)]
    pub sub_search: Pipeline,
}
impl_pyclass!(JoinCommand {
    join_type: String,
    use_time: bool,
    earlier: bool,
    overwrite: bool,
    max: i64,
    fields: Vec<Field>,
    sub_search: Pipeline
});

#[derive(Debug, Default)]
pub struct JoinParser {}
pub struct JoinCommandOptions {
    join_type: String,
    use_time: bool,
    earlier: bool,
    overwrite: bool,
    max: i64,
}

impl SplCommandOptions for JoinCommandOptions {}

impl TryFrom<ParsedCommandOptions> for JoinCommandOptions {
    type Error = anyhow::Error;

    fn try_from(value: ParsedCommandOptions) -> Result<Self, Self::Error> {
        Ok(Self {
            join_type: value.get_string("type", "inner")?,
            use_time: value.get_boolean("usetime", false)?,
            earlier: value.get_boolean("earlier", true)?,
            overwrite: value.get_boolean("overwrite", false)?,
            max: value.get_int("max", 1)?,
        })
    }
}

impl SplCommand<JoinCommand> for JoinParser {
    type RootCommand = crate::commands::JoinCommandRoot;
    type Options = JoinCommandOptions;

    fn parse_body(input: &str) -> IResult<&str, JoinCommand> {
        map(
            tuple((
                Self::Options::match_options,
                comma_separated_list1(field),
                sub_search,
            )),
            |(options, fields, pipeline)| JoinCommand {
                join_type: options.join_type,
                use_time: options.use_time,
                earlier: options.earlier,
                overwrite: options.overwrite,
                max: options.max,
                fields,
                sub_search: pipeline,
            },
        )(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::cmd_rename::spl::RenameCommand;
    use crate::commands::cmd_search::spl::SearchCommand;
    use crate::spl::ast;
    use crate::spl::utils::test::_alias;

    //
    //   test("join product_id [search vendors]") {
    //     p(join(_),
    //       JoinCommand(
    //         joinType = "inner",
    //         useTime = false,
    //         earlier = true,
    //         overwrite = false,
    //         max = 1,
    //         Seq(Field("product_id")),
    //         Pipeline(Seq(
    //           SearchCommand(Field("vendors"))))
    //       )
    //     )
    //   }
    #[test]
    fn test_join_1() {
        assert_eq!(
            JoinParser::parse(r#"join product_id [search vendors]"#),
            Ok((
                "",
                JoinCommand {
                    join_type: "inner".to_string(),
                    use_time: false,
                    earlier: true,
                    overwrite: false,
                    max: 1,
                    fields: vec![ast::Field::from("product_id")],
                    sub_search: ast::Pipeline {
                        commands: vec![SearchCommand {
                            expr: ast::Field::from("vendors").into()
                        }
                        .into()],
                    }
                }
            ))
        )
    }

    //
    //   test("join type=left usetime=true earlier=false " +
    //     "overwrite=false product_id, host, name [search vendors]") {
    //     p(join(_),
    //       JoinCommand(
    //         joinType = "left",
    //         useTime = true,
    //         earlier = false,
    //         overwrite = false,
    //         max = 1,
    //         Seq(
    //           Field("product_id"),
    //           Field("host"),
    //           Field("name")
    //         ),
    //         Pipeline(Seq(
    //           SearchCommand(Field("vendors"))))
    //       )
    //     )
    //   }
    #[test]
    fn test_join_2() {
        assert_eq!(
            JoinParser::parse(
                r#"join type=left usetime=true earlier=false overwrite=false product_id, host, name [search vendors]"#
            ),
            Ok((
                "",
                JoinCommand {
                    join_type: "left".to_string(),
                    use_time: true,
                    earlier: false,
                    overwrite: false,
                    max: 1,
                    fields: vec![
                        ast::Field::from("product_id"),
                        ast::Field::from("host"),
                        ast::Field::from("name")
                    ],
                    sub_search: ast::Pipeline {
                        commands: vec![SearchCommand {
                            expr: ast::Field::from("vendors").into()
                        }
                        .into()],
                    }
                }
            ))
        )
    }

    //
    //   test("join product_id [search vendors | rename pid AS product_id]") {
    //     p(join(_),
    //       JoinCommand(
    //         joinType = "inner",
    //         useTime = false,
    //         earlier = true,
    //         overwrite = false,
    //         max = 1,
    //         Seq(Field("product_id")),
    //         Pipeline(Seq(
    //           SearchCommand(Field("vendors")),
    //           RenameCommand(Seq(
    //             Alias(
    //               Field("pid"),
    //               "product_id"
    //             )))
    //         ))
    //       )
    //     )
    //   }
    #[test]
    fn test_join_3() {
        assert_eq!(
            JoinParser::parse(r#"join product_id [search vendors | rename pid AS product_id]"#),
            Ok((
                "",
                JoinCommand {
                    join_type: "inner".to_string(),
                    use_time: false,
                    earlier: true,
                    overwrite: false,
                    max: 1,
                    fields: vec![ast::Field::from("product_id")],
                    sub_search: ast::Pipeline {
                        commands: vec![
                            SearchCommand {
                                expr: ast::Field::from("vendors").into()
                            }
                            .into(),
                            RenameCommand {
                                alias: vec![_alias("product_id", ast::Field::from("pid"))],
                            }
                            .into()
                        ],
                    }
                }
            ))
        )
    }
}
