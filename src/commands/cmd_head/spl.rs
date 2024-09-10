use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::ast::{BoolValue, Expr, ParsedCommandOptions};
use crate::spl::parser::{bool_, expr, int, ws};
use crate::spl::python::impl_pyclass;
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
