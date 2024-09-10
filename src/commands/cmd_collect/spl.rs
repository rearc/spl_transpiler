use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::ast::{Field, ParsedCommandOptions};
use crate::spl::parser::field_list;
use crate::spl::python::impl_pyclass;
use anyhow::anyhow;
use nom::combinator::map;
use nom::sequence::pair;
use nom::IResult;
use pyo3::prelude::*;
//
//   def collect[_: P]: P[CollectCommand] = "collect" ~ commandOptions ~ fieldList map {
//     case (cmdOptions, fields) => CollectCommand(
//       index = cmdOptions.getStringOption("index") match {
//         case Some(index) => index
//         case None => throw new Exception("index is mandatory in collect command !")
//       },
//       fields = fields,
//       addTime = cmdOptions.getBoolean("addtime", default = true),
//       file = cmdOptions.getString("file", default = null),
//       host = cmdOptions.getString("host", default = null),
//       marker = cmdOptions.getString("marker", default = null),
//       outputFormat = cmdOptions.getString("output_format", default = "raw"),
//       runInPreview = cmdOptions.getBoolean("run_in_preview"),
//       spool = cmdOptions.getBoolean("spool", default = true),
//       source = cmdOptions.getString("source", default = null),
//       sourceType = cmdOptions.getString("sourcetype", default = null),
//       testMode = cmdOptions.getBoolean("testmode")
//     )
//   }

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct CollectCommand {
    #[pyo3(get)]
    pub index: String,
    #[pyo3(get)]
    pub fields: Vec<Field>,
    #[pyo3(get)]
    pub add_time: bool,
    #[pyo3(get)]
    pub file: Option<String>,
    #[pyo3(get)]
    pub host: Option<String>,
    #[pyo3(get)]
    pub marker: Option<String>,
    #[pyo3(get)]
    pub output_format: String,
    #[pyo3(get)]
    pub run_in_preview: bool,
    #[pyo3(get)]
    pub spool: bool,
    #[pyo3(get)]
    pub source: Option<String>,
    #[pyo3(get)]
    pub source_type: Option<String>,
    #[pyo3(get)]
    pub test_mode: bool,
}
impl_pyclass!(CollectCommand {
    index: String,
    fields: Vec<Field>,
    add_time: bool,
    file: Option<String>,
    host: Option<String>,
    marker: Option<String>,
    output_format: String,
    run_in_preview: bool,
    spool: bool,
    source: Option<String>,
    source_type: Option<String>,
    test_mode: bool
});

#[derive(Debug, Default)]
pub struct CollectParser {}
pub struct CollectCommandOptions {
    index: String,
    add_time: bool,
    file: Option<String>,
    host: Option<String>,
    marker: Option<String>,
    output_format: String,
    run_in_preview: bool,
    spool: bool,
    source: Option<String>,
    source_type: Option<String>,
    test_mode: bool,
}

impl SplCommandOptions for CollectCommandOptions {}

impl TryFrom<ParsedCommandOptions> for CollectCommandOptions {
    type Error = anyhow::Error;

    fn try_from(value: ParsedCommandOptions) -> Result<Self, Self::Error> {
        Ok(Self {
            index: value
                .get_string_option("index")?
                .ok_or(anyhow!("No index provided"))?,
            add_time: value.get_boolean("addtime", true)?,
            file: value.get_string_option("file")?,
            host: value.get_string_option("host")?,
            marker: value.get_string_option("marker")?,
            output_format: value.get_string("output_format", "raw")?,
            run_in_preview: value.get_boolean("run_in_preview", false)?,
            spool: value.get_boolean("spool", true)?,
            source: value.get_string_option("source")?,
            source_type: value.get_string_option("sourcetype")?,
            test_mode: value.get_boolean("testmode", false)?,
        })
    }
}

impl SplCommand<CollectCommand> for CollectParser {
    type RootCommand = crate::commands::CollectCommandRoot;
    type Options = CollectCommandOptions;

    fn parse_body(input: &str) -> IResult<&str, CollectCommand> {
        map(
            pair(Self::Options::match_options, field_list),
            |(
                CollectCommandOptions {
                    index,
                    add_time,
                    file,
                    host,
                    marker,
                    output_format,
                    run_in_preview,
                    spool,
                    source,
                    source_type,
                    test_mode,
                },
                fields,
            )| CollectCommand {
                index,
                add_time,
                file,
                host,
                marker,
                output_format,
                run_in_preview,
                spool,
                source,
                source_type,
                test_mode,
                fields,
            },
        )(input)
    }
}
