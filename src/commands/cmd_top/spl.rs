use crate::commands::spl::{SplCommand, SplCommandOptions};
use crate::spl::ast::{Field, ParsedCommandOptions};
use crate::spl::parser::{field_list, int, ws};
use crate::spl::python::impl_pyclass;
use nom::bytes::complete::tag_no_case;
use nom::combinator::{map, opt};
use nom::sequence::{preceded, tuple};
use nom::IResult;
use pyo3::prelude::*;

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct TopCommand {
    #[pyo3(get)]
    pub fields: Vec<Field>,
    #[pyo3(get)]
    pub n: u64,
    #[pyo3(get)]
    pub by: Option<Vec<Field>>,
    #[pyo3(get)]
    pub count_field: String,
    // #[pyo3(get)]
    // pub limit: u64,
    #[pyo3(get)]
    pub other_str: String,
    #[pyo3(get)]
    pub percent_field: String,
    #[pyo3(get)]
    pub show_count: bool,
    #[pyo3(get)]
    pub show_percent: bool,
    #[pyo3(get)]
    pub use_other: bool,
}
impl_pyclass!(TopCommand {
    fields: Vec<Field>,
    n: u64,
    by: Option<Vec<Field>>,
    count_field: String,
    other_str: String,
    percent_field: String,
    show_count: bool,
    show_percent: bool,
    use_other: bool
});

#[derive(Debug, Default)]
pub struct TopParser {}
pub struct TopCommandOptions {
    count_field: String,
    limit: u64,
    other_str: String,
    percent_field: String,
    show_count: bool,
    show_percent: bool,
    use_other: bool,
}

impl SplCommandOptions for TopCommandOptions {}

/*
countfield
Syntax: countfield=<string>
Description: For each value returned by the top command, the results also return a count of the events that have that value. This argument specifies the name of the field that contains the count. The count is returned by default. If you do not want to return the count of events, specify showcount=false.
Default: count
limit
Syntax: limit=<int>
Description: Specifies how many results to return. To return all values, specify zero ( 0 ). Specifying top limit=<int> is the same as specifying top N.
Default: 10
otherstr
Syntax: otherstr=<string>
Description: If useother=true, a row representing all other values is added to the results. Use otherstr=<string> to specify the name of the label for the row.
Default: OTHER
percentfield
Syntax: percentfield=<string>
Description: For each value returned by the top command, the results also return a percentage of the events that have that value. This argument specifies the name of the field that contains the percentage. The percentage is returned by default. If you do not want to return the percentage of events, specify showperc=false.
Default: percent
showcount
Syntax: showcount=<bool>
Description: Specify whether to create a field called "count" (see "countfield" option) with the count of that tuple.
Default: true
showperc
Syntax: showperc=<bool>
Description: Specify whether to create a field called "percent" (see "percentfield" option) with the relative prevalence of that tuple.
Default: true
useother
Syntax: useother=<bool>
Description: Specify whether or not to add a row that represents all values not included due to the limit cutoff.
Default: false
 */

impl TryFrom<ParsedCommandOptions> for TopCommandOptions {
    type Error = anyhow::Error;

    fn try_from(value: ParsedCommandOptions) -> Result<Self, Self::Error> {
        Ok(Self {
            count_field: value.get_string("countfield", "count")?,
            limit: value.get_int("limit", 10)? as u64,
            other_str: value.get_string("otherstr", "OTHER")?,
            percent_field: value.get_string("percentfield", "percent")?,
            show_count: value.get_boolean("showcount", true)?,
            show_percent: value.get_boolean("showperc", true)?,
            use_other: value.get_boolean("useother", false)?,
        })
    }
}

impl SplCommand<TopCommand> for TopParser {
    type RootCommand = crate::commands::TopCommandRoot;
    type Options = TopCommandOptions;

    fn parse_body(input: &str) -> IResult<&str, TopCommand> {
        map(
            tuple((
                ws(opt(int)),
                ws(Self::Options::match_options),
                ws(field_list),
                ws(opt(preceded(ws(tag_no_case("BY")), field_list))),
            )),
            |(n, opts, fields, by_fields)| TopCommand {
                fields,
                n: n.map(|i| i.0 as u64).unwrap_or(opts.limit),
                by: by_fields,
                count_field: opts.count_field,
                // limit: opts.limit,
                other_str: opts.other_str,
                percent_field: opts.percent_field,
                show_count: opts.show_count,
                show_percent: opts.show_percent,
                use_other: opts.use_other,
            },
        )(input)
    }
}
