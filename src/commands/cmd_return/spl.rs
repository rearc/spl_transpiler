use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::ast;
use crate::spl::ast::{FieldOrAlias, IntValue, ParsedCommandOptions};
use crate::spl::parser::{field, field_and_value, int, ws};
use crate::spl::python::impl_pyclass;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::{map, opt};
use nom::multi::many1;
use nom::sequence::{preceded, tuple};
use nom::IResult;
use pyo3::prelude::*;
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

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct ReturnCommand {
    #[pyo3(get)]
    pub count: IntValue,
    #[pyo3(get)]
    pub fields: Vec<FieldOrAlias>,
}
impl_pyclass!(ReturnCommand { count: IntValue, fields: Vec<FieldOrAlias> });

#[derive(Debug, Default)]
pub struct ReturnParser {}
pub struct ReturnCommandOptions {}

impl SplCommandOptions for ReturnCommandOptions {}

impl TryFrom<ParsedCommandOptions> for ReturnCommandOptions {
    type Error = anyhow::Error;

    fn try_from(_value: ParsedCommandOptions) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl SplCommand<ReturnCommand> for ReturnParser {
    type RootCommand = crate::commands::ReturnCommandRoot;
    type Options = ReturnCommandOptions;

    fn parse_body(input: &str) -> IResult<&str, ReturnCommand> {
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
            |(maybe_count, fields)| ReturnCommand {
                count: maybe_count.unwrap_or(1.into()),
                fields,
            },
        )(input)
    }
}
