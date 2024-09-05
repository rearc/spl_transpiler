use crate::ast::ast;
use crate::ast::ast::ParsedCommandOptions;
use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::quoted_search;
use nom::combinator::map;
use nom::sequence::pair;
use nom::{IResult, Parser};

//
//   def _map[_: P]: P[MapCommand] = "map" ~ quotedSearch ~ commandOptions map {
//     case (subPipe, options) => MapCommand(
//       subPipe,
//       options.getInt("maxsearches", 10)
//     )
//   }

#[derive(Debug, Default)]
pub struct MapParser {}
pub struct MapCommandOptions {
    max_searches: i64,
}

impl SplCommandOptions for MapCommandOptions {}

impl TryFrom<ParsedCommandOptions> for MapCommandOptions {
    type Error = anyhow::Error;

    fn try_from(value: ParsedCommandOptions) -> Result<Self, Self::Error> {
        Ok(Self {
            max_searches: value.get_int("maxsearches", 10)?,
        })
    }
}

impl SplCommand<ast::MapCommand> for MapParser {
    type RootCommand = crate::commands::MapCommand;
    type Options = MapCommandOptions;

    fn parse_body(input: &str) -> IResult<&str, ast::MapCommand> {
        map(
            pair(quoted_search, Self::Options::match_options),
            |(subpipe, options)| ast::MapCommand {
                search: subpipe,
                max_searches: options.max_searches,
            },
        )(input)
    }
}
