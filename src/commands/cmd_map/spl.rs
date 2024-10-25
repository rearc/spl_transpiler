use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::ast::{ParsedCommandOptions, Pipeline};
use crate::spl::parser::quoted_search;
use crate::spl::python::*;
use nom::combinator::map;
use nom::sequence::pair;
use nom::IResult;
use pyo3::prelude::*;
//
//   def _map[_: P]: P[MapCommand] = "map" ~ quotedSearch ~ commandOptions map {
//     case (subPipe, options) => MapCommand(
//       subPipe,
//       options.getInt("maxsearches", 10)
//     )
//   }

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct MapCommand {
    #[pyo3(get)]
    pub search: Pipeline,
    #[pyo3(get)]
    pub max_searches: i64,
}
impl_pyclass!(MapCommand {
    search: Pipeline,
    max_searches: i64
});

#[derive(Debug, Default)]
pub struct MapParser {}
pub struct MapCommandOptions {
    max_searches: i64,
}

impl SplCommandOptions for MapCommandOptions {}

impl TryFrom<ParsedCommandOptions> for MapCommandOptions {
    type Error = anyhow::Error;

    fn try_from(value: ParsedCommandOptions) -> Result<Self, Self::Error> {
        Ok(Self {
            max_searches: value.get_int("maxsearches", 10)?,
        })
    }
}

impl SplCommand<MapCommand> for MapParser {
    type RootCommand = crate::commands::MapCommandRoot;
    type Options = MapCommandOptions;

    fn parse_body(input: &str) -> IResult<&str, MapCommand> {
        map(
            pair(quoted_search, Self::Options::match_options),
            |(subpipe, options)| MapCommand {
                search: subpipe,
                max_searches: options.max_searches,
            },
        )(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::cmd_dedup::spl::DedupCommand;
    use crate::commands::cmd_eval::spl::EvalCommand;
    use crate::commands::cmd_search::spl::SearchCommand;
    use crate::commands::cmd_sort::spl::SortCommand;
    use crate::spl::ast;
    use crate::spl::parser::{command, double_quoted_alt, pipeline};
    use crate::spl::utils::test::*;
    use rstest::rstest;

    //
    //   test("map search=\"search index=dummy host=$host_var$\" maxsearches=20") {
    //     p(command(_), MapCommand(
    //       Pipeline(
    //         Seq(
    //           SearchCommand(
    //             Binary(
    //               Binary(
    //                 Field("index"),
    //                 Equals,
    //                 Field("dummy")
    //               ),
    //               And,
    //               Binary(
    //                 Field("host"),
    //                 Equals,
    //                 Variable("host_var")
    //               )
    //             )
    //           )
    //         )
    //       ),
    //       maxSearches = 20))
    //   }
    #[rstest]
    fn test_command_map_1() {
        assert_eq!(
            command(r#"map search="search index=dummy host=$host_var$" maxsearches=20"#),
            Ok((
                "",
                MapCommand {
                    search: ast::Pipeline {
                        commands: vec![SearchCommand {
                            expr: _and(
                                _eq(ast::Field::from("index"), ast::Field::from("dummy"),),
                                _eq(ast::Field::from("host"), ast::Variable::from("host_var"),)
                            )
                        }
                        .into()]
                    },
                    max_searches: 20,
                }
                .into()
            ))
        )
    }

    //
    //   test(
    //     """map search="search index=dummy host=$host_var$ | eval this=\"that\" |
    //       |dedup 10 keepevents=true keepempty=false consecutive=true host ip port"""".stripMargin) {
    //     p(_map(_), MapCommand(
    //       Pipeline(
    //         Seq(
    //           SearchCommand(
    //             Binary(
    //               Binary(
    //                 Field("index"),
    //                 Equals,
    //                 Field("dummy")
    //               ),
    //               And,
    //               Binary(
    //                 Field("host"),
    //                 Equals,
    //                 Variable("host_var")
    //               )
    //             )
    //           ),
    //           EvalCommand(Seq(
    //             (Field("this"), StrValue("that"))
    //           )),
    //           DedupCommand(10,
    //             Seq(Field("host"), Field("ip"), Field("port")),
    //             keepEvents = true,
    //             keepEmpty = false,
    //             consecutive = true,
    //             SortCommand(Seq(
    //               (Some("+"), Field("_no"))
    //             ))
    //           )
    //         )
    //       ),
    //       maxSearches = 10))
    //   }
    #[rstest]
    fn test_quoted_search() {
        assert_eq!(
            double_quoted_alt(r#""search index=\"dummy\"""#),
            Ok(("", r#"search index=\"dummy\""#))
        );
        assert_eq!(
            quoted_search(r#"search="search index=\"dummy\"""#),
            Ok((
                "",
                ast::Pipeline {
                    commands: vec![SearchCommand {
                        expr: _eq(ast::Field::from("index"), ast::StrValue::from("dummy"))
                    }
                    .into()],
                }
            ))
        );
        assert_eq!(
            MapParser::parse(r#"map search="search index=\"dummy\"""#),
            Ok((
                "",
                MapCommand {
                    search: ast::Pipeline {
                        commands: vec![SearchCommand {
                            expr: _eq(ast::Field::from("index"), ast::StrValue::from("dummy"))
                        }
                        .into()],
                    },
                    max_searches: 10,
                }
            ))
        );
        assert_eq!(
            MapParser::parse(r#"map search="search index=dummy""#),
            Ok((
                "",
                MapCommand {
                    search: ast::Pipeline {
                        commands: vec![SearchCommand {
                            expr: _eq(ast::Field::from("index"), ast::Field::from("dummy"))
                        }
                        .into()],
                    },
                    max_searches: 10,
                }
            ))
        );
    }

    #[rstest]
    fn test_map_1() {
        let s = r#"map search="search index=dummy host=$host_var$ | eval this=\"that\" | dedup 10 keepevents=true keepempty=false consecutive=true host ip port""#;
        let _pipeline = ast::Pipeline {
            commands: vec![
                SearchCommand {
                    expr: _and(
                        _eq(ast::Field::from("index"), ast::Field::from("dummy")),
                        _eq(ast::Field::from("host"), ast::Variable::from("host_var")),
                    ),
                }
                .into(),
                EvalCommand {
                    fields: vec![(ast::Field::from("this"), ast::StrValue::from("that").into())],
                }
                .into(),
                DedupCommand {
                    num_results: 10,
                    fields: vec![
                        ast::Field::from("host"),
                        ast::Field::from("ip"),
                        ast::Field::from("port"),
                    ],
                    keep_events: true,
                    keep_empty: false,
                    consecutive: true,
                    sort_by: SortCommand::new_simple(vec![(
                        Some("+".into()),
                        ast::Field::from("_no").into(),
                    )]),
                }
                .into(),
            ],
        };
        assert_eq!(
            pipeline(
                r#"search index=dummy host=$host_var$ | eval this="that" | dedup 10 keepevents=true keepempty=false consecutive=true host ip port"#
            ),
            Ok(("", _pipeline.clone()))
        );
        assert!(quoted_search(r#"search="search index=dummy host=$host_var$ | eval this=\"that\" | dedup 10 keepevents=true keepempty=false consecutive=true host ip port""#).is_ok());
        assert_eq!(
            command(s),
            MapParser::parse(s).map(|(remaining, result)| (remaining, result.into()))
        );
        assert_eq!(
            command(s),
            Ok((
                "",
                MapCommand {
                    search: _pipeline,
                    max_searches: 10,
                }
                .into()
            ))
        )
    }
}
