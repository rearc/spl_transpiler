use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::ast::{Field, ParsedCommandOptions, Pipeline};
use crate::spl::parser::{field_list1, sub_search};
use crate::spl::python::*;
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
    // TODO: While apparently a rarely used feature, you can do
    //  `join left=L right=R where L.f1=R.f2 [...]`, which we should support eventually
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
            overwrite: value.get_boolean("overwrite", true)?,
            max: value.get_int("max", 1)?,
        })
    }
}

impl SplCommand<JoinCommand> for JoinParser {
    type RootCommand = crate::commands::JoinCommandRoot;
    type Options = JoinCommandOptions;

    fn parse_body(input: &str) -> IResult<&str, JoinCommand> {
        map(
            tuple((Self::Options::match_options, field_list1, sub_search)),
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
    use crate::commands::stats_utils::MaybeSpannedField;
    use crate::spl::ast;
    use crate::spl::utils::test::*;
    use rstest::rstest;

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
    #[rstest]
    fn test_join_1() {
        assert_eq!(
            JoinParser::parse(r#"join product_id [search vendors]"#),
            Ok((
                "",
                JoinCommand {
                    join_type: "inner".to_string(),
                    use_time: false,
                    earlier: true,
                    overwrite: true,
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
    #[rstest]
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
    #[rstest]
    fn test_join_3() {
        assert_eq!(
            JoinParser::parse(r#"join product_id [search vendors | rename pid AS product_id]"#),
            Ok((
                "",
                JoinCommand {
                    join_type: "inner".to_string(),
                    use_time: false,
                    earlier: true,
                    overwrite: true,
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

    #[rstest]
    fn test_join_4() {
        let query = r#"join process_guid, _time [
            | tstats
                summariesonly=false allow_old_summaries=true fillnull_value=null
                count min(_time) as firstTime max(_time) as lastTime
                FROM datamodel=Endpoint.Filesystem
                where Filesystem.file_path IN ("*\\HttpProxy\\owa\\auth\\*", "*\\inetpub\\wwwroot\\aspnet_client\\*", "*\\HttpProxy\\OAB\\*") Filesystem.file_name="*.aspx"
                by _time span=1h Filesystem.dest Filesystem.file_create_time Filesystem.file_name Filesystem.file_path
            | rename Filesystem.* AS *
            | fields _time dest file_create_time file_name file_path process_name process_path process process_guid
            ]"#;
        assert_eq!(
            JoinParser::parse(query),
            Ok((
                "",
                JoinCommand {
                    join_type: "inner".to_string(),
                    use_time: false,
                    earlier: true,
                    overwrite: true,
                    max: 1,
                    fields: vec![ast::Field::from("process_guid"), ast::Field::from("_time"),],
                    sub_search: Pipeline {
                        commands: vec![
                            crate::commands::cmd_t_stats::spl::TStatsCommand {
                                prestats: false,
                                local: false,
                                append: false,
                                summaries_only: false,
                                include_reduced_buckets: false,
                                allow_old_summaries: true,
                                chunk_size: 10000000,
                                fillnull_value: Some("null".into()),
                                exprs: vec![
                                    _call!(count()).into(),
                                    _alias("firstTime", _call!(min(ast::Field::from("_time"))))
                                        .into(),
                                    _alias("lastTime", _call!(max(ast::Field::from("_time"))))
                                        .into(),
                                ],
                                datamodel: Some("Endpoint.Filesystem".into()),
                                nodename: None,
                                where_condition: Some(_and(
                                    _isin(
                                        "Filesystem.file_path",
                                        vec![
                                            ast::Wildcard::from(r#"*\\HttpProxy\\owa\\auth\\*"#)
                                                .into(),
                                            ast::Wildcard::from(
                                                r#"*\\inetpub\\wwwroot\\aspnet_client\\*"#
                                            )
                                            .into(),
                                            ast::Wildcard::from(r#"*\\HttpProxy\\OAB\\*"#).into(),
                                        ]
                                    ),
                                    _eq(
                                        ast::Field::from("Filesystem.file_name"),
                                        ast::Wildcard::from("*.aspx")
                                    )
                                )),
                                by_fields: Some(vec![
                                    MaybeSpannedField {
                                        field: ast::Field::from("_time"),
                                        span: Some(ast::TimeSpan {
                                            value: 1,
                                            scale: "hours".to_string()
                                        }),
                                    },
                                    ast::Field::from("Filesystem.dest").into(),
                                    ast::Field::from("Filesystem.file_create_time").into(),
                                    ast::Field::from("Filesystem.file_name").into(),
                                    ast::Field::from("Filesystem.file_path").into(),
                                ]),
                                by_prefix: None,
                            }
                            .into(),
                            crate::commands::cmd_rename::spl::RenameCommand {
                                alias: vec![_alias("*", ast::Field::from("Filesystem.*")),],
                            }
                            .into(),
                            crate::commands::cmd_fields::spl::FieldsCommand {
                                remove_fields: false,
                                fields: vec![
                                    ast::Field::from("_time"),
                                    ast::Field::from("dest"),
                                    ast::Field::from("file_create_time"),
                                    ast::Field::from("file_name"),
                                    ast::Field::from("file_path"),
                                    ast::Field::from("process_name"),
                                    ast::Field::from("process_path"),
                                    ast::Field::from("process"),
                                    ast::Field::from("process_guid"),
                                ],
                            }
                            .into(),
                        ],
                    },
                }
            ))
        );
    }
}
