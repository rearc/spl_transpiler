// use crate::spl::ast;
use crate::spl::parser::{expr, token, ws};
use nom::bytes::complete::{tag, take_until};
use nom::character::complete::none_of;
use nom::combinator::{all_consuming, into, map, opt, recognize};
use nom::multi::{many0, separated_list0};
use nom::sequence::{delimited, pair, terminated};
use nom::IResult;
use pyo3::pyclass;

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct MacroCall {
    #[pyo3(get)]
    pub macro_name: String,
    #[pyo3(get)]
    pub args: Vec<(Option<String>, String)>,
}

type ChunkedQuery<'a> = (Vec<(&'a str, MacroCall)>, &'a str);

pub fn spl_macros(input: &str) -> IResult<&str, ChunkedQuery> {
    all_consuming(pair(
        many0(pair(
            take_until("`"),
            delimited(
                tag("`"),
                ws(map(
                    pair(
                        ws(token),
                        opt(delimited(
                            ws(tag("(")),
                            separated_list0(
                                ws(tag(",")),
                                pair(
                                    opt(into(terminated(token, tag("=")))),
                                    into(recognize(expr)),
                                ),
                            ),
                            ws(tag(")")),
                        )),
                    ),
                    |(name, args)| MacroCall {
                        macro_name: name.to_string(),
                        args: args.unwrap_or(Vec::new()),
                    },
                )),
                tag("`"),
            ),
        )),
        recognize(many0(none_of("`"))),
    ))(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    fn test_spl_macros_simple_no_args() {
        let input = r#"`foo`"#;
        let result = spl_macros(input).unwrap();
        assert_eq!(
            result,
            (
                "",
                (
                    vec![(
                        "",
                        MacroCall {
                            macro_name: "foo".to_string(),
                            args: Vec::new()
                        }
                    )],
                    ""
                )
            )
        );
    }

    #[rstest]
    fn test_spl_macros_simple_with_args() {
        let input = r#"`foo(bar, 1, "s")`"#;
        let result = spl_macros(input).unwrap();
        assert_eq!(
            result,
            (
                "",
                (
                    vec![(
                        "",
                        MacroCall {
                            macro_name: "foo".to_string(),
                            args: vec![
                                (None, "bar".to_string()),
                                (None, "1".to_string()),
                                (None, "\"s\"".to_string()),
                            ]
                        }
                    )],
                    ""
                )
            )
        );
    }

    #[rstest]
    fn test_spl_macros_simple_named_args() {
        let input = r#"`foo(foo=bar, baz=1)`"#;
        let result = spl_macros(input).unwrap();
        assert_eq!(
            result,
            (
                "",
                (
                    vec![(
                        "",
                        MacroCall {
                            macro_name: "foo".to_string(),
                            args: vec![
                                (Some("foo".to_string()), "bar".to_string()),
                                (Some("baz".to_string()), "1".to_string()),
                            ]
                        }
                    )],
                    ""
                )
            )
        );
    }

    #[rstest]
    fn test_spl_macros_multiple() {
        let input = r#"index=main | `foo(bar, 1, "s")` x=`f` y=3"#;
        let result = spl_macros(input).unwrap();
        assert_eq!(
            result,
            (
                "",
                (
                    vec![
                        (
                            "index=main | ",
                            MacroCall {
                                macro_name: "foo".to_string(),
                                args: vec![
                                    (None, "bar".to_string()),
                                    (None, "1".to_string()),
                                    (None, "\"s\"".to_string()),
                                ]
                            }
                        ),
                        (
                            " x=",
                            MacroCall {
                                macro_name: "f".to_string(),
                                args: Vec::new(),
                            }
                        )
                    ],
                    " y=3"
                )
            )
        );
    }

    #[rstest]
    fn test_spl_macros_quoted_backtick() {
        let input = r#"`foo("`")`"#;
        let result = spl_macros(input).unwrap();
        assert_eq!(
            result,
            (
                "",
                (
                    vec![(
                        "",
                        MacroCall {
                            macro_name: "foo".to_string(),
                            args: vec![(None, "\"`\"".to_string()),]
                        }
                    )],
                    ""
                )
            )
        );
    }
}
