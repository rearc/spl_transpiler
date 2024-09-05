use crate::ast::ast;
use crate::ast::ast::ParsedCommandOptions;
use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::{field, field_and_value, int, ws};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::{map, opt};
use nom::multi::many1;
use nom::sequence::{preceded, tuple};
use nom::{IResult, Parser};
//
//   def _return[_: P]: P[ReturnCommand] = "return" ~ int.? ~ (
//     fieldAndValue.rep(1) | ("$" ~~ field).rep(1) | field.rep(1)) map {
//     case (maybeValue, exprs) =>
//       ReturnCommand(maybeValue.getOrElse(IntValue(1)), exprs map {
//         case fv: FV => Alias(Field(fv.value), fv.field).asInstanceOf[FieldOrAlias]
//         case field: Field => field.asInstanceOf[FieldOrAlias]
//         case a: Any => throw new IllegalArgumentException(s"field $a")
//       })
//   }

#[derive(Debug, Default)]
pub struct ReturnParser {}
pub struct ReturnCommandOptions {}

impl SplCommandOptions for ReturnCommandOptions {}

impl TryFrom<ParsedCommandOptions> for ReturnCommandOptions {
    type Error = anyhow::Error;

    fn try_from(value: ParsedCommandOptions) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl SplCommand<ast::ReturnCommand> for ReturnParser {
    type RootCommand = crate::commands::ReturnCommand;
    type Options = ReturnCommandOptions;

    fn parse_body(input: &str) -> IResult<&str, ast::ReturnCommand> {
        map(
            tuple((
                ws(opt(int)),
                alt((
                    many1(map(ws(field_and_value), |v| {
                        ast::Alias {
                            expr: Box::new(ast::Field::from(v.value).into()),
                            name: v.field,
                        }
                        .into()
                    })),
                    many1(map(ws(preceded(tag("$"), field)), |v| v.into())),
                    many1(map(ws(field), |v| v.into())),
                )),
            )),
            |(maybe_count, fields)| ast::ReturnCommand {
                count: maybe_count.unwrap_or(1.into()),
                fields,
            },
        )(input)
    }
}
