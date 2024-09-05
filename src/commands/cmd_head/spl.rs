use crate::ast::ast;
use crate::ast::ast::ParsedCommandOptions;
use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::{bool_, expr, int, ws};
use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::combinator::{map, opt};
use nom::sequence::{preceded, tuple};
use nom::{IResult, Parser};

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

#[derive(Debug, Default)]
pub struct HeadParser {}
pub struct HeadCommandOptions {}

impl SplCommandOptions for HeadCommandOptions {}

impl TryFrom<ParsedCommandOptions> for HeadCommandOptions {
    type Error = anyhow::Error;

    fn try_from(value: ParsedCommandOptions) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl SplCommand<ast::HeadCommand> for HeadParser {
    type RootCommand = crate::commands::HeadCommand;
    type Options = HeadCommandOptions;

    fn parse_body(input: &str) -> IResult<&str, ast::HeadCommand> {
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
            |(limit_or_expr, keep_last_opt, null_opt)| ast::HeadCommand {
                eval_expr: limit_or_expr.into(),
                keep_last: keep_last_opt.unwrap_or(false.into()),
                null_option: null_opt.unwrap_or(false.into()),
            },
        )(input)
    }
}
