use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::ast::{ParsedCommandOptions, Pipeline};
use crate::spl::parser::quoted_search;
use crate::spl::python::impl_pyclass;
use nom::combinator::map;
use nom::sequence::pair;
use nom::IResult;
use pyo3::prelude::*;
//
//   def _map[_: P]: P[MapCommand] = "map" ~ quotedSearch ~ commandOptions map {
//     case (subPipe, options) => MapCommand(
//       subPipe,
//       options.getInt("maxsearches", 10)
//     )
//   }

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct MapCommand {
    #[pyo3(get)]
    pub search: Pipeline,
    #[pyo3(get)]
    pub max_searches: i64,
}
impl_pyclass!(MapCommand {
    search: Pipeline,
    max_searches: i64
});

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

impl SplCommand<MapCommand> for MapParser {
    type RootCommand = crate::commands::MapCommandRoot;
    type Options = MapCommandOptions;

    fn parse_body(input: &str) -> IResult<&str, MapCommand> {
        map(
            pair(quoted_search, Self::Options::match_options),
            |(subpipe, options)| MapCommand {
                search: subpipe,
                max_searches: options.max_searches,
            },
        )(input)
    }
}
