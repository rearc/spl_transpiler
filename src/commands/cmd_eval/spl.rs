use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::ast::{Expr, Field, ParsedCommandOptions};
use crate::spl::parser::{comma_separated_list0, expr, field, ws};
use crate::spl::python::impl_pyclass;
use nom::bytes::complete::tag;
use nom::combinator::map;
use nom::sequence::separated_pair;
use nom::IResult;
use pyo3::prelude::*;
//
//   def eval[_: P]: P[EvalCommand] = "eval" ~ (field ~ "=" ~ expr).rep(sep = ",") map EvalCommand

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct EvalCommand {
    #[pyo3(get)]
    pub fields: Vec<(Field, Expr)>,
}
impl_pyclass!(EvalCommand { fields: Vec<(Field, Expr)> });

#[derive(Debug, Default)]
pub struct EvalParser {}
pub struct EvalCommandOptions {}

impl SplCommandOptions for EvalCommandOptions {}

impl TryFrom<ParsedCommandOptions> for EvalCommandOptions {
    type Error = anyhow::Error;

    fn try_from(_value: ParsedCommandOptions) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl SplCommand<EvalCommand> for EvalParser {
    type RootCommand = crate::commands::EvalCommandRoot;
    type Options = EvalCommandOptions;

    fn parse_body(input: &str) -> IResult<&str, EvalCommand> {
        map(
            comma_separated_list0(separated_pair(ws(field), ws(tag("=")), ws(expr))),
            |assignments| EvalCommand {
                fields: assignments,
            },
        )(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spl::ast;
    use crate::spl::parser::*;
    use crate::spl::utils::test::*;

    //
    //   test("eval mitre_category=\"Discovery\"") {
    //     p(eval(_), EvalCommand(Seq(
    //       (Field("mitre_category"), StrValue("Discovery"))
    //     )))
    //   }
    #[test]
    fn test_eval_1() {
        assert_eq!(
            EvalParser::parse("eval mitre_category=\"Discovery\""),
            Ok((
                "",
                EvalCommand {
                    fields: vec![(
                        ast::Field::from("mitre_category"),
                        ast::StrValue::from("Discovery").into()
                    ),],
                }
            ))
        )
    }

    //
    //   test("eval email_lower=lower(email)") {
    //     p(eval(_), EvalCommand(Seq(
    //       (Field("email_lower"), Call("lower", Seq(Field("email"))))
    //     )))
    //   }
    #[test]
    fn test_eval_2() {
        assert_eq!(
            EvalParser::parse("eval email_lower=lower(email)"),
            Ok((
                "",
                EvalCommand {
                    fields: vec![(
                        ast::Field::from("email_lower"),
                        _call!(lower(ast::Field::from("email"))).into(),
                    ),],
                }
            ))
        )
    }

    //
    //   test("eval replaced=replace(email, \"@.+\", \"\")") {
    //     p(eval(_), EvalCommand(Seq(
    //       (Field("replaced"),
    //         Call("replace", Seq(Field("email"), StrValue("@.+"), StrValue(""))))
    //     )))
    //   }
    #[test]
    fn test_eval_3_args() {
        assert_eq!(
            call("replace(email, \"@.+\", \"\")"),
            Ok((
                "",
                _call!(replace(
                    ast::Field::from("email"),
                    ast::StrValue::from("@.+"),
                    ast::StrValue::from("")
                ))
            ))
        );
        assert_eq!(
            expr("replace(email, \"@.+\", \"\")"),
            Ok((
                "",
                _call!(replace(
                    ast::Field::from("email"),
                    ast::StrValue::from("@.+"),
                    ast::StrValue::from("")
                ))
                .into()
            ))
        );
    }

    #[test]
    fn test_eval_3() {
        assert_eq!(
            EvalParser::parse("eval replaced=replace(email, \"@.+\", \"\")"),
            Ok((
                "",
                EvalCommand {
                    fields: vec![(
                        ast::Field::from("replaced"),
                        _call!(replace(
                            ast::Field::from("email"),
                            ast::StrValue::from("@.+"),
                            ast::StrValue::from("")
                        ))
                        .into()
                    ),],
                }
            ))
        )
    }

    //
    //   test("eval hash_sha256= lower(hash_sha256), b=c") {
    //     p(eval(_), EvalCommand(Seq(
    //       (Field("hash_sha256"), Call("lower", Seq(Field("hash_sha256")))),
    //       (Field("b"), Field("c"))
    //     )))
    //   }
    #[test]
    fn test_eval_4() {
        assert_eq!(
            EvalParser::parse("eval hash_sha256= lower(hash_sha256), b=c"),
            Ok((
                "",
                EvalCommand {
                    fields: vec![
                        (
                            ast::Field::from("hash_sha256"),
                            _call!(lower(ast::Field::from("hash_sha256"))).into(),
                        ),
                        (ast::Field::from("b"), ast::Field::from("c").into()),
                    ],
                }
            ))
        )
    }
}
