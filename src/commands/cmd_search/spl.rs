use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::ast::{Expr, ParsedCommandOptions};
use crate::spl::operators::OperatorSymbolTrait;
use crate::spl::parser::{expr, ws};
use crate::spl::python::impl_pyclass;
use crate::spl::{ast, operators};
use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::character::complete::{multispace0, multispace1};
use nom::combinator::{eof, map, verify};
use nom::multi::fold_many_m_n;
use nom::sequence::tuple;
use nom::IResult;
use pyo3::prelude::*;
//   def impliedSearch[_: P]: P[SearchCommand] =
//     "search".? ~ expr.rep(max = 100) map(_.reduce((a, b) => Binary(a, And, b))) map SearchCommand

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct SearchCommand {
    #[pyo3(get)]
    pub expr: Expr,
}
impl_pyclass!(SearchCommand { expr: Expr });

#[derive(Debug, Default)]
pub struct SearchParser {}
pub struct SearchCommandOptions {}

impl SplCommandOptions for SearchCommandOptions {}

impl TryFrom<ParsedCommandOptions> for SearchCommandOptions {
    type Error = anyhow::Error;

    fn try_from(_value: ParsedCommandOptions) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl SplCommand<SearchCommand> for SearchParser {
    type RootCommand = crate::commands::SearchCommandRoot;
    type Options = SearchCommandOptions;

    fn parse_body(input: &str) -> IResult<&str, SearchCommand> {
        map(
            verify(
                fold_many_m_n(
                    1, // <-- differs from original code, but I don't see how 0 makes sense
                    100,
                    ws(expr),
                    || None,
                    |a, b| match a {
                        None => Some(b),
                        Some(a) => Some(Expr::Binary(ast::Binary {
                            left: Box::new(a),
                            symbol: operators::And::SYMBOL.into(),
                            right: Box::new(b),
                        })),
                    },
                ),
                |v| v.is_some(),
            ),
            |v| SearchCommand { expr: v.unwrap() },
        )(input)
    }

    fn match_name(input: &str) -> IResult<&str, ()> {
        alt((
            map(
                tuple((tag_no_case("search"), alt((multispace1, eof)))),
                |_| (),
            ),
            map(multispace0, |_| ()),
        ))(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spl::parser::field_in;
    use crate::spl::utils::test::*;

    //   test("search index=dummy host=$host_var$") {
    //     p(search(_), SearchCommand(
    //       Binary(
    //         Binary(
    //           Field("index"),
    //           Equals,
    //           Field("dummy")
    //         ),
    //         And,
    //         Binary(
    //           Field("host"),
    //           Equals,
    //           Variable("host_var")
    //         )
    //       )]
    /// Basic search test; explicit search; index and host
    #[test]
    fn test_search_1() {
        assert_eq!(
            SearchParser::parse("search index=dummy host=$host_var$"),
            Ok((
                "",
                SearchCommand {
                    expr: _and(
                        _eq(ast::Field::from("index"), ast::Field::from("dummy")),
                        _eq(ast::Field::from("host"), ast::Variable::from("host_var"))
                    )
                }
            ))
        );
    }

    //   test("a=b b=c (c=f OR d=t)") {
    //     p(impliedSearch(_), SearchCommand(Binary(
    //       Binary(
    //         Binary(
    //           Field("a"),
    //           Equals,
    //           Field("b")
    //         ),
    //         And,
    //         Binary(
    //           Field("b"),
    //           Equals,
    //           Field("c")
    //         )
    //       ),
    //       And,
    //       Binary(
    //         Binary(
    //           Field("c"),
    //           Equals,
    //           Bool(false)
    //         ),
    //         Or,
    //         Binary(
    //           Field("d"),
    //           Equals,
    //           Bool(true)
    //         )
    //       )
    //     )))
    //   }
    #[test]
    fn test_implied_search_2() {
        assert_eq!(
            SearchParser::parse("a=b b=c (c=f OR d=t)"),
            Ok((
                "",
                SearchCommand {
                    expr: _and(
                        _and(
                            _field_equals("a", ast::Field("b".to_string()).into()),
                            _field_equals("b", ast::Field("c".to_string()).into())
                        ),
                        _or(
                            _field_equals("c", ast::BoolValue(false).into()),
                            _field_equals("d", ast::BoolValue(true).into())
                        )
                    )
                }
            ))
        )
    }

    //   test("code IN(4*, 5*)") {
    //     p(impliedSearch(_), SearchCommand(
    //       FieldIn("code", Seq(
    //         Wildcard("4*"),
    //         Wildcard("5*")
    //       ))))
    //   }
    #[test]
    fn test_implied_search_3() {
        assert_eq!(
            SearchParser::parse("code IN(4*, 5*)"),
            Ok((
                "",
                SearchCommand {
                    expr: ast::FieldIn {
                        field: "code".into(),
                        exprs: vec![
                            ast::Wildcard("4*".to_string()).into(),
                            ast::Wildcard("5*".to_string()).into(),
                        ],
                    }
                    .into()
                }
            ))
        )
    }

    //
    //   test("var_5 IN (str_2 str_3)") {
    //     p(impliedSearch(_), SearchCommand(
    //       FieldIn("var_5", Seq(
    //         Field("str_2"),
    //         Field("str_3")
    //       ))))
    //   }
    #[test]
    fn test_implied_search_4() {
        assert_eq!(
            field_in("var_5 IN (str_2, str_3)"),
            Ok((
                "",
                ast::FieldIn {
                    field: "var_5".into(),
                    exprs: vec![
                        ast::Field::from("str_2").into(),
                        ast::Field::from("str_3").into(),
                    ],
                }
            ))
        );
        assert_eq!(
            SearchParser::parse("var_5 IN (str_2, str_3)"),
            Ok((
                "",
                SearchCommand {
                    expr: ast::FieldIn {
                        field: "var_5".into(),
                        exprs: vec![
                            ast::Field::from("str_2").into(),
                            ast::Field::from("str_3").into(),
                        ],
                    }
                    .into()
                }
            ))
        );
    }

    //
    //   test("NOT code IN(4*, 5*)") {
    //     p(impliedSearch(_), SearchCommand(
    //       Unary(UnaryNot,
    //         FieldIn("code", Seq(
    //           Wildcard("4*"),
    //           Wildcard("5*"))))
    //     ))
    //   }
    #[test]
    fn test_implied_search_5() {
        assert_eq!(
            SearchParser::parse("NOT code IN(4*, 5*)"),
            Ok((
                "",
                SearchCommand {
                    expr: _not(ast::FieldIn {
                        field: "code".into(),
                        exprs: vec![
                            ast::Wildcard("4*".to_string()).into(),
                            ast::Wildcard("5*".to_string()).into(),
                        ],
                    }),
                }
            ))
        );
    }

    //
    //   test("code IN(10, 29, 43) host!=\"localhost\" xqp>5") {
    //     p(impliedSearch(_), SearchCommand(
    //       Binary(
    //         Binary(
    //           FieldIn("code", Seq(
    //             IntValue(10),
    //             IntValue(29),
    //             IntValue(43))),
    //           And,
    //           Binary(
    //             Field("host"),
    //             NotEquals,
    //             StrValue("localhost")
    //           )
    //         ),
    //         And,
    //         Binary(
    //           Field("xqp"),
    //           GreaterThan,
    //           IntValue(5)
    //         )
    //       )
    //     ))
    //   }
    #[test]
    fn test_implied_search_6() {
        assert_eq!(
            SearchParser::parse("code IN(10, 29, 43) host!=\"localhost\" xqp>5"),
            Ok((
                "",
                SearchCommand {
                    expr: _and(
                        _and(
                            ast::FieldIn {
                                field: "code".into(),
                                exprs: vec![
                                    ast::IntValue(10).into(),
                                    ast::IntValue(29).into(),
                                    ast::IntValue(43).into(),
                                ],
                            },
                            _neq(
                                ast::Field("host".to_string()),
                                ast::StrValue("localhost".to_string()),
                            )
                        ),
                        _gt(ast::Field("xqp".to_string()), ast::IntValue(5),)
                    )
                }
            ))
        )
    }

    /// Tests issue with rhs tokens not accepting certain characters
    #[test]
    fn test_search_7() {
        let query = r#"sourcetype=XmlWinEventLog:Microsoft-Windows-Sysmon/Operational OR source=XmlWinEventLog:Microsoft-Windows-Sysmon/Operational OR source=Syslog:Linux-Sysmon/Operational EventCode=6 Signature="Noriyuki MIYAZAKI" OR ImageLoaded= "*\\WinRing0x64.sys""#;
        assert_eq!(
            SearchParser::parse(query),
            Ok((
                "",
                SearchCommand {
                    expr: _and(
                        _and(
                            _or(
                                _eq(
                                    ast::Field::from("sourcetype"),
                                    ast::Field::from(
                                        "XmlWinEventLog:Microsoft-Windows-Sysmon/Operational"
                                    )
                                ),
                                _or(
                                    _eq(
                                        ast::Field::from("source"),
                                        ast::Field::from(
                                            "XmlWinEventLog:Microsoft-Windows-Sysmon/Operational"
                                        )
                                    ),
                                    _eq(
                                        ast::Field::from("source"),
                                        ast::Field::from("Syslog:Linux-Sysmon/Operational")
                                    ),
                                ),
                            ),
                            _eq(ast::Field::from("EventCode"), ast::IntValue::from(6)),
                        ),
                        _or(
                            _eq(
                                ast::Field::from("Signature"),
                                ast::StrValue::from("Noriyuki MIYAZAKI")
                            ),
                            _eq(
                                ast::Field::from("ImageLoaded"),
                                ast::Wildcard::from(r#"*\\WinRing0x64.sys"#)
                            )
                        )
                    )
                }
            ))
        );
    }

    #[test]
    fn test_search_8() {
        let query = r#"search
        query!="SELECT * FROM Win32_ProcessStartTrace WHERE ProcessName = 'wsmprovhost.exe'"
        AND query!="SELECT * FROM __InstanceOperationEvent WHERE TargetInstance ISA 'AntiVirusProduct' OR TargetInstance ISA 'FirewallProduct' OR TargetInstance ISA 'AntiSpywareProduct'"
        "#;

        assert_eq!(
            SearchParser::parse(query),
            Ok((
                "",
                SearchCommand {
                    expr: _and(
                        _neq(
                            ast::Field::from("query"),
                            ast::Wildcard::from("SELECT * FROM Win32_ProcessStartTrace WHERE ProcessName = 'wsmprovhost.exe'")
                        ),
                        _neq(
                            ast::Field::from("query"),
                            ast::Wildcard::from("SELECT * FROM __InstanceOperationEvent WHERE TargetInstance ISA 'AntiVirusProduct' OR TargetInstance ISA 'FirewallProduct' OR TargetInstance ISA 'AntiSpywareProduct'")
                        )
                    )
                }
           ))
        );
    }
}
