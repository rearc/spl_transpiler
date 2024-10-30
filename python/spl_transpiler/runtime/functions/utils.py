# fn offset_time(time_col: impl Into<Expr>, offset: TimeSpan) -> ColumnLike {
#     let TimeSpan { value, scale } = offset;
#     let time_col: Expr = time_col.into();
#     column_like!([time_col] + [expr("INTERVAL {} {}", value, scale.to_ascii_uppercase())])
# }
from spl_transpiler.runtime.base import enforce_types, TimeSpan, TimeScale, Expr
from pyspark.sql import functions as F


@enforce_types
def offset_time(time_col, offset: TimeSpan):
    return time_col + F.expr(
        f"INTERVAL {offset.value} {offset.scale.to_ascii_uppercase()}"
    )


#
# fn round_time(time_col: impl Into<Expr>, scale: String) -> Result<ColumnLike> {
#     let round_to = match scale.as_str() {
#         "years" => "year",
#         "months" => "month",
#         "weeks" => "week",
#         "days" => "day",
#         "hours" => "hour",
#         "minutes" => "minute",
#         "seconds" => "second",
#         "milliseconds" => "millisecond",
#         "microseconds" => "microsecond",
#         _ => bail!("Invalid snap time specifier"),
#     };
#     Ok(column_like!(date_trunc([py_lit(round_to)], [time_col])))
# }


@enforce_types
def round_time(time_col, scale: TimeScale) -> Expr:
    try:
        round_to = {
            scale.MONTHS: "month",
            scale.WEEKS: "week",
            scale.DAYS: "day",
            scale.HOURS: "hour",
            scale.MINUTES: "minute",
            scale.SECONDS: "second",
        }[scale]
    except KeyError as e:
        raise ValueError(f"Invalid snap time specifier: {scale}") from e
    return F.date_trunc(round_to, time_col)


# const CONVERSIONS: [(&str, Option<&str>); 37] = [
#     // Date and time variables
#     ("%c", None), // No direct equivalent
#     ("%+", None), // No direct equivalent
#     // Time variables
#     ("%Ez", None),              // No direct equivalent
#     ("%f", Some("SSS")),        // Microseconds as a decimal number
#     ("%H", Some("HH")),         // Hour (24-hour clock)
#     ("%I", Some("hh")),         // Hour (12-hour clock)
#     ("%k", Some("H")),          // Hour (24-hour clock, leading space)
#     ("%M", Some("mm")),         // Minute
#     ("%N", Some("SSS")),        // Subsecond digits (default to milliseconds)
#     ("%p", Some("a")),          // AM or PM
#     ("%Q", Some("SSS")),        // Subsecond component of a UTC timestamp (default to milliseconds)
#     ("%3Q", Some("SSS")),       // Subsecond component of a UTC timestamp (milliseconds)
#     ("%6Q", Some("SSSSSS")),    // Subsecond component of a UTC timestamp (microseconds)
#     ("%9Q", Some("SSSSSSSSS")), // Subsecond component of a UTC timestamp (nanoseconds)
#     ("%S", Some("ss")),         // Second
#     ("%s", None),               // No direct equivalent for UNIX Epoch Time timestamp
#     ("%T", Some("HH:mm:ss")),   // Time in 24-hour notation
#     ("%X", None),               // No direct equivalent for locale-specific time
#     ("%Z", Some("z")),          // Timezone abbreviation
#     // Use %z to specify hour and minute, for example -0500
#     // Use %:z to specify hour and minute separated by a colon, for example -05:00
#     // Use %::z to specify hour minute and second separated with colons, for example -05:00:00
#     // Use %:::z to specify hour only, for example -05
#     ("%z", Some("x")),        // Timezone offset from UTC
#     ("%:z", Some("xxxx")),    // Timezone offset from UTC
#     ("%::z", Some("xxxxxx")), // Timezone offset from UTC
#     ("%:::z", Some("xx")),    // Timezone offset from UTC
#     // Date variables
#     ("%F", Some("yyyy-MM-dd")), // ISO 8601 date format
#     ("%x", None),               // No direct equivalent for locale-specific date
#     // Specifying days and weeks
#     ("%A", Some("EEEE")), // Full weekday name
#     ("%a", Some("EEE")),  // Abbreviated weekday name
#     ("%d", Some("dd")),   // Day of the month
#     ("%e", Some("d")),    // Day of the month (leading space)
#     ("%j", Some("D")),    // Day of year
#     // ("%V", Some("w")), // Week of the year (ISO)
#     // ("%U", Some("w")), // Week of the year (starting with 0)
#     ("%w", Some("e")), // Weekday as a decimal number (1 = Monday, ..., 7 = Sunday)
#     // Specifying months
#     ("%b", Some("MMM")),  // Abbreviated month name
#     ("%B", Some("MMMM")), // Full month name
#     ("%m", Some("MM")),   // Month as a decimal number
#     // Specifying year
#     ("%y", Some("yy")),   // Year without century
#     ("%Y", Some("yyyy")), // Year with century
#     // Other
#     ("%%", Some("%")), // A literal "%" character
# ];

CONVERSIONS = {
    "%c": None,
    "%+": None,
    "%Ez": None,
    "%f": "SSS",
    "%H": "HH",
    "%I": "hh",
    "%k": "H",
    "%M": "mm",
    "%N": "SSS",
    "%p": "a",
    "%Q": "SSS",
    "%3Q": "SSS",
    "%6Q": "SSSSSS",
    "%9Q": "SSSSSSSSS",
    "%S": "ss",
    "%s": None,
    "%T": "HH:mm:ss",
    "%X": None,
    "%Z": "z",
    "%z": "x",
    "%:z": "xxxx",
    "%::z": "xxxxxx",
    "%:::z": "xx",
    "%F": "yyyy-MM-dd",
    "%x": None,
    "%A": "EEEE",
    "%a": "EEE",
    "%d": "dd",
    "%e": "d",
    "%j": "D",
    # "%V": "w",
    # "%U": "w",
    "%w": "e",
    "%b": "MMM",
    "%B": "MMMM",
    "%m": "MM",
    "%y": "yy",
    "%Y": "yyyy",
    "%%": "%",
}

#
# pub fn convert_time_format(spl_time_format: impl ToString) -> Result<String> {
#     let mut fmt_string = spl_time_format.to_string();
#
#     for (original, replacement) in CONVERSIONS {
#         if !fmt_string.contains(original) {
#             continue;
#         }
#         match replacement {
#             None => bail!("No known replacement pattern for `{}`", original),
#             Some(replacement) => fmt_string = fmt_string.replace(original, replacement),
#         }
#     }
#     Ok(fmt_string)
# }


def convert_time_format(fmt_string: str) -> str:
    for original, replacement in CONVERSIONS.items():
        if original not in fmt_string:
            continue
        if replacement is None:
            raise ValueError(f"No known replacement pattern for `{original}`")
        fmt_string = fmt_string.replace(original, replacement)
    return fmt_string
