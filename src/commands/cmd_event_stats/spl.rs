use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::ast::{Expr, Field, ParsedCommandOptions};
use crate::spl::parser::{field_list0, stats_call, ws};
use crate::spl::python::*;
use nom::bytes::complete::tag_no_case;
use nom::combinator::{map, opt};
use nom::sequence::{preceded, tuple};
use nom::IResult;
use pyo3::prelude::*;
//
//   def eventStats[_: P]: P[EventStatsCommand] = ("eventstats" ~ commandOptions ~ statsCall
//     ~ (W("by") ~ fieldList).?.map(fields => fields.getOrElse(Seq()))).map {
//     case (options, exprs, fields) =>
//       EventStatsCommand(
//         allNum = options.getBoolean("allnum"),
//         funcs = exprs,
//         by = fields
//       )
//   }

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct EventStatsCommand {
    #[pyo3(get)]
    pub all_num: bool,
    #[pyo3(get)]
    pub funcs: Vec<Expr>,
    #[pyo3(get)]
    pub by: Vec<Field>,
}
impl_pyclass!(EventStatsCommand { all_num: bool, funcs: Vec<Expr>, by: Vec<Field> });

#[derive(Debug, Default)]
pub struct EventStatsParser {}
pub struct EventStatsCommandOptions {
    all_num: bool,
}

impl SplCommandOptions for EventStatsCommandOptions {}

impl TryFrom<ParsedCommandOptions> for EventStatsCommandOptions {
    type Error = anyhow::Error;

    fn try_from(value: ParsedCommandOptions) -> Result<Self, Self::Error> {
        Ok(Self {
            all_num: value.get_boolean("allnum", false)?,
        })
    }
}

impl SplCommand<EventStatsCommand> for EventStatsParser {
    type RootCommand = crate::commands::EventStatsCommandRoot;
    type Options = EventStatsCommandOptions;

    fn parse_body(input: &str) -> IResult<&str, EventStatsCommand> {
        map(
            tuple((
                Self::Options::match_options,
                ws(stats_call),
                opt(preceded(ws(tag_no_case("by")), field_list0)),
            )),
            |(options, funcs, by)| EventStatsCommand {
                all_num: options.all_num,
                funcs,
                by: by.unwrap_or(vec![]),
            },
        )(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spl::ast;
    use crate::spl::parser::command;
    use crate::spl::utils::test::*;
    use rstest::rstest;

    //
    //   test("eventstats min(n) by gender") {
    //     p(command(_), EventStatsCommand(
    //       allNum = false,
    //       funcs = Seq(
    //         Call("min", Seq(Field("n")))
    //       ),
    //       by = Seq(Field("gender"))
    //     ))
    //   }
    #[rstest]
    fn test_command_eventstats_1() {
        assert_eq!(
            command(r#"eventstats min(n) by gender"#),
            Ok((
                "",
                EventStatsCommand {
                    all_num: false,
                    funcs: vec![_call!(min(ast::Field::from("n"))).into()],
                    by: vec![ast::Field::from("gender")],
                }
                .into()
            ))
        )
    }
}
