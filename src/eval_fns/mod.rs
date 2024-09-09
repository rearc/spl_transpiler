use crate::ast::ast;
use crate::pyspark::ast::*;
use crate::pyspark::dealias::Dealias;
use crate::pyspark::transpiler::utils::convert_time_format;
use anyhow::{bail, Result};

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

trait EvalFunction {
    const SPL_NAME: &'static str;

    fn to_column_like(&self) -> Result<ColumnLike>;
}

macro_rules! _eval_fn_args {
    ([$args:ident, $i:expr] ()) => {};
    // [$($body:tt)*] => {_eval_fn_arg!($($body)*)};
    // ($name:ident : Expr , $($tail:tt)*) => {
    //     $name: Expr = map_arg(&args[_i])?;
    //     _i += 1;
    //     _eval_fn_args!($($tail)*)
    // };
    // (, $($tail:tt)*) => {_eval_fn_args!($($tail)*)};
    ([$args:ident, $i:ident] ($name:ident : $type:ty , $($tail:tt)*)) => {
        let $name: $type = map_arg(&$args[$i])?;
        $i += 1;
        _eval_fn_args!([$args,$i] ($($tail)*));
    };
    ([$args:ident, $i:ident] ($name:ident : $type:ty)) => {
        let $name: $type = map_arg(&$args[$i])?;
        $i += 1;
    };
    ([$args:ident, $i:ident] ($name:ident , $($tail:tt)*)) => {
        let $name: Expr = map_arg(&$args[$i])?;
        $i += 1;
        _eval_fn_args!([$args,$i] ($($tail)*));
    };
    ([$args:ident, $i:ident] ($name:ident)) => {
        let $name: Expr = map_arg(&$args[$i])?;
        $i += 1;
    };
    // ([$args:expr, $i:ident] ($($tail:tt)*)) => {
    //     _eval_fn_args!([$args,$i] $($tail)*);
    // };
}

macro_rules! eval_fn {
    ($name:ident [$arg_vec:ident] $args:tt { $out:expr }) => {
        {
            use anyhow::ensure;
            use crate::pyspark::ast::column_like;

            let mut _i: usize = 0;
            _eval_fn_args!([$arg_vec, _i] $args);
            ensure!(_i == $arg_vec.len(), "Mistmatched number of arguments (code: {}, runtime: {}); fix arg list or assign remaining arguments using `eval_fn!({} [{} -> mapped_args] ...`", _i, $arg_vec.len(), stringify!($name), stringify!($arg_vec));
            Ok(column_like!([$out].alias(stringify!($name))).into())
        }
    };
    ($name:ident [$arg_vec:ident -> $mapped_arg_name:ident] $args:tt { $out:expr }) => {
        {
            use crate::pyspark::ast::column_like;

            let mut _i: usize = 0;
            _eval_fn_args!([$arg_vec, _i] $args);
            let $mapped_arg_name: Vec<Expr> = map_args($arg_vec.iter().skip(_i).cloned().collect())?;
            Ok(column_like!([$out].alias(stringify!($name))).into())
        }
    };
}

// macro_rules! _args {
//     ($args:tt) => {_eval_fn_args!($args)};
// }

// _eval_fn_args!(a: Expr, b: String, c);
// fn f() {
//     let args: Vec<Expr> = vec![];
//     // let _i = 0;
//     // _eval_fn_args!([args, _i] (x, y, z));
//     // _eval_fn_args!( [ args , _i ] y, z );
//     eval_fn!([args] (condition, then_expr, else_expr) {
//         column_like!([when([condition], [then_expr])].otherwise([else_expr]))
//     });
// }
// const _: &str = _eval_fn_args!(b: String);

// static SIMPLE_FUNC_MAP: phf::Map<&'static str, Option<&'static str>> = phf_map! {
//
// };

impl TryFrom<ast::Expr> for i64 {
    type Error = anyhow::Error;

    fn try_from(value: ast::Expr) -> std::result::Result<i64, Self::Error> {
        match value {
            ast::Expr::Leaf(ast::LeafExpr::Constant(ast::Constant::Int(ast::IntValue(val)))) => {
                Ok(val)
            }
            _ => bail!("No default conversion from {:?} to i64", value),
        }
    }
}

impl TryFrom<ast::Expr> for f64 {
    type Error = anyhow::Error;
    fn try_from(value: ast::Expr) -> std::result::Result<f64, Self::Error> {
        match value {
            ast::Expr::Leaf(ast::LeafExpr::Constant(ast::Constant::Double(ast::DoubleValue(
                val,
            )))) => Ok(val),
            _ => bail!("No default conversion from {:?} to f64", value),
        }
    }
}

impl TryFrom<ast::Expr> for String {
    type Error = anyhow::Error;

    fn try_from(value: ast::Expr) -> std::result::Result<String, Self::Error> {
        match value {
            ast::Expr::Leaf(ast::LeafExpr::Constant(const_)) => match const_ {
                ast::Constant::Null(_) => {
                    bail!("No default conversion from {:?} to String", const_)
                }
                ast::Constant::Bool(_) => {
                    bail!("No default conversion from {:?} to String", const_)
                }
                ast::Constant::Int(_) => bail!("No default conversion from {:?} to String", const_),
                ast::Constant::Double(_) => {
                    bail!("No default conversion from {:?} to String", const_)
                }
                ast::Constant::Str(ast::StrValue(val)) => Ok(val.clone()),
                ast::Constant::SnapTime(_) => {
                    bail!("No default conversion from {:?} to String", const_)
                }
                ast::Constant::SplSpan(_) => {
                    bail!("No default conversion from {:?} to String", const_)
                }
                ast::Constant::Field(ast::Field(val)) => Ok(val.clone()),
                ast::Constant::Wildcard(ast::Wildcard(val)) => Ok(val.clone()),
                ast::Constant::Variable(ast::Variable(val)) => Ok(val.clone()),
                ast::Constant::IPv4CIDR(ast::IPv4CIDR(val)) => Ok(val.clone()),
            },
            _ => bail!("Unsupported mvindex start argument: {:?}", value),
        }
    }
}

fn map_arg<E, T>(arg: &ast::Expr) -> Result<T>
where
    E: Into<anyhow::Error>,
    T: TryFrom<ast::Expr, Error = E> + Dealias,
{
    arg.clone()
        .try_into()
        .map(|e: T| e.unaliased())
        .map_err(|e: E| e.into())
}

fn map_args<E, T>(args: Vec<ast::Expr>) -> Result<Vec<T>>
where
    E: Into<anyhow::Error>,
    T: TryFrom<ast::Expr, Error = E> + Dealias,
{
    args.iter().map(|arg| map_arg(arg)).collect()
}

pub fn eval_fn(call: ast::Call) -> Result<ColumnLike> {
    let ast::Call { name, args } = call;

    // match (name.as_str(), args.len()) {
    match name.as_str() {
        // if(condition, then_expr, else_expr) -> when(condition, then_expr).otherwise(else_expr)
        // "if" => {
        //     let condition: Expr = map_arg(&args[0])?;
        //     let then_expr: Expr = map_arg(&args[1])?;
        //     let else_expr: Expr = map_arg(&args[2])?;
        //
        //     Ok(column_like!([when([condition], [then_expr])].otherwise([else_expr])).into())
        // }
        // eval_fn!(if[3](condition, then_expr, else_expr) => column_like!([when([condition], [then_expr])].otherwise([else_expr])))

        // coalesce(a, b, ...) -> coalesce(col(a), col(b), ...)
        // ("coalesce", _) => {
        //     let cols: Vec<Expr> = map_args(args)?;
        //     let coalesce = ColumnLike::FunctionCall {
        //         func: "coalesce".to_string(),
        //         args: cols,
        //     };
        //     Ok(column_like!([coalesce].alias("coalesce")).into())
        // }

        // mvcount(d) -> size(col(d))
        // "mvcount" => {
        //     let column: Expr = map_arg(&args[0])?;
        //     Ok(column_like!([size([column])].alias("mvcount")).into())
        // }

        // mvindex(a, b, c) -> slice(a, b+1, c+1)
        // "mvindex" => {
        //     let x: Expr = map_arg(&args[0])?;
        //     let start: i64 = map_arg(&args[1])?;
        //     let end: i64 = map_arg(&args[2])?;
        //     let length = end - start + 1;
        //     Ok(
        //         column_like!([slice([x], [py_lit(start + 1)], [py_lit(length)])].alias("mvindex"))
        //             .into(),
        //     )
        // }

        // mvappend(a, b) -> concat(a, b)
        // "mvappend" => {
        //     let left: Expr = map_arg(&args[0])?;
        //     let right: Expr = map_arg(&args[1])?;
        //     Ok(column_like!([concat([left], [right])].alias("mvappend")).into())
        // }

        // mvfilter(condition_about_d) -> filter(d, lambda d_: condition)
        // TODO
        // ast::Expr::Call(ast::Call { name, args }) if name == "mvfilter" && args.len() == 2 => {
        //     let condition: Expr = map_arg(&args[0])?;
        //     let lambda: Expr = map_arg(&args[1])?;
        //     Ok(column_like!([filter([condition], [lambda])].alias("mvfilter")).into())
        // },

        // strftime(d, c_fmt) -> date_format(d, spark_fmt)
        // "strftime" => {
        //     let date: Expr = map_arg(&args[0])?;
        //     let format: String = map_arg(&args[1])?;
        //     let format = convert_time_format(format);
        //     Ok(column_like!([date_format([date], [py_lit(format)])].alias("strftime")).into())
        // }

        // min(d, val) -> least(d, val)
        // "min" => {
        //     let column: Expr = map_arg(&args[0])?;
        //     let value: Expr = map_arg(&args[1])?;
        //     Ok(column_like!([least([column], [value])].alias("min")).into())
        // }

        // max(d, val) -> greatest(d, val)
        // "max" => {
        //     let column: Expr = map_arg(&args[0])?;
        //     let value: Expr = map_arg(&args[1])?;
        //     Ok(column_like!([greatest([column], [value])].alias("max")).into())
        // }

        // round(f, n) -> round(f, n)
        // "round" => {
        //     let column: Expr = map_arg(&args[0])?;
        //     let precision: i64 = map_arg(&args[1])?;
        //     Ok(column_like!([round([column], [py_lit(precision)])].alias("round")).into())
        // }

        // substr(s, start, len) -> substring(s, start, len)
        // "substr" => {
        //     let column: Expr = map_arg(&args[0])?;
        //     let start: i64 = map_arg(&args[1])?;
        //     let length: i64 = map_arg(&args[2])?;
        //     Ok(column_like!(
        //         [substring([column], [py_lit(start)], [py_lit(length)])].alias("substr")
        //     )
        //     .into())
        // }

        // cidrmatch(cidr, c) => expr("cidr_match({}, {})")
        // "cidrmatch" => {
        //     let cidr: String = map_arg(&args[0])?;
        //     let col: String = map_arg(&args[1])?;
        //     Ok(column_like!(
        //         [expr([py_lit(format!("cidr_match('{}', {})", cidr, col))])].alias("cidrmatch")
        //     )
        //     .into())
        // }

        // memk(v) => ...
        // Converts a string like "10k", "3g" or "7" (implicit "k") to kilobytes
        "memk" => {
            let col: Expr = map_arg(&args[0])?;
            const REGEX: &str = r#"(?i)^(\d*\.?\d+)([kmg])$"#;
            let num_val = column_like!(regexp_extract([col.clone()], [py_lit(REGEX)], [py_lit(1)]));
            let num_val = column_like!([num_val].cast([py_lit("double")]));
            let unit = column_like!(upper([regexp_extract(
                [col.clone()],
                [py_lit(REGEX)],
                [py_lit(2)]
            )]));
            let is_k = column_like!([unit.clone()] == [lit("K")]);
            let is_m = column_like!([unit.clone()] == [lit("M")]);
            let is_g = column_like!([unit.clone()] == [lit("G")]);
            let scaling_factor = column_like!(when([is_k], [lit(1.0)]));
            let scaling_factor = column_like!([scaling_factor].when([is_m], [lit(1024.0)]));
            let scaling_factor = column_like!([scaling_factor].when([is_g], [lit(1048576.0)]));
            let scaling_factor = column_like!([scaling_factor].otherwise([lit(1.0)]));
            Ok(column_like!([[num_val] * [scaling_factor]].alias("memk")).into())
        }

        // rmunit(s) => ...
        // Given a string like "10k", "3g", or "7", return just the numeric component
        "rmunit" => {
            let col: Expr = map_arg(&args[0])?;
            const REGEX: &str = r#"(?i)^(\d*\.?\d+)(\w*)$"#;
            let num_val = column_like!(regexp_extract([col.clone()], [py_lit(REGEX)], [py_lit(1)]));
            let num_val = column_like!([num_val].cast([py_lit("double")]));
            Ok(column_like!([num_val].alias("rmunit")).into())
        }

        // rmcomma(s) => ...
        // Given a string, remove all commas from it and cast it to a double
        "rmcomma" => {
            let col: Expr = map_arg(&args[0])?;
            Ok(column_like!(
                [[regexp_replace([col.clone()], [py_lit(",")], [py_lit("")])]
                    .cast([py_lit("double")])]
                .alias("rmcomma")
            )
            .into())
        }

        // "replace" => {
        //     let input: Expr = map_arg(&args[0])?;
        //     let regex: String = map_arg(&args[1])?;
        //     let replacement: String = map_arg(&args[2])?;
        //     Ok(column_like!(regexp_replace(
        //         [input.clone()],
        //         [py_lit(regex)],
        //         [py_lit(replacement)]
        //     )))
        // }

        // Bitwise functions
        // bit_and(<values>)                                   	 Bitwise AND function that takes two or more non-negative integers as arguments and sequentially performs logical bitwise AND on them.
        "bit_and" => eval_fn!(bit_and [args] (a, b) { column_like!([a].bitwiseAND([b])) }),
        // bit_or(<values>)                                    	 Bitwise OR function that takes two or more non-negative integers as arguments and sequentially performs bitwise OR on them.
        "bit_or" => eval_fn!(bit_or [args] (a, b) { column_like!([a].bitwiseOR([b])) }),
        // bit_not(<value>, <bitmask>)                         	 Bitwise NOT function that takes a non-negative as an argument and inverts every bit in the binary representation of that number. It also takes an optional second argument that acts as a bitmask.
        "bit_not" => eval_fn!(bit_not [args] (x) { column_like!(bitwise_not([x])) }),
        // bit_xor(<values>)                                   	 Bitwise XOR function that takes two or more non-negative integers as arguments and sequentially performs bitwise XOR of each of the given arguments.
        "bit_xor" => eval_fn!(bit_or [args] (a, b) { column_like!([a].bitwiseXOR([b])) }),
        // bit_shift_left(<value>, <shift_offset>)             	 Logical left shift function that takes two non-negative integers as arguments and shifts the binary representation of the first integer over to the left by the specified shift amount.
        "bit_shift_left" => {
            eval_fn!(bit_not [args] (x, offset: i64) { column_like!(shiftleft([x], [py_lit(offset)])) })
        }
        // bit_shift_right(<value>, <shift_offset>)            	 Logical right shift function that takes two non-negative integers as arguments and shifts the binary representation of the first integer over to the right by the specified shift amount.
        "bit_shift_right" => {
            eval_fn!(bit_not [args] (x, offset: i64) { column_like!(shiftright([x], [py_lit(offset)])) })
        }

        // Comparison and Conditional functions
        // case(<condition>,<value>,...)                       	 Accepts alternating conditions and values. Returns the first value for which the condition evaluates to TRUE.
        "case" => {
            unimplemented!("Unsupported function: {}", name)
        }
        // cidrmatch(<cidr>,<ip>)                              	 Returns TRUE when an IP address, <ip>, belongs to a particular CIDR subnet, <cidr>.
        "cidrmatch" => eval_fn!(cidrmatch [args] (cidr: String, col: String) {
            column_like!(expr("cidr_match('{}', {})", cidr, col))
        }),
        // coalesce(<values>)                                  	 Takes one or more values and returns the first value that is not NULL.
        "coalesce" => eval_fn!(coalesce [args -> mapped_args] () {
            column_like!(coalesce(mapped_args))
        }),
        // false()                                             	 Returns FALSE.
        "false" => eval_fn!(false [args] (x) { column_like!(lit(false)) }),
        // if(<predicate>,<true_value>,<false_value>)          	 If the <predicate> expression evaluates to TRUE, returns the <true_value>, otherwise the function returns the <false_value>.
        "if" => eval_fn!(if [args](condition, then_expr, else_expr) {
            column_like!([when([condition], [then_expr])].otherwise([else_expr]))
        }),
        // in(<field>,<list>)                                  	 Returns TRUE if one of the values in the list matches a value that you specify.
        "in" => eval_fn!(in [args] (x, vals) { column_like!([x].isin([vals])) }),
        // like(<str>,<pattern>)                               	 Returns TRUE only if <str> matches <pattern>.
        "like" => eval_fn!(like [args] (x, pattern) { column_like!(like([x], [pattern])) }),
        // lookup(<lookup_table>, <json_object>, <json_array>) 	 Performs a CSV lookup. Returns the output field or fields in the form of a JSON object. The lookup() function is available only to Splunk Enterprise users.
        "lookup" => {
            unimplemented!("Unsupported function: {}", name)
        }
        // match(<str>, <regex>)                               	 Returns TRUE if the regular expression <regex> finds a match against any substring of the string value <str>. Otherwise returns FALSE.
        "match" => eval_fn!(match [args] (x, regex) { column_like!(regexp_like([x], [regex])) }),
        // null()                                              	 This function takes no arguments and returns NULL.
        "null" => eval_fn!(null [args] (x) { column_like!(lit(None)) }),
        // nullif(<field1>,<field2>)                           	 Compares the values in two fields and returns NULL if the value in <field1> is equal to the value in <field2>. Otherwise returns the value in <field1>.
        "nullif" => eval_fn!(nullif [args] (x, y) { column_like!(nullif([x], [y])) }),
        // searchmatch(<search_str>)                           	 Returns TRUE if the event matches the search string.
        "searchmatch" => {
            unimplemented!("Unsupported function: {}", name)
        }
        // true()                                              	 Returns TRUE.
        "true" => eval_fn!(true [args] (x) { column_like!(lit(true)) }),
        // validate(<condition>, <value>,...)                  	 Takes a list of conditions and values and returns the value that corresponds to the condition that evaluates to FALSE. This function defaults to NULL if all conditions evaluate to TRUE. This function is the opposite of the case function.
        "validate" => {
            unimplemented!("Unsupported function: {}", name)
        }

        // Conversion functions
        // ipmask(<mask>,<ip>)                                 	 Generates a new masked IP address by applying a mask to an IP address using a bitwise AND operation.
        "ipmask" => {
            unimplemented!("Unsupported function: {}", name)
        }
        // printf(<format>,<arguments>)                        	 Creates a formatted string based on a format description that you provide.
        "printf" => eval_fn!(printf [args] (x) { column_like!(format_string([x])) }),
        // tonumber(<str>,<base>)                              	 Converts a string to a number.
        "tonumber" => eval_fn!(tonumber [args] (x) { column_like!(cast([x])) }),
        // tostring(<value>,<format>)                          	 Converts the input, such as a number or a Boolean value, to a string.
        "tostring" => eval_fn!(tostring [args] (x) { column_like!(cast([x])) }),

        // Cryptographic functions
        // md5(<str>)                                          	 Computes the md5 hash for the string value.
        "md5" => eval_fn!(md5 [args] (x) { column_like!(md5([x])) }),
        // sha1(<str>)                                         	 Computes the sha1 hash for the string value.
        "sha1" => eval_fn!(sha1 [args] (x) { column_like!(sha1([x])) }),
        // sha256(<str>)                                       	 Computes the sha256 hash for the string value.
        "sha256" => eval_fn!(sha256 [args] (x) { column_like!(sha2([x], [py_lit(256)])) }),
        // sha512(<str>)                                       	 Computes the sha512 hash for the string value.
        "sha512" => eval_fn!(sha512 [args] (x) { column_like!(sha2([x], [py_lit(512)])) }),

        // Date and Time functions
        // now()                                               	 Returns the time that the search was started.
        "now" => eval_fn!(now [args] () { column_like!(current_timestamp()) }),
        // relative_time(<time>,<specifier>)                   	 Adjusts the time by a relative time specifier.
        "relative_time" => {
            unimplemented!("Unsupported function: {}", name)
        }
        // strftime(<time>,<format>)                           	 Takes a UNIX time and renders it into a human readable format.
        "strftime" => eval_fn!(strftime [args] (date, format: String) {
            column_like!(date_format([date], [py_lit(convert_time_format(format))]))
        }),
        // strptime(<str>,<format>)                            	 Takes a human readable time and renders it into UNIX time.
        "strptime" => {
            unimplemented!("Unsupported function: {}", name)
        }
        // time()                                              	 The time that eval function was computed. The time will be different for each event, based on when the event was processed.
        "time" => eval_fn!(time [args] () { column_like!(current_timestamp()) }),

        // Informational functions
        // isbool(<value>)                                     	 Returns TRUE if the field value is Boolean.
        "isbool" => {
            unimplemented!("Unsupported function: {}", name)
        }
        // isint(<value>)                                      	 Returns TRUE if the field value is an integer.
        "isint" => {
            unimplemented!("Unsupported function: {}", name)
        }
        // isnotnull(<value>)                                  	 Returns TRUE if the field value is not NULL.
        "isnotnull" => eval_fn!(isnotnull [args] (x) { column_like!(isnotnull([x])) }),
        // isnull(<value>)                                     	 Returns TRUE if the field value is NULL.
        "isnull" => eval_fn!(isnull [args] (x) { column_like!(isnull([x])) }),
        // isnum(<value>)                                      	 Returns TRUE if the field value is a number.
        "isnum" => {
            unimplemented!("Unsupported function: {}", name)
        }
        // isstr(<value>)                                      	 Returns TRUE if the field value is a string.
        "isstr" => {
            unimplemented!("Unsupported function: {}", name)
        }
        // typeof(<value>)                                     	 Returns a string that indicates the field type, such as Number, String, Boolean, and so forth
        "typeof" => {
            unimplemented!("Unsupported function: {}", name)
        }

        // JSON functions
        // json_object(<members>)                              	 Creates a new JSON object from members of key-value pairs.
        "json_object" => {
            unimplemented!("Unsupported function: {}", name)
        }
        // json_append(<json>, <path_value_pairs>)             	 Appends values to the ends of indicated arrays within a JSON document.
        "json_append" => {
            unimplemented!("Unsupported function: {}", name)
        }
        // json_array(<values>)                                	 Creates a JSON array using a list of values.
        "json_array" => {
            unimplemented!("Unsupported function: {}", name)
        }
        // json_array_to_mv(<json_array>, <boolean>)           	 Maps the elements of a proper JSON array into a multivalue field.
        "json_array_to_mv" => {
            unimplemented!("Unsupported function: {}", name)
        }
        // json_extend(<json>, <path_value_pairs>)             	 Flattens arrays into their component values and appends those values to the ends of indicated arrays within a valid JSON document.
        "json_extend" => {
            unimplemented!("Unsupported function: {}", name)
        }
        // json_extract(<json>, <paths>)                       	 This function returns a value from a piece JSON and zero or more paths. The value is returned in either a JSON array, or a Splunk software native type value.
        "json_extract" => {
            unimplemented!("Unsupported function: {}", name)
        }
        // json_extract_exact(<json>,<keys>)                   	 Returns Splunk software native type values from a piece of JSON by matching literal strings in the event and extracting them as keys.
        "json_extract_exact" => {
            unimplemented!("Unsupported function: {}", name)
        }
        // json_keys(<json>)                                   	 Returns the keys from the key-value pairs in a JSON object as a JSON array.
        "json_keys" => {
            unimplemented!("Unsupported function: {}", name)
        }
        // json_set(<json>, <path_value_pairs>)                	 Inserts or overwrites values for a JSON node with the values provided and returns an updated JSON object.
        "json_set" => {
            unimplemented!("Unsupported function: {}", name)
        }
        // json_set_exact(<json>,<key_value_pairs>)            	 Uses provided key-value pairs to generate or overwrite a JSON object.
        "json_set_exact" => {
            unimplemented!("Unsupported function: {}", name)
        }
        // json_valid(<json>)                                  	 Evaluates whether piece of JSON uses valid JSON syntax and returns either TRUE or FALSE.
        "json_valid" => {
            unimplemented!("Unsupported function: {}", name)
        }

        // Mathematical functions
        // abs(<num>)                                          	 Returns the absolute value.
        "abs" => eval_fn!(abs [args] (x) { column_like!(abs([x])) }),
        // ceiling(<num>)                                      	 Rounds the value up to the next highest integer.
        "ceiling" => eval_fn!(ceiling [args] (x) { column_like!(ceil([x])) }),
        // exact(<expression>)                                 	 Returns the result of a numeric eval calculation with a larger amount of precision in the formatted output.
        "exact" => {
            unimplemented!("Unsupported function: {}", name)
        }
        // exp(<num>)                                          	 Returns the exponential function eN.
        "exp" => eval_fn!(exp [args] (x) { column_like!(exp([x])) }),
        // floor(<num>)                                        	 Rounds the value down to the next lowest integer.
        "floor" => eval_fn!(floor [args] (x) { column_like!(floor([x])) }),
        // ln(<num>)                                           	 Returns the natural logarithm.
        "ln" => eval_fn!(ln [args] (x) { column_like!(log([x])) }),
        // log(<num>,<base>)                                   	 Returns the logarithm of <num> using <base> as the base. If <base> is omitted, base 10 is used.
        "log" => eval_fn!(log [args] (x, base) { column_like!(log([x], [base])) }),
        // pi()                                                	 Returns the constant pi to 11 digits of precision.
        "pi" => eval_fn!(pi [args] () { column_like!(lit(3.141592653589793)) }),
        // pow(<num>,<exp>)                                    	 Returns <num> to the power of <exp>, <num><exp>.
        "pow" => eval_fn!(pow [args] (x, exp) { column_like!(pow([x], [exp])) }),
        // round(<num>,<precision>)                            	 Returns <num> rounded to the amount of decimal places specified by <precision>. The default is to round to an integer.
        "round" => eval_fn!(round [args] (x, precision: i64) {
            column_like!(round([x], [py_lit(precision)]))
        }),
        // sigfig(<num>)                                       	 Rounds <num> to the appropriate number of significant figures.
        "sigfig" => {
            unimplemented!("Unsupported function: {}", name)
        }
        // sqrt(<num>)                                         	 Returns the square root of the value.
        "sqrt" => eval_fn!(sqrt [args] (x) { column_like!(sqrt([x])) }),
        // sum(<num>,...)                                      	 Returns the sum of numerical values as an integer.
        "sum" => eval_fn!(sum [args] (x) { column_like!(sum([x])) }),
        // commands(<value>)                                   	 Returns a multivalued field that contains a list of the commands used in <value>.
        "commands" => {
            unimplemented!("Unsupported function: {}", name)
        }
        // mvappend(<values>)                                  	 Returns a multivalue result based on all of values specified.
        "mvappend" => eval_fn!(mvappend [args -> mapped_args] () {
            column_like!(concat(mapped_args))
        }),
        // mvcount(<mv>)                                       	 Returns the count of the number of values in the specified field.
        "mvcount" => eval_fn!(mvcount [args] (column) {
            column_like!(size([column]))
        }),
        // mvdedup(<mv>)                                       	 Removes all of the duplicate values from a multivalue field.
        "mvdedup" => {
            unimplemented!("Unsupported function: {}", name)
        }
        // mvfilter(<predicate>)                               	 Filters a multivalue field based on an arbitrary Boolean expression.
        "mvfilter" => {
            unimplemented!("Unsupported function: {}", name)
        }
        // mvfind(<mv>,<regex>)                                	 Finds the index of a value in a multivalue field that matches the regular expression.
        "mvfind" => {
            unimplemented!("Unsupported function: {}", name)
        }
        // mvindex(<mv>,<start>,<end>)                         	 Returns a subset of the multivalue field using the start and end index values.
        "mvindex" => eval_fn!(mvindex [args] (x, start: i64, end: i64) {
             column_like!(slice([x], [py_lit(start + 1)], [py_lit(end - start + 1)]))
        }),
        // mvjoin(<mv>,<delim>)                                	 Takes all of the values in a multivalue field and appends the values together using a delimiter.
        "mvjoin" => {
            unimplemented!("Unsupported function: {}", name)
        }
        // mvmap(<mv>,<expression>)                            	 This function iterates over the values of a multivalue field, performs an operation using the <expression> on each value, and returns a multivalue field with the list of results.
        "mvmap" => {
            unimplemented!("Unsupported function: {}", name)
        }
        // mvrange(<start>,<end>,<step>)                       	 Creates a multivalue field based on a range of specified numbers.
        "mvrange" => {
            unimplemented!("Unsupported function: {}", name)
        }
        // mvsort(<mv>)                                        	 Returns the values of a multivalue field sorted lexicographically.
        "mvsort" => {
            unimplemented!("Unsupported function: {}", name)
        }
        // mvzip(<mv_left>,<mv_right>,<delim>)                 	 Combines the values in two multivalue fields. The delimiter is used to specify a delimiting character to join the two values.
        "mvzip" => {
            unimplemented!("Unsupported function: {}", name)
        }
        // mv_to_json_array(<field>, <inver_types>)            	 Maps the elements of a multivalue field to a JSON array.
        "mv_to_json_array" => {
            unimplemented!("Unsupported function: {}", name)
        }
        // split(<str>,<delim>)                                	 Splits the string values on the delimiter and returns the string values as a multivalue field.
        "split" => eval_fn!(split [args] (x) { column_like!(split([x])) }),
        // avg(<values>)                                       	 Returns the average of numerical values as an integer.
        "avg" => eval_fn!(avg [args] (x) { column_like!(avg([x])) }),
        // max(<values>)                                       	 Returns the maximum of a set of string or numeric values.
        "max" => eval_fn!(max [args] (x, y) { column_like!(greatest([x], [y])) }),
        // min(<values>)                                       	 Returns the minimum of a set of string or numeric values.
        "min" => eval_fn!(min [args] (x, y) { column_like!(least([x], [y])) }),
        // random()                                            	 Returns a pseudo-random integer ranging from zero to 2^31-1.
        "random" => eval_fn!(random [args] () { column_like!(rand()) }),
        // len(<str>)                                          	 Returns the count of the number of characters, not bytes, in the string.
        "len" => eval_fn!(len [args] (x) { column_like!(length([x])) }),
        // lower(<str>)                                        	 Converts the string to lowercase.
        "lower" => eval_fn!(lower [args] (x) { column_like!(lower([x])) }),
        // ltrim(<str>,<trim_chars>)                           	 Removes characters from the left side of a string.
        "ltrim" => eval_fn!(ltrim [args] (x, chars) { column_like!(ltrim([x], [chars])) }),
        // replace(<str>,<regex>,<replacement>)                	 Substitutes the replacement string for every occurrence of the regular expression in the string.
        "replace" => eval_fn!(replace [args] (input, regex: String, replacement: String) {
            column_like!(regexp_replace([input.clone()], [py_lit(regex)], [py_lit(replacement)]))
        }),
        // rtrim(<str>,<trim_chars>)                           	 Removes the trim characters from the right side of the string.
        "rtrim" => eval_fn!(rtrim [args] (x, chars) { column_like!(rtrim([x], [chars])) }),
        // spath(<value>,<path>)                               	 Extracts information from the structured data formats XML and JSON.
        "spath" => {
            unimplemented!("Unsupported function: {}", name)
        }
        // substr(<str>,<start>,<length>)                      	 Returns a substring of a string, beginning at the start index. The length of the substring specifies the number of character to return.
        "substr" => eval_fn!(substr [args] (column, start: i64, length: i64) {
            column_like!(substring([column], [py_lit(start)], [py_lit(length)]))
        }),
        // trim(<str>,<trim_chars>)                            	 Trim characters from both sides of a string.
        "trim" => eval_fn!(trim [args] (x, chars) { column_like!(trim([x], [chars])) }),
        // upper(<str>)                                        	 Returns the string in uppercase.
        "upper" => eval_fn!(upper [args] (x) { column_like!(upper([x])) }),
        // urldecode(<url>)                                    	 Replaces URL escaped characters with the original characters.
        "urldecode" => {
            unimplemented!("Unsupported function: {}", name)
        }
        // acos(X)                                             	 Computes the arc cosine of X.
        "acos" => eval_fn!(acos [args] (x) { column_like!(acos([x])) }),
        // acosh(X)                                            	 Computes the arc hyperbolic cosine of X.
        "acosh" => eval_fn!(acosh [args] (x) { column_like!(acosh([x])) }),
        // asin(X)                                             	 Computes the arc sine of X.
        "asin" => eval_fn!(asin [args] (x) { column_like!(asin([x])) }),
        // asinh(X)                                            	 Computes the arc hyperbolic sine of X.
        "asinh" => eval_fn!(asinh [args] (x) { column_like!(asinh([x])) }),
        // atan(X)                                             	 Computes the arc tangent of X.
        "atan" => eval_fn!(atan [args] (x) { column_like!(atan([x])) }),
        // atan2(X,Y)                                          	 Computes the arc tangent of X,Y.
        "atan2" => eval_fn!(atan2 [args] (x, y) { column_like!(atan2([x], [y])) }),
        // atanh(X)                                            	 Computes the arc hyperbolic tangent of X.
        "atanh" => eval_fn!(atanh [args] (x) { column_like!(atanh([x])) }),
        // cos(X)                                              	 Computes the cosine of an angle of X radians.
        "cos" => eval_fn!(cos [args] (x) { column_like!(cos([x])) }),
        // cosh(X)                                             	 Computes the hyperbolic cosine of X radians.
        "cosh" => eval_fn!(cosh [args] (x) { column_like!(cosh([x])) }),
        // hypot(X,Y)                                          	 Computes the hypotenuse of a triangle.
        "hypot" => eval_fn!(hypot [args] (x, y) { column_like!(hypot([x], [y])) }),
        // sin(X)                                              	 Computes the sine of X.
        "sin" => eval_fn!(sin [args] (x) { column_like!(sin([x])) }),
        // sinh(X)                                             	 Computes the hyperbolic sine of X.
        "sinh" => eval_fn!(sinh [args] (x) { column_like!(sinh([x])) }),
        // tan(X)                                              	 Computes the tangent of X.
        "tan" => eval_fn!(tan [args] (x) { column_like!(tan([x])) }),
        // tanh(X)                                             	 Computes the hyperbolic tangent of X.
        "tanh" => eval_fn!(tanh [args] (x) { column_like!(tanh([x])) }),

        // Fallback
        // (n, l) => bail!("Unsupported function: {}([{} args])", n, l),
        name => {
            let args: Vec<Expr> = map_args(args)?;
            // let spark_name = match SIMPLE_FUNC_MAP.get(name).cloned() {
            //     // Unknown function
            //     None => {
            //         warn!("Unknown eval function encountered, returning as is: {}", name);
            //         name
            //     },
            //     // Known function with no known trivial mapping
            //     Some(None) => name,
            //     // Known function with known trivial mapping
            //     Some(Some(spark_name)) => spark_name,
            // };
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
