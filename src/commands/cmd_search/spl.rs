use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::ast::{Expr, ParsedCommandOptions};
use crate::spl::operators;
use crate::spl::operators::OperatorSymbolTrait;
use crate::spl::parser::{combine_all_expressions, comma_or_space_separated_list1, expr};
use crate::spl::python::*;
use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::character::complete::{multispace0, multispace1};
use nom::combinator::{eof, map};
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
        map(comma_or_space_separated_list1(expr), |exprs| {
            SearchCommand {
                expr: combine_all_expressions(exprs, operators::And::SYMBOL).unwrap(),
            }
        })(input)
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
    use crate::spl::ast;
    use crate::spl::parser::field_in;
    use crate::spl::utils::test::*;
    use rstest::rstest;

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
    #[rstest]
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
    #[rstest]
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
    #[rstest]
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
    #[rstest]
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
    #[rstest]
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
    #[rstest]
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
    #[rstest]
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

    #[rstest]
    fn test_search_8() {
        let query = r#"search
        query!="SELECT * FROM Win32_ProcessStartTrace WHERE ProcessName = 'wsmprovhost.exe'"
        AND query!="SELECT * FROM __InstanceOperationEvent WHERE TargetInstance ISA 'AntiVirusProduct' OR TargetInstance ISA 'FirewallProduct' OR TargetInstance ISA 'AntiSpywareProduct'"
        "#.trim();

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

    #[rstest]
    fn test_search_9() {
        let query = r#"
            sourcetype=XmlWinEventLog:Microsoft-Windows-Sysmon/Operational
            OR source=XmlWinEventLog:Microsoft-Windows-Sysmon/Operational
            OR source=Syslog:Linux-Sysmon/Operational (process_name=3CXDesktopApp.exe OR OriginalFileName=3CXDesktopApp.exe)  FileVersion=18.12.*"#;

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
                            _or(
                                _eq(
                                    ast::Field::from("process_name"),
                                    ast::Field::from("3CXDesktopApp.exe")
                                ),
                                _eq(
                                    ast::Field::from("OriginalFileName"),
                                    ast::Field::from("3CXDesktopApp.exe")
                                )
                            )
                        ),
                        _eq(
                            ast::Field::from("FileVersion"),
                            ast::Wildcard::from("18.12.*")
                        )
                    )
                }
            ))
        );
    }

    #[rstest]
    fn test_search_10() {
        let query = r#"
        eventtype=wineventlog_security OR Channel=security OR source=XmlWinEventLog:Security
        EventCode=5137 OR (
            EventCode=5136
            AttributeValue!="New Group Policy Object" AND
            (
                AttributeLDAPDisplayName=displayName OR
                AttributeLDAPDisplayName=gPCFileSysPath
            )
        )
        ObjectClass=groupPolicyContainer"#;

        assert_eq!(
            SearchParser::parse(query),
            Ok((
                "",
                SearchCommand {
                    expr: _and(
                        _and(
                            _or(
                                _eq(
                                    ast::Field::from("eventtype"),
                                    ast::Field::from("wineventlog_security")
                                ),
                                _or(
                                    _eq(ast::Field::from("Channel"), ast::Field::from("security")),
                                    _eq(
                                        ast::Field::from("source"),
                                        ast::Field::from("XmlWinEventLog:Security")
                                    ),
                                ),
                            ),
                            _or(
                                _eq(ast::Field::from("EventCode"), ast::IntValue::from(5137)),
                                _and(
                                    _eq(ast::Field::from("EventCode"), ast::IntValue::from(5136)),
                                    _and(
                                        _neq(
                                            ast::Field::from("AttributeValue"),
                                            ast::StrValue::from("New Group Policy Object")
                                        ),
                                        _or(
                                            _eq(
                                                ast::Field::from("AttributeLDAPDisplayName"),
                                                ast::Field::from("displayName")
                                            ),
                                            _eq(
                                                ast::Field::from("AttributeLDAPDisplayName"),
                                                ast::Field::from("gPCFileSysPath")
                                            )
                                        )
                                    )
                                )
                            ),
                        ),
                        _eq(
                            ast::Field::from("ObjectClass"),
                            ast::Field::from("groupPolicyContainer")
                        )
                    ),
                }
            ))
        );
    }

    #[rstest]
    fn test_search_11() {
        let query = "eventtype=wineventlog_security OR Channel=security OR source=XmlWinEventLog:Security EventCode=5136 AttributeLDAPDisplayName IN (\"msDS-AllowedToDelegateTo\",\"msDS-AllowedToActOnBehalfOfOtherIdentity\",\"scriptPath\",\"msTSInitialProgram\") OperationType=%%14674";
        assert_eq!(SearchParser::parse(query).unwrap().0, "");
    }

    #[rstest]
    fn test_search_12() {
        let query = "sourcetype=XmlWinEventLog:Microsoft-Windows-Sysmon/Operational OR source=XmlWinEventLog:Microsoft-Windows-Sysmon/Operational OR source=Syslog:Linux-Sysmon/Operational EventCode=22 QueryName IN (\"*pastebin*\",\"\"*textbin*\"\", \"*ngrok.io*\", \"*discord*\", \"*duckdns.org*\", \"*pasteio.com*\")";
        assert_eq!(SearchParser::parse(query).unwrap().0, "");
    }

    #[rstest]
    fn test_search_13() {
        assert_eq!(
            SearchParser::parse(r#"search [| inputlookup x]"#)
                .unwrap()
                .0,
            ""
        );

        let query = "search [| tstats summariesonly=false allow_old_summaries=true fillnull_value=null count FROM datamodel=Endpoint.Processes where Processes.parent_process_name=cmd.exe Processes.process_name= reg.exe by Processes.parent_process_id Processes.dest Processes.process_name | rename \"Processes\".* AS * | convert timeformat=\"%Y-%m-%dT%H:%M:%S\" ctime(firstTime) | convert timeformat=\"%Y-%m-%dT%H:%M:%S\" ctime(lastTime) | rename parent_process_id as process_id | dedup process_id | table process_id dest]";
        assert_eq!(SearchParser::parse(query).unwrap().0, "");
    }

    #[rstest]
    fn test_search_14() {
        let query = "sourcetype=XmlWinEventLog:Microsoft-Windows-Sysmon/Operational OR source=XmlWinEventLog:Microsoft-Windows-Sysmon/Operational OR source=Syslog:Linux-Sysmon/Operational EventCode = 8 parent_process_name IN (\"powershell_ise.exe\", \"powershell.exe\") TargetImage IN (\"*\\\\svchost.exe\",\"*\\\\csrss.exe\" \"*\\\\gpupdate.exe\", \"*\\\\explorer.exe\",\"*\\\\services.exe\",\"*\\\\winlogon.exe\",\"*\\\\smss.exe\",\"*\\\\wininit.exe\",\"*\\\\userinit.exe\",\"*\\\\spoolsv.exe\",\"*\\\\taskhost.exe\")";
        assert_eq!(SearchParser::parse(query).unwrap().0, "");
    }

    #[rstest]
    fn test_search_15() {
        assert_eq!(field_in("email IN(\"\", \"null\")").unwrap().0, "");
        assert_eq!(expr("(email IN(\"\", \"null\"))").unwrap().0, "");
        assert_eq!(expr("NOT (email IN(\"\", \"null\"))").unwrap().0, "");
        assert_eq!(
            SearchParser::parse("sourcetype=gsuite:drive:json NOT (email IN(\"\", \"null\"))")
                .unwrap()
                .0,
            ""
        );
    }

    #[rstest]
    fn test_search_16() {
        let query = r#"search http_method, "POST""#;
        assert_eq!(
            SearchParser::parse(query),
            Ok((
                "",
                SearchCommand {
                    expr: _and(ast::Field::from("http_method"), ast::StrValue::from("POST"))
                }
            ))
        );
    }
}
