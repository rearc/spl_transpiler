use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::ast::{BoolValue, Expr, ParsedCommandOptions};
use crate::spl::parser::{bool_, expr, int, ws};
use crate::spl::python::*;
use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::combinator::{map, opt};
use nom::sequence::{preceded, tuple};
use nom::IResult;
use pyo3::prelude::*;
//
//   /**
//    * TODO Add condition
//    * TODO: refactor to use command options
//    */
//   def head[_: P]: P[HeadCommand] = ("head" ~ ((int | "limit=" ~ int) | expr)
//     ~ ("keeplast=" ~ bool).?
//     ~ ("null=" ~ bool).?).map(item => {
//     HeadCommand(item._1, item._2.getOrElse(Bool(false)), item._3.getOrElse(Bool(false)))
//   })

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct HeadCommand {
    #[pyo3(get)]
    pub eval_expr: Expr,
    #[pyo3(get)]
    pub keep_last: BoolValue,
    #[pyo3(get)]
    pub null_option: BoolValue,
}
impl_pyclass!(HeadCommand {
    eval_expr: Expr,
    keep_last: BoolValue,
    null_option: BoolValue
});

#[derive(Debug, Default)]
pub struct HeadParser {}
pub struct HeadCommandOptions {}

impl SplCommandOptions for HeadCommandOptions {}

impl TryFrom<ParsedCommandOptions> for HeadCommandOptions {
    type Error = anyhow::Error;

    fn try_from(_value: ParsedCommandOptions) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl SplCommand<HeadCommand> for HeadParser {
    type RootCommand = crate::commands::HeadCommandRoot;
    type Options = HeadCommandOptions;

    fn parse_body(input: &str) -> IResult<&str, HeadCommand> {
        map(
            tuple((
                ws(alt((
                    map(alt((int, preceded(ws(tag_no_case("limit=")), int))), |v| {
                        v.into()
                    }),
                    expr,
                ))),
                ws(opt(preceded(ws(tag_no_case("keeplast=")), bool_))),
                ws(opt(preceded(ws(tag_no_case("null=")), bool_))),
            )),
            |(limit_or_expr, keep_last_opt, null_opt)| HeadCommand {
                eval_expr: limit_or_expr,
                keep_last: keep_last_opt.unwrap_or(false.into()),
                null_option: null_opt.unwrap_or(false.into()),
            },
        )(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spl::ast::*;
    use crate::spl::utils::test::*;
    use rstest::rstest;

    //
    //   test("head 20") {
    //     p(head(_),
    //       HeadCommand(
    //         IntValue(20),
    //         Bool(false),
    //         Bool(false)
    //       )
    //     )
    //   }
    #[rstest]
    fn test_head_limit_1() {
        assert_eq!(
            HeadParser::parse("head 20"),
            Ok((
                "",
                HeadCommand {
                    eval_expr: IntValue(20).into(),
                    keep_last: BoolValue(false),
                    null_option: BoolValue(false),
                }
            ))
        )
    }

    //
    //   test("head limit=400") {
    //     p(head(_),
    //       HeadCommand(
    //         IntValue(400),
    //         Bool(false),
    //         Bool(false))
    //     )
    //   }
    #[rstest]
    fn test_head_limit_2() {
        assert_eq!(
            HeadParser::parse("head limit=400"),
            Ok((
                "",
                HeadCommand {
                    eval_expr: IntValue(400).into(),
                    keep_last: BoolValue(false),
                    null_option: BoolValue(false),
                }
            ))
        )
    }

    //
    //   test("head limit=400 keeplast=true null=false") {
    //     p(head(_),
    //       HeadCommand(
    //         IntValue(400),
    //         Bool(true),
    //         Bool(false)
    //       )
    //     )
    //   }
    #[rstest]
    fn test_head_limit_3() {
        assert_eq!(
            HeadParser::parse("head limit=400 keeplast=true null=false"),
            Ok((
                "",
                HeadCommand {
                    eval_expr: IntValue(400).into(),
                    keep_last: BoolValue(true),
                    null_option: BoolValue(false),
                }
            ))
        )
    }

    //
    //   test("head count>10") {
    //     p(head(_),
    //       HeadCommand(
    //         Binary(
    //           Field("count"),
    //           GreaterThan,
    //           IntValue(10)
    //         ),
    //         Bool(false),
    //         Bool(false)
    //       )
    //     )
    //   }
    #[rstest]
    fn test_head_count_greater_than_10() {
        assert_eq!(
            HeadParser::parse("head count>10"),
            Ok((
                "",
                HeadCommand {
                    eval_expr: _gt(Field("count".to_string()), IntValue(10),),
                    keep_last: BoolValue(false),
                    null_option: BoolValue(false),
                }
            ))
        )
    }

    #[rstest]
    fn test_head_count_greater_than_10_keeplast() {
        assert_eq!(
            HeadParser::parse("head count>=10 keeplast=true"),
            Ok((
                "",
                HeadCommand {
                    eval_expr: _gte(Field("count".to_string()), IntValue(10),),
                    keep_last: BoolValue(true),
                    null_option: BoolValue(false),
                }
            ))
        )
    }
}
