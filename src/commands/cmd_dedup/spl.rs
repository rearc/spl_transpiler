use crate::ast::ast;
use crate::ast::ast::ParsedCommandOptions;
use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::{field, int, ws};
use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case};
use nom::combinator::{map, opt, verify};
use nom::multi::many1;
use nom::sequence::{pair, preceded, tuple};
use nom::{IResult, Parser};

//
//   /*
//    * Specific field repetition which exclude the term sortby
//    * to avoid any conflict with the sortby command during the parsing
//    */
//   def dedupFieldRep[_: P]: P[Seq[Field]] = field.filter {
//     case Field(myVal) => !myVal.toLowerCase(Locale.ROOT).equals("sortby")
//   }.rep(1)
fn dedup_field_rep(input: &str) -> IResult<&str, Vec<ast::Field>> {
    many1(ws(verify(field, |f| f.0.to_ascii_lowercase() != "sortby")))(input)
}

//
//   def dedup[_: P]: P[DedupCommand] = (
//     "dedup" ~ int.? ~ commandOptions ~ dedupFieldRep
//       ~ ("sortby" ~ (("+"|"-").!.? ~~ field).rep(1)).?) map {
//     case (limit, kv, fields, sortByQuery) =>
//       val sortByCommand = sortByQuery match {
//         case Some(query) => SortCommand(query)
//         case _ => SortCommand(Seq((Some("+"), Field("_no"))))
//       }
//       DedupCommand(
//         numResults = limit.getOrElse(IntValue(1)).value,
//         fields = fields,
//         keepEvents = kv.getBoolean("keepevents"),
//         keepEmpty = kv.getBoolean("keepEmpty"),
//         consecutive = kv.getBoolean("consecutive"),
//         sortByCommand
//       )
//   }

#[derive(Debug, Default)]
pub struct DedupParser {}
pub struct DedupCommandOptions {
    keep_events: bool,
    keep_empty: bool,
    consecutive: bool,
}

impl SplCommandOptions for DedupCommandOptions {}

impl TryFrom<ParsedCommandOptions> for DedupCommandOptions {
    type Error = anyhow::Error;

    fn try_from(value: ParsedCommandOptions) -> Result<Self, Self::Error> {
        Ok(Self {
            keep_events: value.get_boolean("keepevents", false)?,
            keep_empty: value.get_boolean("keepempty", false)?,
            consecutive: value.get_boolean("consecutive", false)?,
        })
    }
}

impl SplCommand<ast::DedupCommand> for DedupParser {
    type RootCommand = crate::commands::DedupCommand;
    type Options = DedupCommandOptions;

    fn parse_body(input: &str) -> IResult<&str, ast::DedupCommand> {
        map(
            tuple((
                opt(int),
                Self::Options::match_options,
                dedup_field_rep,
                opt(preceded(
                    ws(tag_no_case("sortby")),
                    map(
                        many1(ws(pair(
                            opt(map(alt((tag("+"), tag("-"))), Into::into)),
                            map(field, Into::into),
                        ))),
                        |fields_to_sort| ast::SortCommand { fields_to_sort },
                    ),
                )),
            )),
            |(limit, options, fields, sort_by)| ast::DedupCommand {
                // num_results: 0,
                // fields: vec![],
                // keep_events: false,
                // keep_empty: false,
                // consecutive: false,
                // sort_by: SortCommand {},
                num_results: limit.map(|v| v.0).unwrap_or(1),
                fields,
                keep_events: options.keep_events,
                keep_empty: options.keep_empty,
                consecutive: options.consecutive,
                sort_by: sort_by.unwrap_or(ast::SortCommand {
                    fields_to_sort: vec![(Some("+".into()), ast::Field::from("_no").into())],
                }),
            },
        )(input)
    }
}
