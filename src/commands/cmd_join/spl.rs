use crate::ast::ast;
use crate::ast::ast::ParsedCommandOptions;
use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::{field, sub_search, ws};
use nom::bytes::complete::tag;
use nom::combinator::map;
use nom::multi::separated_list1;
use nom::sequence::tuple;
use nom::{IResult, Parser};

//   def join[_: P]: P[JoinCommand] =
//     ("join" ~ commandOptions ~ field.rep(min = 1, sep = ",") ~ subSearch) map {
//       case (options, fields, pipeline) => JoinCommand(
//         joinType = options.getString("type", "inner"),
//         useTime = options.getBoolean("usetime"),
//         earlier = options.getBoolean("earlier", default = true),
//         overwrite = options.getBoolean("overwrite"),
//         max = options.getInt("max", 1),
//         fields = fields,
//         subSearch = pipeline)
//     }

#[derive(Debug, Default)]
pub struct JoinParser {}
pub struct JoinCommandOptions {
    join_type: String,
    use_time: bool,
    earlier: bool,
    overwrite: bool,
    max: i64,
}

impl SplCommandOptions for JoinCommandOptions {}

impl TryFrom<ParsedCommandOptions> for JoinCommandOptions {
    type Error = anyhow::Error;

    fn try_from(value: ParsedCommandOptions) -> Result<Self, Self::Error> {
        Ok(Self {
            join_type: value.get_string("type", "inner")?,
            use_time: value.get_boolean("usetime", false)?,
            earlier: value.get_boolean("earlier", true)?,
            overwrite: value.get_boolean("overwrite", false)?,
            max: value.get_int("max", 1)?,
        })
    }
}

impl SplCommand<ast::JoinCommand> for JoinParser {
    type RootCommand = crate::commands::JoinCommand;
    type Options = JoinCommandOptions;

    fn parse_body(input: &str) -> IResult<&str, ast::JoinCommand> {
        map(
            tuple((
                Self::Options::match_options,
                separated_list1(ws(tag(",")), field),
                sub_search,
            )),
            |(options, fields, pipeline)| ast::JoinCommand {
                join_type: options.join_type,
                use_time: options.use_time,
                earlier: options.earlier,
                overwrite: options.overwrite,
                max: options.max,
                fields,
                sub_search: pipeline,
            },
        )(input)
    }
}
