use crate::functions::shared::{memk, rmcomma, rmunit};
use crate::functions::*;
use crate::pyspark::alias::Aliasable;
use crate::pyspark::ast::column_like;
use crate::pyspark::ast::*;
use crate::pyspark::base::{
    ContextualizedExpr, PysparkTranspileContext, RuntimeSelection, ToSparkExpr,
};
use crate::pyspark::transpiler::utils::convert_time_format;
use crate::spl::ast;
use crate::spl::ast::TimeSpan;
use anyhow::{bail, ensure, Result};
use log::warn;
use std::any::type_name;
use std::ops::Deref;
/*
https://docs.splunk.com/Documentation/SplunkCloud/9.2.2406/SearchReference/CommonEvalFunctions#Function_list_by_category
Type of function
Supported functions and syntax                           Description

Bitwise functions
bit_and(<values>)                                   	 Bitwise AND function that takes two or more non-negative integers as arguments and sequentially performs logical bitwise AND on them.
bit_or(<values>)                                    	 Bitwise OR function that takes two or more non-negative integers as arguments and sequentially performs bitwise OR on them.
bit_not(<value>, <bitmask>)                         	 Bitwise NOT function that takes a non-negative as an argument and inverts every bit in the binary representation of that number. It also takes an optional second argument that acts as a bitmask.
bit_xor(<values>)                                   	 Bitwise XOR function that takes two or more non-negative integers as arguments and sequentially performs bitwise XOR of each of the given arguments.
bit_shift_left(<value>, <shift_offset>)             	 Logical left shift function that takes two non-negative integers as arguments and shifts the binary representation of the first integer over to the left by the specified shift amount.
bit_shift_right(<value>, <shift_offset>)            	 Logical right shift function that takes two non-negative integers as arguments and shifts the binary representation of the first integer over to the right by the specified shift amount.

Comparison and Conditional functions
case(<condition>,<value>,...)                       	 Accepts alternating conditions and values. Returns the first value for which the condition evaluates to TRUE.
cidrmatch(<cidr>,<ip>)                              	 Returns TRUE when an IP address, <ip>, belongs to a particular CIDR subnet, <cidr>.
coalesce(<values>)                                  	 Takes one or more values and returns the first value that is not NULL.
false()                                             	 Returns FALSE.
if(<predicate>,<true_value>,<false_value>)          	 If the <predicate> expression evaluates to TRUE, returns the <true_value>, otherwise the function returns the <false_value>.
in(<field>,<list>)                                  	 Returns TRUE if one of the values in the list matches a value that you specify.
like(<str>,<pattern>)                               	 Returns TRUE only if <str> matches <pattern>.
lookup(<lookup_table>, <json_object>, <json_array>) 	 Performs a CSV lookup. Returns the output field or fields in the form of a JSON object. The lookup() function is available only to Splunk Enterprise users.
match(<str>, <regex>)                               	 Returns TRUE if the regular expression <regex> finds a match against any substring of the string value <str>. Otherwise returns FALSE.
null()                                              	 This function takes no arguments and returns NULL.
nullif(<field1>,<field2>)                           	 Compares the values in two fields and returns NULL if the value in <field1> is equal to the value in <field2>. Otherwise returns the value in <field1>.
searchmatch(<search_str>)                           	 Returns TRUE if the event matches the search string.
true()                                              	 Returns TRUE.
validate(<condition>, <value>,...)                  	 Takes a list of conditions and values and returns the value that corresponds to the condition that evaluates to FALSE. This function defaults to NULL if all conditions evaluate to TRUE. This function is the opposite of the case function.

Conversion functions
ipmask(<mask>,<ip>)                                 	 Generates a new masked IP address by applying a mask to an IP address using a bitwise AND operation.
printf(<format>,<arguments>)                        	 Creates a formatted string based on a format description that you provide.
tonumber(<str>,<base>)                              	 Converts a string to a number.
tostring(<value>,<format>)                          	 Converts the input, such as a number or a Boolean value, to a string.

Cryptographic functions
md5(<str>)                                          	 Computes the md5 hash for the string value.
sha1(<str>)                                         	 Computes the sha1 hash for the string value.
sha256(<str>)                                       	 Computes the sha256 hash for the string value.
sha512(<str>)                                       	 Computes the sha512 hash for the string value.

Date and Time functions
now()                                               	 Returns the time that the search was started.
relative_time(<time>,<specifier>)                   	 Adjusts the time by a relative time specifier.
strftime(<time>,<format>)                           	 Takes a UNIX time and renders it into a human readable format.
strptime(<str>,<format>)                            	 Takes a human readable time and renders it into UNIX time.
time()                                              	 The time that eval function was computed. The time will be different for each event, based on when the event was processed.

Informational functions
isbool(<value>)                                     	 Returns TRUE if the field value is Boolean.
isint(<value>)                                      	 Returns TRUE if the field value is an integer.
isnotnull(<value>)                                  	 Returns TRUE if the field value is not NULL.
isnull(<value>)                                     	 Returns TRUE if the field value is NULL.
isnum(<value>)                                      	 Returns TRUE if the field value is a number.
isstr(<value>)                                      	 Returns TRUE if the field value is a string.
typeof(<value>)                                     	 Returns a string that indicates the field type, such as Number, String, Boolean, and so forth

JSON functions
json_object(<members>)                              	 Creates a new JSON object from members of key-value pairs.
json_append(<json>, <path_value_pairs>)             	 Appends values to the ends of indicated arrays within a JSON document.
json_array(<values>)                                	 Creates a JSON array using a list of values.
json_array_to_mv(<json_array>, <boolean>)           	 Maps the elements of a proper JSON array into a multivalue field.
json_extend(<json>, <path_value_pairs>)             	 Flattens arrays into their component values and appends those values to the ends of indicated arrays within a valid JSON document.
json_extract(<json>, <paths>)                       	 This function returns a value from a piece JSON and zero or more paths. The value is returned in either a JSON array, or a Splunk software native type value.
json_extract_exact(<json>,<keys>)                   	 Returns Splunk software native type values from a piece of JSON by matching literal strings in the event and extracting them as keys.
json_keys(<json>)                                   	 Returns the keys from the key-value pairs in a JSON object as a JSON array.
json_set(<json>, <path_value_pairs>)                	 Inserts or overwrites values for a JSON node with the values provided and returns an updated JSON object.
json_set_exact(<json>,<key_value_pairs>)            	 Uses provided key-value pairs to generate or overwrite a JSON object.
json_valid(<json>)                                  	 Evaluates whether piece of JSON uses valid JSON syntax and returns either TRUE or FALSE.

Mathematical functions
abs(<num>)                                          	 Returns the absolute value.
ceiling(<num>)                                      	 Rounds the value up to the next highest integer.
exact(<expression>)                                 	 Returns the result of a numeric eval calculation with a larger amount of precision in the formatted output.
exp(<num>)                                          	 Returns the exponential function eN.
floor(<num>)                                        	 Rounds the value down to the next lowest integer.
ln(<num>)                                           	 Returns the natural logarithm.
log(<num>,<base>)                                   	 Returns the logarithm of <num> using <base> as the base. If <base> is omitted, base 10 is used.
pi()                                                	 Returns the constant pi to 11 digits of precision.
pow(<num>,<exp>)                                    	 Returns <num> to the power of <exp>, <num><exp>.
round(<num>,<precision>)                            	 Returns <num> rounded to the amount of decimal places specified by <precision>. The default is to round to an integer.
sigfig(<num>)                                       	 Rounds <num> to the appropriate number of significant figures.
sqrt(<num>)                                         	 Returns the square root of the value.
sum(<num>,...)                                      	 Returns the sum of numerical values as an integer.

Multivalue eval functions
commands(<value>)                                   	 Returns a multivalued field that contains a list of the commands used in <value>.
mvappend(<values>)                                  	 Returns a multivalue result based on all of values specified.
mvcount(<mv>)                                       	 Returns the count of the number of values in the specified field.
mvdedup(<mv>)                                       	 Removes all of the duplicate values from a multivalue field.
mvfilter(<predicate>)                               	 Filters a multivalue field based on an arbitrary Boolean expression.
mvfind(<mv>,<regex>)                                	 Finds the index of a value in a multivalue field that matches the regular expression.
mvindex(<mv>,<start>,<end>)                         	 Returns a subset of the multivalue field using the start and end index values.
mvjoin(<mv>,<delim>)                                	 Takes all of the values in a multivalue field and appends the values together using a delimiter.
mvmap(<mv>,<expression>)                            	 This function iterates over the values of a multivalue field, performs an operation using the <expression> on each value, and returns a multivalue field with the list of results.
mvrange(<start>,<end>,<step>)                       	 Creates a multivalue field based on a range of specified numbers.
mvsort(<mv>)                                        	 Returns the values of a multivalue field sorted lexicographically.
mvzip(<mv_left>,<mv_right>,<delim>)                 	 Combines the values in two multivalue fields. The delimiter is used to specify a delimiting character to join the two values.
mv_to_json_array(<field>, <inver_types>)            	 Maps the elements of a multivalue field to a JSON array.
split(<str>,<delim>)                                	 Splits the string values on the delimiter and returns the string values as a multivalue field.

Statistical eval functions
avg(<values>)                                       	 Returns the average of numerical values as an integer.
max(<values>)                                       	 Returns the maximum of a set of string or numeric values.
min(<values>)                                       	 Returns the minimum of a set of string or numeric values.
random()                                            	 Returns a pseudo-random integer ranging from zero to 2^31-1.

Text functions
len(<str>)                                          	 Returns the count of the number of characters, not bytes, in the string.
lower(<str>)                                        	 Converts the string to lowercase.
ltrim(<str>,<trim_chars>)                           	 Removes characters from the left side of a string.
replace(<str>,<regex>,<replacement>)                	 Substitutes the replacement string for every occurrence of the regular expression in the string.
rtrim(<str>,<trim_chars>)                           	 Removes the trim characters from the right side of the string.
spath(<value>,<path>)                               	 Extracts information from the structured data formats XML and JSON.
substr(<str>,<start>,<length>)                      	 Returns a substring of a string, beginning at the start index. The length of the substring specifies the number of character to return.
trim(<str>,<trim_chars>)                            	 Trim characters from both sides of a string.
upper(<str>)                                        	 Returns the string in uppercase.
urldecode(<url>)                                    	 Replaces URL escaped characters with the original characters.

Trigonometry and Hyperbolic functions
acos(X)                                             	 Computes the arc cosine of X.
acosh(X)                                            	 Computes the arc hyperbolic cosine of X.
asin(X)                                             	 Computes the arc sine of X.
asinh(X)                                            	 Computes the arc hyperbolic sine of X.
atan(X)                                             	 Computes the arc tangent of X.
atan2(X,Y)                                          	 Computes the arc tangent of X,Y.
atanh(X)                                            	 Computes the arc hyperbolic tangent of X.
cos(X)                                              	 Computes the cosine of an angle of X radians.
cosh(X)                                             	 Computes the hyperbolic cosine of X radians.
hypot(X,Y)                                          	 Computes the hypotenuse of a triangle.
sin(X)                                              	 Computes the sine of X.
sinh(X)                                             	 Computes the hyperbolic sine of X.
tan(X)                                              	 Computes the tangent of X.
tanh(X)                                             	 Computes the hyperbolic tangent of X.
 */

fn offset_time(time_col: impl Into<Expr>, offset: TimeSpan) -> ColumnLike {
    let TimeSpan { value, scale } = offset;
    let time_col: Expr = time_col.into();
    column_like!([time_col] + [expr("INTERVAL {} {}", value, scale.to_ascii_uppercase())])
}

fn round_time(time_col: impl Into<Expr>, scale: String) -> Result<ColumnLike> {
    let round_to = match scale.as_str() {
        "years" => "year",
        "months" => "month",
        "weeks" => "week",
        "days" => "day",
        "hours" => "hour",
        "minutes" => "minute",
        "seconds" => "second",
        "milliseconds" => "millisecond",
        "microseconds" => "microsecond",
        _ => bail!("Invalid snap time specifier"),
    };
    Ok(column_like!(date_trunc([py_lit(round_to)], [time_col])))
}

pub fn eval_fn(call: ast::Call, ctx: &PysparkTranspileContext) -> Result<ColumnLike> {
    let ast::Call { name, args } = call;
    let args: Vec<_> = args.into_iter().map(|arg| arg.with_context(ctx)).collect();

    match ctx.runtime {
        RuntimeSelection::Disallow => eval_fn_bare(name, args),
        RuntimeSelection::Require => eval_fn_runtime(name, args),
        RuntimeSelection::Allow => {
            eval_fn_runtime(name.clone(), args.clone()).or_else(|_| eval_fn_bare(name, args))
        }
    }
}

fn eval_fn_runtime(name: String, args: Vec<ContextualizedExpr<ast::Expr>>) -> Result<ColumnLike> {
    match name.as_str() {
        "cidrmatch" => function_transform!(cidrmatch [args] (cidr: String, col) {
            ColumnLike::FunctionCall {
                func: "functions.eval.cidrmatch".into(),
                args: vec![PyLiteral::from(cidr).into(), col]
            }
        }),
        name => {
            // warn!(
            //     "Unknown eval function encountered, returning as is: {}",
            //     name
            // );
            let func = format!("functions.eval.{}", name);
            let args: Result<Vec<Expr>> = map_args(args);
            Ok(ColumnLike::FunctionCall { func, args: args? })
        }
    }
}

fn eval_fn_bare(name: String, args: Vec<ContextualizedExpr<ast::Expr>>) -> Result<ColumnLike> {
    match name.as_str() {
        // Convert functions (not sure this is actually valid, but it's in the original transpiler...)
        "memk" => function_transform!(memk [args] (c) { memk(c) }),
        "rmunit" => function_transform!(memk [args] (c) { rmunit(c) }),
        "rmcomma" => function_transform!(memk [args] (c) { rmcomma(c) }),

        // Bitwise functions
        // bit_and(<values>)                                   	 Bitwise AND function that takes two or more non-negative integers as arguments and sequentially performs logical bitwise AND on them.
        "bit_and" => {
            function_transform!(bit_and [args] (a, b) { column_like!([a].bitwiseAND([b])) })
        }
        // bit_or(<values>)                                    	 Bitwise OR function that takes two or more non-negative integers as arguments and sequentially performs bitwise OR on them.
        "bit_or" => function_transform!(bit_or [args] (a, b) { column_like!([a].bitwiseOR([b])) }),
        // bit_not(<value>, <bitmask>)                         	 Bitwise NOT function that takes a non-negative as an argument and inverts every bit in the binary representation of that number. It also takes an optional second argument that acts as a bitmask.
        "bit_not" => function_transform!(bit_not [args] (x) { column_like!(bitwise_not([x])) }),
        // bit_xor(<values>)                                   	 Bitwise XOR function that takes two or more non-negative integers as arguments and sequentially performs bitwise XOR of each of the given arguments.
        "bit_xor" => {
            function_transform!(bit_or [args] (a, b) { column_like!([a].bitwiseXOR([b])) })
        }
        // bit_shift_left(<value>, <shift_offset>)             	 Logical left shift function that takes two non-negative integers as arguments and shifts the binary representation of the first integer over to the left by the specified shift amount.
        "bit_shift_left" => {
            function_transform!(bit_not [args] (x, offset: i64) { column_like!(shiftleft([x], [py_lit(offset)])) })
        }
        // bit_shift_right(<value>, <shift_offset>)            	 Logical right shift function that takes two non-negative integers as arguments and shifts the binary representation of the first integer over to the right by the specified shift amount.
        "bit_shift_right" => {
            function_transform!(bit_not [args] (x, offset: i64) { column_like!(shiftright([x], [py_lit(offset)])) })
        }

        // Comparison and Conditional functions
        // case(<condition>,<value>,...)                       	 Accepts alternating conditions and values. Returns the first value for which the condition evaluates to TRUE.
        "case" => function_transform!(case [args -> mapped_args] () {
            ensure!(mapped_args.len() % 2 == 0, "case() function requires an even number of arguments (conditions and values).");
            let mut c = None;
            for i in (0..mapped_args.len()).step_by(2) {
                let condition = mapped_args[i].clone();
                let value = mapped_args[i + 1].clone();
                c = match c {
                    None => Some(column_like!(when([condition], [value]))),
                    Some(c) => Some(column_like!([c].when([condition], [value])))
                }
            }
            ensure!(c.is_some(), "No condition-value pairs found in case() function.");
            c.unwrap()
        }),
        // cidrmatch(<cidr>,<ip>)                              	 Returns TRUE when an IP address, <ip>, belongs to a particular CIDR subnet, <cidr>.
        "cidrmatch" => function_transform!(cidrmatch [args] (cidr: String, col: String) {
            column_like!(expr("cidr_match('{}', {})", cidr, col))
        }),
        // coalesce(<values>)                                  	 Takes one or more values and returns the first value that is not NULL.
        "coalesce" => function_transform!(coalesce [args -> mapped_args] () {
            column_like!(coalesce(mapped_args))
        }),
        // false()                                             	 Returns FALSE.
        "false" => function_transform!(false [args] () { column_like!(lit(false)) }),
        // if(<predicate>,<true_value>,<false_value>)          	 If the <predicate> expression evaluates to TRUE, returns the <true_value>, otherwise the function returns the <false_value>.
        "if" => function_transform!(if [args](condition, then_expr, else_expr) {
            column_like!([when([condition], [then_expr])].otherwise([else_expr]))
        }),
        // in(<field>,<list>)                                  	 Returns TRUE if one of the values in the list matches a value that you specify.
        "in" => function_transform!(in [args] (x, vals) { column_like!([x].isin([vals])) }),
        // like(<str>,<pattern>)                               	 Returns TRUE only if <str> matches <pattern>.
        "like" => {
            function_transform!(like [args] (x, pattern) { column_like!(like([x], [pattern])) })
        }
        // lookup(<lookup_table>, <json_object>, <json_array>) 	 Performs a CSV lookup. Returns the output field or fields in the form of a JSON object. The lookup() function is available only to Splunk Enterprise users.
        "lookup" => {
            bail!("UNIMPLEMENTED: Unsupported function: {}", name)
        }
        // match(<str>, <regex>)                               	 Returns TRUE if the regular expression <regex> finds a match against any substring of the string value <str>. Otherwise returns FALSE.
        "match" => {
            function_transform!(match [args] (x, regex) { column_like!(regexp_like([x], [regex])) })
        }
        // null()                                              	 This function takes no arguments and returns NULL.
        "null" => function_transform!(null [args] () { column_like!(lit(None)) }),
        // nullif(<field1>,<field2>)                           	 Compares the values in two fields and returns NULL if the value in <field1> is equal to the value in <field2>. Otherwise returns the value in <field1>.
        "nullif" => function_transform!(nullif [args] (x, y) { column_like!(nullif([x], [y])) }),
        // searchmatch(<search_str>)                           	 Returns TRUE if the event matches the search string.
        "searchmatch" => {
            bail!("UNIMPLEMENTED: Unsupported function: {}", name)
        }
        // true()                                              	 Returns TRUE.
        "true" => function_transform!(true [args] () { column_like!(lit(true)) }),
        // validate(<condition>, <value>,...)                  	 Takes a list of conditions and values and returns the value that corresponds to the condition that evaluates to FALSE. This function defaults to NULL if all conditions evaluate to TRUE. This function is the opposite of the case function.
        "validate" => function_transform!(validate [args -> mapped_args] () {
            ensure!(mapped_args.len() % 2 == 0, "case() function requires an even number of arguments (conditions and values).");
            let mut c = None;
            for i in (0..mapped_args.len()).step_by(2) {
                let condition = mapped_args[i].clone();
                let value = mapped_args[i + 1].clone();
                c = match c {
                    None => Some(column_like!(when([~[condition]], [value]))),
                    Some(c) => Some(column_like!([c].when([~[condition]], [value])))
                }
            }
            ensure!(c.is_some(), "No condition-value pairs found in case() function.");
            c.unwrap()
        }),

        // Conversion functions
        // ipmask(<mask>,<ip>)                                 	 Generates a new masked IP address by applying a mask to an IP address using a bitwise AND operation.
        "ipmask" => {
            bail!("UNIMPLEMENTED: Unsupported function: {}", name)
        }
        // printf(<format>,<arguments>)                        	 Creates a formatted string based on a format description that you provide.
        "printf" => bail!("UNIMPLEMENTED: Unsupported function: {}", name),
        // tonumber(<str>,<base>)                              	 Converts a string to a number.
        "tonumber" => function_transform!(tonumber [args -> mapped_args] (x) {
            ensure!(mapped_args.is_empty(), "UMIMPLEMENTED: Unsupported `base` argument provided to `tonumber`");
            column_like!([x].cast([py_lit("double")]))
        }),
        // tostring(<value>,<format>)                          	 Converts the input, such as a number or a Boolean value, to a string.
        "tostring" => function_transform!(tostring [args -> mapped_args] (x) {
            ensure!(mapped_args.is_empty(), "UMIMPLEMENTED: Unsupported `format` argument provided to `tostring`");
            column_like!([x].cast([py_lit("string")]))
        }),

        // Cryptographic functions
        // md5(<str>)                                          	 Computes the md5 hash for the string value.
        "md5" => function_transform!(md5 [args] (x) { column_like!(md5([x])) }),
        // sha1(<str>)                                         	 Computes the sha1 hash for the string value.
        "sha1" => function_transform!(sha1 [args] (x) { column_like!(sha1([x])) }),
        // sha256(<str>)                                       	 Computes the sha256 hash for the string value.
        "sha256" => {
            function_transform!(sha256 [args] (x) { column_like!(sha2([x], [py_lit(256)])) })
        }
        // sha512(<str>)                                       	 Computes the sha512 hash for the string value.
        "sha512" => {
            function_transform!(sha512 [args] (x) { column_like!(sha2([x], [py_lit(512)])) })
        }

        // Date and Time functions
        // now()                                               	 Returns the time that the search was started.
        "now" => function_transform!(now [args] () { column_like!(current_timestamp()) }),
        // relative_time(<time>,<specifier>)                   	 Adjusts the time by a relative time specifier.
        "relative_time" => function_transform!(now [args ...] (time) {
            let relative_time_spec = match args.get(1).map(Deref::deref) {
                Some(ast::Expr::Leaf(ast::LeafExpr::Constant(ast::Constant::SnapTime(snap_time)))) => snap_time,
                Some(v) => bail!("`relative_time` function requires a second argument of type SnapTime, got {:?}", v),
                None => bail!("`relative_time` function requires a second argument"),
            };
            let time = column_like!(to_timestamp([time]));
            let time = match relative_time_spec.span.clone() {
                None => time,
                Some(time_span) => offset_time(time, time_span)
            };
            let time = round_time(time, relative_time_spec.snap.clone())?;

            match relative_time_spec.snap_offset.clone() {
                None => time,
                Some(snap_offset) => offset_time(time, snap_offset)
            }
        }),
        // strftime(<time>,<format>)                           	 Takes a UNIX time and renders it into a human readable format.
        "strftime" => function_transform!(strftime [args] (date, format: String) {
            let converted_time_format = convert_time_format(format)?;
            column_like!(date_format([date], [py_lit(converted_time_format)]))
        }),
        // strptime(<str>,<format>)                            	 Takes a human readable time and renders it into UNIX time.
        "strptime" => function_transform!(strptime [args] (date, format: String) {
            let converted_time_format = convert_time_format(format)?;
            column_like!(to_timestamp([date], [py_lit(converted_time_format)]))
        }),
        // time()                                              	 The time that eval function was computed. The time will be different for each event, based on when the event was processed.
        "time" => function_transform!(time [args] () { column_like!(current_timestamp()) }),

        // Informational functions
        // isbool(<value>)                                     	 Returns TRUE if the field value is Boolean.
        "isbool" => {
            bail!("UNIMPLEMENTED: Unsupported function: {}", name)
        }
        // isint(<value>)                                      	 Returns TRUE if the field value is an integer.
        "isint" => {
            bail!("UNIMPLEMENTED: Unsupported function: {}", name)
        }
        // isnotnull(<value>)                                  	 Returns TRUE if the field value is not NULL.
        "isnotnull" => function_transform!(isnotnull [args] (x) { column_like!(isnotnull([x])) }),
        // isnull(<value>)                                     	 Returns TRUE if the field value is NULL.
        "isnull" => function_transform!(isnull [args] (x) { column_like!(isnull([x])) }),
        // isnum(<value>)                                      	 Returns TRUE if the field value is a number.
        "isnum" => {
            bail!("UNIMPLEMENTED: Unsupported function: {}", name)
        }
        // isstr(<value>)                                      	 Returns TRUE if the field value is a string.
        "isstr" => {
            bail!("UNIMPLEMENTED: Unsupported function: {}", name)
        }
        // typeof(<value>)                                     	 Returns a string that indicates the field type, such as Number, String, Boolean, and so forth
        "typeof" => {
            bail!("UNIMPLEMENTED: Unsupported function: {}", name)
        }

        // JSON functions
        // json_object(<members>)                              	 Creates a new JSON object from members of key-value pairs.
        "json_object" => {
            bail!("UNIMPLEMENTED: Unsupported function: {}", name)
        }
        // json_append(<json>, <path_value_pairs>)             	 Appends values to the ends of indicated arrays within a JSON document.
        "json_append" => {
            bail!("UNIMPLEMENTED: Unsupported function: {}", name)
        }
        // json_array(<values>)                                	 Creates a JSON array using a list of values.
        "json_array" => {
            bail!("UNIMPLEMENTED: Unsupported function: {}", name)
        }
        // json_array_to_mv(<json_array>, <boolean>)           	 Maps the elements of a proper JSON array into a multivalue field.
        "json_array_to_mv" => {
            bail!("UNIMPLEMENTED: Unsupported function: {}", name)
        }
        // json_extend(<json>, <path_value_pairs>)             	 Flattens arrays into their component values and appends those values to the ends of indicated arrays within a valid JSON document.
        "json_extend" => {
            bail!("UNIMPLEMENTED: Unsupported function: {}", name)
        }
        // json_extract(<json>, <paths>)                       	 This function returns a value from a piece JSON and zero or more paths. The value is returned in either a JSON array, or a Splunk software native type value.
        "json_extract" => {
            bail!("UNIMPLEMENTED: Unsupported function: {}", name)
        }
        // json_extract_exact(<json>,<keys>)                   	 Returns Splunk software native type values from a piece of JSON by matching literal strings in the event and extracting them as keys.
        "json_extract_exact" => {
            bail!("UNIMPLEMENTED: Unsupported function: {}", name)
        }
        // json_keys(<json>)                                   	 Returns the keys from the key-value pairs in a JSON object as a JSON array.
        "json_keys" => function_transform!(json_keys [args] (x) {
            column_like!(json_object_keys([x]))
        }),
        // json_set(<json>, <path_value_pairs>)                	 Inserts or overwrites values for a JSON node with the values provided and returns an updated JSON object.
        "json_set" => {
            bail!("UNIMPLEMENTED: Unsupported function: {}", name)
        }
        // json_set_exact(<json>,<key_value_pairs>)            	 Uses provided key-value pairs to generate or overwrite a JSON object.
        "json_set_exact" => {
            bail!("UNIMPLEMENTED: Unsupported function: {}", name)
        }
        // json_valid(<json>)                                  	 Evaluates whether piece of JSON uses valid JSON syntax and returns either TRUE or FALSE.
        "json_valid" => function_transform!(json_valid [args] (x) {
            column_like!([get_json_object([x], [py_lit("$")])].isNotNull())
        }),

        // Mathematical functions
        // abs(<num>)                                          	 Returns the absolute value.
        "abs" => function_transform!(abs [args] (x) { column_like!(abs([x])) }),
        // ceiling(<num>)                                      	 Rounds the value up to the next highest integer.
        "ceiling" => function_transform!(ceiling [args] (x) { column_like!(ceil([x])) }),
        // exact(<expression>)                                 	 Returns the result of a numeric eval calculation with a larger amount of precision in the formatted output.
        "exact" => function_transform!(exact [args] (x) {
            warn!("`exact()` is meaningless in Spark since significant figures are not tracked, use proper precision for your expression inputs instead");
            column_like!([x].cast([py_lit("double")]))
        }),
        // exp(<num>)                                          	 Returns the exponential function eN.
        "exp" => function_transform!(exp [args] (x) { column_like!(exp([x])) }),
        // floor(<num>)                                        	 Rounds the value down to the next lowest integer.
        "floor" => function_transform!(floor [args] (x) { column_like!(floor([x])) }),
        // ln(<num>)                                           	 Returns the natural logarithm.
        "ln" => function_transform!(ln [args] (x) { column_like!(log([x])) }),
        // log(<num>,<base>)                                   	 Returns the logarithm of <num> using <base> as the base. If <base> is omitted, base 10 is used.
        "log" => function_transform!(log [args] (x, base) { column_like!(log([x], [base])) }),
        // pi()                                                	 Returns the constant pi to 11 digits of precision.
        "pi" => function_transform!(pi [args] () { column_like!(lit(3.141592653589793)) }),
        // pow(<num>,<exp>)                                    	 Returns <num> to the power of <exp>, <num><exp>.
        "pow" => function_transform!(pow [args] (x, exp) { column_like!(pow([x], [exp])) }),
        // round(<num>,<precision>)                            	 Returns <num> rounded to the amount of decimal places specified by <precision>. The default is to round to an integer.
        "round" => function_transform!(round [args] (x, precision: i64) {
            column_like!(round([x], [py_lit(precision)]))
        }),
        // sigfig(<num>)                                       	 Rounds <num> to the appropriate number of significant figures.
        "sigfig" => {
            bail!("Unsupported function `{}`: significant figures are not tracked in Spark, use proper precision for your expression inputs instead", name);
        }
        // sqrt(<num>)                                         	 Returns the square root of the value.
        "sqrt" => function_transform!(sqrt [args] (x) { column_like!(sqrt([x])) }),
        // sum(<num>,...)                                      	 Returns the sum of numerical values as an integer.
        "sum" => function_transform!(sum [args] (x) { column_like!(sum([x])) }),

        // Multivalue eval functions
        // commands(<value>)                                   	 Returns a multivalued field that contains a list of the commands used in <value>.
        "commands" => {
            bail!("UNIMPLEMENTED: Unsupported function: {}", name)
        }
        // mvappend(<values>)                                  	 Returns a multivalue result based on all of values specified.
        "mvappend" => function_transform!(mvappend [args -> mapped_args] () {
            column_like!(concat(mapped_args))
        }),
        // mvcount(<mv>)                                       	 Returns the count of the number of values in the specified field.
        "mvcount" => function_transform!(mvcount [args] (column) {
            column_like!(size([column]))
        }),
        // mvdedup(<mv>)                                       	 Removes all of the duplicate values from a multivalue field.
        "mvdedup" => {
            bail!("UNIMPLEMENTED: Unsupported function: {}", name)
        }
        // mvfilter(<predicate>)                               	 Filters a multivalue field based on an arbitrary Boolean expression.
        "mvfilter" => {
            bail!("UNIMPLEMENTED: Unsupported function: {}", name)
        }
        // mvfind(<mv>,<regex>)                                	 Finds the index of a value in a multivalue field that matches the regular expression.
        "mvfind" => {
            bail!("UNIMPLEMENTED: Unsupported function: {}", name)
        }
        // mvindex(<mv>,<start>,<end>)                         	 Returns a subset of the multivalue field using the start and end index values.
        "mvindex" => function_transform!(mvindex [args ...] (x, start: i64) {
            match args.get(2) {
                None => column_like!(get([x], [py_lit(start)])),
                Some(end) => {
                    let end: i64 = map_arg(end) ?;
                    column_like!(slice([x], [py_lit(start + 1)], [py_lit(end - start + 1)]))
                },
            }
        }),
        // mvjoin(<mv>,<delim>)                                	 Takes all of the values in a multivalue field and appends the values together using a delimiter.
        "mvjoin" => {
            bail!("UNIMPLEMENTED: Unsupported function: {}", name)
        }
        // mvmap(<mv>,<expression>)                            	 This function iterates over the values of a multivalue field, performs an operation using the <expression> on each value, and returns a multivalue field with the list of results.
        "mvmap" => {
            bail!("UNIMPLEMENTED: Unsupported function: {}", name)
        }
        // mvrange(<start>,<end>,<step>)                       	 Creates a multivalue field based on a range of specified numbers.
        "mvrange" => {
            bail!("UNIMPLEMENTED: Unsupported function: {}", name)
        }
        // mvsort(<mv>)                                        	 Returns the values of a multivalue field sorted lexicographically.
        "mvsort" => {
            bail!("UNIMPLEMENTED: Unsupported function: {}", name)
        }
        // mvzip(<mv_left>,<mv_right>,<delim>)                 	 Combines the values in two multivalue fields. The delimiter is used to specify a delimiting character to join the two values.
        "mvzip" => function_transform!(mvzip [args -> _mapped_args] (left, right) {
            let delim: String = match args.get(2) {
                None => ",".into(),
                Some(delim) => map_arg(delim) ?,
            };
            let zip_function: Expr = PyLiteral(
                format !(r#"lambda left_, right_: F.concat_ws(r"{}", left_, right_)"#, delim)
            ).into();
            column_like!(
                zip_with(
                    [left],
                    [right],
                    [zip_function]
                )
            )
        }),
        // mv_to_json_array(<field>, <inver_types>)            	 Maps the elements of a multivalue field to a JSON array.
        "mv_to_json_array" => {
            bail!("UNIMPLEMENTED: Unsupported function: {}", name)
        }
        // split(<str>,<delim>)                                	 Splits the string values on the delimiter and returns the string values as a multivalue field.
        "split" => function_transform!(split [args] (x, delim: String) {
            column_like!(split([x], [py_lit(delim)]))
        }),

        // Statistical eval functions
        // avg(<values>)                                       	 Returns the average of numerical values as an integer.
        "avg" => function_transform!(avg [args -> mapped_args] () {
            ensure!(!mapped_args.is_empty(), "Must provide at least one column to average together");
            let mut out: Option<Expr> = None;
            let num_args = mapped_args.len() as i64;
            for c in mapped_args {
                out = match out {
                    Some(v) => Some(column_like!([v] + [c]).into()),
                    None => Some(c),
                }
            };
            assert!(out.is_some(), "Failed to compute average");
            column_like!([out.unwrap()] / [py_lit(num_args)])
        }),
        // max(<values>)                                       	 Returns the maximum of a set of string or numeric values.
        "max" => function_transform!(max [args -> mapped_args] () {
            ensure!(!mapped_args.is_empty(), "Must provide at least one column to max together");
            let mut out: Option<Expr> = None;
            for c in mapped_args {
                out = match out {
                    Some(v) => Some(column_like!(greatest([v], [c])).into()),
                    None => Some(c),
                }
            };
            assert!(out.is_some(), "Failed to compute average");
            out.unwrap()
        }),
        // min(<values>)                                       	 Returns the minimum of a set of string or numeric values.
        "min" => function_transform!(min [args -> mapped_args] () {
            ensure!(!mapped_args.is_empty(), "Must provide at least one column to max together");
            let mut out: Option<Expr> = None;
            for c in mapped_args {
                out = match out {
                    Some(v) => Some(column_like!(least([v], [c])).into()),
                    None => Some(c),
                }
            };
            assert!(out.is_some(), "Failed to compute average");
            out.unwrap()
        }),
        // random()                                            	 Returns a pseudo-random integer ranging from zero to 2^31-1.
        "random" => function_transform!(random [args] () { column_like!(rand()) }),

        // Text functions
        // len(<str>)                                          	 Returns the count of the number of characters, not bytes, in the string.
        "len" => function_transform!(len [args] (x) { column_like!(length([x])) }),
        // lower(<str>)                                        	 Converts the string to lowercase.
        "lower" => function_transform!(lower [args] (x) { column_like!(lower([x])) }),
        // ltrim(<str>,<trim_chars>)                           	 Removes characters from the left side of a string.
        "ltrim" => {
            function_transform!(ltrim [args] (x, chars) { column_like!(ltrim([x], [chars])) })
        }
        // replace(<str>,<regex>,<replacement>)                	 Substitutes the replacement string for every occurrence of the regular expression in the string.
        "replace" => {
            function_transform!(replace [args] (input, regex: String, replacement: String) {
                column_like!(regexp_replace([input.clone()], [py_lit(regex)], [py_lit(replacement)]))
            })
        }
        // rtrim(<str>,<trim_chars>)                           	 Removes the trim characters from the right side of the string.
        "rtrim" => {
            function_transform!(rtrim [args] (x, chars) { column_like!(rtrim([x], [chars])) })
        }
        // spath(<value>,<path>)                               	 Extracts information from the structured data formats XML and JSON.
        "spath" => {
            bail!("UNIMPLEMENTED: Unsupported function: {}", name)
        }
        // substr(<str>,<start>,<length>)                      	 Returns a substring of a string, beginning at the start index. The length of the substring specifies the number of character to return.
        "substr" => function_transform!(substr [args] (column, start: i64, length: i64) {
            column_like!(substring([column], [py_lit(start)], [py_lit(length)]))
        }),
        // trim(<str>,<trim_chars>)                            	 Trim characters from both sides of a string.
        "trim" => function_transform!(trim [args] (x, chars) { column_like!(trim([x], [chars])) }),
        // upper(<str>)                                        	 Returns the string in uppercase.
        "upper" => function_transform!(upper [args] (x) { column_like!(upper([x])) }),
        // urldecode(<url>)                                    	 Replaces URL escaped characters with the original characters.
        "urldecode" => function_transform!(urldecode [args] (x) { column_like!(url_decode([x])) }),

        // Trigonometry and Hyperbolic functions
        // acos(X)                                             	 Computes the arc cosine of X.
        "acos" => function_transform!(acos [args] (x) { column_like!(acos([x])) }),
        // acosh(X)                                            	 Computes the arc hyperbolic cosine of X.
        "acosh" => function_transform!(acosh [args] (x) { column_like!(acosh([x])) }),
        // asin(X)                                             	 Computes the arc sine of X.
        "asin" => function_transform!(asin [args] (x) { column_like!(asin([x])) }),
        // asinh(X)                                            	 Computes the arc hyperbolic sine of X.
        "asinh" => function_transform!(asinh [args] (x) { column_like!(asinh([x])) }),
        // atan(X)                                             	 Computes the arc tangent of X.
        "atan" => function_transform!(atan [args] (x) { column_like!(atan([x])) }),
        // atan2(X,Y)                                          	 Computes the arc tangent of X,Y.
        "atan2" => function_transform!(atan2 [args] (x, y) { column_like!(atan2([x], [y])) }),
        // atanh(X)                                            	 Computes the arc hyperbolic tangent of X.
        "atanh" => function_transform!(atanh [args] (x) { column_like!(atanh([x])) }),
        // cos(X)                                              	 Computes the cosine of an angle of X radians.
        "cos" => function_transform!(cos [args] (x) { column_like!(cos([x])) }),
        // cosh(X)                                             	 Computes the hyperbolic cosine of X radians.
        "cosh" => function_transform!(cosh [args] (x) { column_like!(cosh([x])) }),
        // hypot(X,Y)                                          	 Computes the hypotenuse of a triangle.
        "hypot" => function_transform!(hypot [args] (x, y) { column_like!(hypot([x], [y])) }),
        // sin(X)                                              	 Computes the sine of X.
        "sin" => function_transform!(sin [args] (x) { column_like!(sin([x])) }),
        // sinh(X)                                             	 Computes the hyperbolic sine of X.
        "sinh" => function_transform!(sinh [args] (x) { column_like!(sinh([x])) }),
        // tan(X)                                              	 Computes the tangent of X.
        "tan" => function_transform!(tan [args] (x) { column_like!(tan([x])) }),
        // tanh(X)                                             	 Computes the hyperbolic tangent of X.
        "tanh" => function_transform!(tanh [args] (x) { column_like!(tanh([x])) }),

        // Fallback
        name => {
            warn!(
                "Unknown eval function encountered, returning as is: {}",
                name
            );
            let args: Vec<Expr> = map_args(args)?;
            Ok(ColumnLike::Aliased {
                name: name.into(),
                col: Box::new(
                    ColumnLike::FunctionCall {
                        func: name.to_string(),
                        args,
                    }
                    .into(),
                ),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pyspark::base::PysparkTranspileContext;
    use crate::pyspark::utils::test::assert_python_code_eq;
    use crate::pyspark::ToSparkQuery;
    use rstest::rstest;

    #[rstest]
    fn test_simple_function_max(
        #[from(crate::pyspark::base::test::ctx_bare)] ctx: PysparkTranspileContext,
    ) {
        let result = eval_fn(
            ast::Call {
                name: "max".to_string(),
                args: vec![ast::Field::from("a").into(), ast::Field::from("b").into()],
            },
            &ctx,
        );
        assert_eq!(
            result.unwrap(),
            column_like!([greatest([col("a")], [col("b")])].alias("max"))
        );
    }

    #[rstest]
    fn test_graceful_failure_for_missing_args(
        #[from(crate::pyspark::base::test::ctx_bare)] ctx: PysparkTranspileContext,
    ) {
        let result = eval_fn(
            ast::Call {
                name: "sin".to_string(),
                args: vec![],
            },
            &ctx,
        );
        assert!(result.is_err());
    }

    #[rstest]
    fn test_mvindex_split(
        #[from(crate::pyspark::base::test::ctx_bare)] ctx: PysparkTranspileContext,
    ) {
        let (_, ast) =
            crate::spl::parser::call(r#"mvindex(split(mvindex(BoundaryRanges, -1), ":"), 0)"#)
                .unwrap();
        let result = eval_fn(ast, &ctx);
        assert_eq!(
            result.unwrap().unaliased(),
            column_like!(get(
                [split(
                    [get([col("BoundaryRanges")], [py_lit(-1)])],
                    [py_lit(":")]
                )],
                [py_lit(0)]
            ))
        );
    }

    #[rstest]
    fn test_mvzip_1(#[from(crate::pyspark::base::test::ctx_bare)] ctx: PysparkTranspileContext) {
        let (_, ast) = crate::spl::parser::call(r#"mvzip(x,y)"#).unwrap();
        let result = eval_fn(ast, &ctx);
        assert_python_code_eq(
            result.unwrap().unaliased().to_spark_query(&ctx).unwrap(),
            "F.zip_with(F.col('x'), F.col('y'), lambda left_, right_: F.concat_ws(r',', left_, right_))",
            true,
        );
    }

    #[rstest]
    fn test_mvzip_2(#[from(crate::pyspark::base::test::ctx_bare)] ctx: PysparkTranspileContext) {
        let (_, ast) = crate::spl::parser::call(r#"mvzip(x,y,"_")"#).unwrap();
        let result = eval_fn(ast, &ctx);
        assert_python_code_eq(
            result.unwrap().unaliased().to_spark_query(&ctx).unwrap(),
            "F.zip_with(F.col('x'), F.col('y'), lambda left_, right_: F.concat_ws(r'_', left_, right_))",
            true,
        );
    }

    #[rstest]
    fn test_relative_time_1(
        #[from(crate::pyspark::base::test::ctx_bare)] ctx: PysparkTranspileContext,
    ) {
        let (_, ast) = crate::spl::parser::call(r#"relative_time(now(), "-70m@m")"#).unwrap();
        let result = eval_fn(ast, &ctx);
        assert_python_code_eq(
            result.unwrap().unaliased().to_spark_query(&ctx).unwrap(),
            r#"F.date_trunc(
                "minute",
                (F.to_timestamp(F.current_timestamp()) + F.expr("INTERVAL -70 MINUTES"))
            )"#,
            true,
        );
    }

    #[rstest]
    fn test_cidrmatch(
        #[from(crate::pyspark::base::test::ctx_runtime)] ctx: PysparkTranspileContext,
    ) {
        let (_, ast) = crate::spl::parser::call(r#"cidrmatch("123.132.32.0/25",ip)"#).unwrap();
        let result = eval_fn(ast, &ctx);
        assert_python_code_eq(
            result.unwrap().unaliased().to_spark_query(&ctx).unwrap(),
            r#"functions.eval.cidrmatch("123.132.32.0/25", F.col("ip"))"#,
            true,
        );
    }
}
