use crate::commands::cmd_convert::spl::{ConvertCommand, FieldConversion};
use crate::functions::shared::{ctime, memk, rmcomma, rmunit};
use crate::pyspark::ast::*;
use crate::spl::ast;
use log::warn;

/*
https://docs.splunk.com/Documentation/SplunkCloud/9.2.2406/SearchReference/Convert

auto()
Syntax: auto(<wc-field>)
Description: Automatically convert the fields to a number using the best conversion. Note that if not all values of a particular field can be converted using a known conversion type, the field is left untouched and no conversion at all is done for that field. You can use a wildcard ( * ) character to specify all fields.
ctime()
Syntax: ctime(<wc-field>)
Description: Convert a UNIX time to an ASCII human readable time. Use the timeformat option to specify the exact format to convert to. You can use a wildcard ( * ) character to specify all fields.
dur2sec()
Syntax: dur2sec(<wc-field>)
Description: Convert a duration format "[D+]HH:MM:SS" to seconds. You can use a wildcard ( * ) character to specify all fields.
memk()
Syntax: memk(<wc-field>)
Description: Accepts a positive number (integer or float) followed by an optional "k", "m", or "g". The letter k indicates kilobytes, m indicates megabytes, and g indicates gigabytes. If no letter is specified, kilobytes is assumed. The output field is a number expressing quantity of kilobytes. Negative values cause data incoherency. You can use a wildcard ( * ) character to specify all fields.
mktime()
Syntax: mktime(<wc-field>)
Description: Convert a human readable time string to an epoch time. Use timeformat option to specify exact format to convert from. You can use a wildcard ( * ) character to specify all fields.
mstime()
Syntax: mstime(<wc-field>)
Description: Convert a [MM:]SS.SSS format to seconds. You can use a wildcard ( * ) character to specify all fields.
none()
Syntax: none(<wc-field>)
Description: In the presence of other wildcards, indicates that the matching fields should not be converted. You can use a wildcard ( * ) character to specify all fields.
num()
Syntax: num(<wc-field>)
Description: Like auto(), except non-convertible values are removed. You can use a wildcard ( * ) character to specify all fields.
rmcomma()
Syntax: rmcomma(<wc-field>)
Description: Removes all commas from value, for example rmcomma(1,000,000.00) returns 1000000.00. You can use a wildcard ( * ) character to specify all fields.
rmunit()
Syntax: rmunit(<wc-field>)
Description: Looks for numbers at the beginning of the value and removes trailing text. You can use a wildcard ( * ) character to specify all fields.
 */

pub fn convert_fn(
    cmd: &ConvertCommand,
    conversion: &FieldConversion,
) -> anyhow::Result<ColumnLike> {
    let ConvertCommand { timeformat, .. } = cmd;
    let timeformat = crate::pyspark::transpiler::utils::convert_time_format(timeformat);
    let FieldConversion {
        func,
        field: ast::Field(field_name),
        ..
    } = conversion;

    match func.as_str() {
        "ctime" => Ok(ctime(column_like!(col(field_name)), timeformat)),
        "memk" => Ok(memk(column_like!(col(field_name)))),
        "rmunit" => Ok(rmunit(column_like!(col(field_name)))),
        "rmcomma" => Ok(rmcomma(column_like!(col(field_name)))),

        name => {
            warn!(
                "Unknown convert function encountered, returning as is: {}",
                name
            );
            Ok(ColumnLike::Aliased {
                name: name.into(),
                col: Box::new(
                    ColumnLike::FunctionCall {
                        func: name.to_string(),
                        args: vec![column_like!(col(field_name)).into()],
                    }
                    .into(),
                ),
            })
        }
    }
}
