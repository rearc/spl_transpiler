import warnings
from functools import reduce

from pyspark.sql import functions as F

from spl_transpiler.runtime import udfs
from spl_transpiler.runtime.base import enforce_types, Expr, TimeSpan, TimeScale
from spl_transpiler.runtime.functions import utils


#         // Bitwise functions
#         // bit_and(<values>)                                   	 Bitwise AND function that takes two or more non-negative integers as arguments and sequentially performs logical bitwise AND on them.
#         "bit_and" => {
#             function_transform!(bit_and [args] (a, b) { column_like!([a].bitwiseAND([b])) })
#         }


@enforce_types
def bit_and(a: Expr, b: Expr) -> Expr:
    a.to_pyspark_expr().bitwiseAND(b.to_pyspark_expr())


#         // bit_or(<values>)                                    	 Bitwise OR function that takes two or more non-negative integers as arguments and sequentially performs bitwise OR on them.
#         "bit_or" => function_transform!(bit_or [args] (a, b) { column_like!([a].bitwiseOR([b])) }),


@enforce_types
def bit_or(a: Expr, b: Expr) -> Expr:
    return a.to_pyspark_expr().bitwiseOR(b.to_pyspark_expr())


#         // bit_not(<value>, <bitmask>)                         	 Bitwise NOT function that takes a non-negative as an argument and inverts every bit in the binary representation of that number. It also takes an optional second argument that acts as a bitmask.
#         "bit_not" => function_transform!(bit_not [args] (x) { column_like!(bitwise_not([x])) }),


@enforce_types
def bit_not(x: Expr) -> Expr:
    return F.bitwise_not(x.to_pyspark_expr())


#         // bit_xor(<values>)                                   	 Bitwise XOR function that takes two or more non-negative integers as arguments and sequentially performs bitwise XOR of each of the given arguments.
#         "bit_xor" => {
#             function_transform!(bit_or [args] (a, b) { column_like!([a].bitwiseXOR([b])) })
#         }


@enforce_types
def bit_xor(a: Expr, b: Expr) -> Expr:
    return a.to_pyspark_expr().bitwiseXOR(b.to_pyspark_expr())


#         // bit_shift_left(<value>, <shift_offset>)             	 Logical left shift function that takes two non-negative integers as arguments and shifts the binary representation of the first integer over to the left by the specified shift amount.
#         "bit_shift_left" => {
#             function_transform!(bit_not [args] (x, offset: i64) { column_like!(shiftleft([x], [py_lit(offset)])) })
#         }


@enforce_types
def bit_shift_left(x: Expr, offset: int) -> Expr:
    return F.shiftleft(x.to_pyspark_expr(), offset)


#         // bit_shift_right(<value>, <shift_offset>)            	 Logical right shift function that takes two non-negative integers as arguments and shifts the binary representation of the first integer over to the right by the specified shift amount.
#         "bit_shift_right" => {
#             function_transform!(bit_not [args] (x, offset: i64) { column_like!(shiftright([x], [py_lit(offset)])) })
#         }


@enforce_types
def bit_shift_right(x: Expr, offset: int) -> Expr:
    return F.shiftright(x.to_pyspark_expr(), offset)


#
#         // Comparison and Conditional functions
#         // case(<condition>,<value>,...)                       	 Accepts alternating conditions and values. Returns the first value for which the condition evaluates to TRUE.
#         "case" => function_transform!(case [args -> mapped_args] () {
#             ensure!(mapped_args.len() % 2 == 0, "case() function requires an even number of arguments (conditions and values).");
#             let mut c = None;
#             for i in (0..mapped_args.len()).step_by(2) {
#                 let condition = mapped_args[i].clone();
#                 let value = mapped_args[i + 1].clone();
#                 c = match c {
#                     None => Some(column_like!(when([condition], [value]))),
#                     Some(c) => Some(column_like!([c].when([condition], [value])))
#                 }
#             }
#             ensure!(c.is_some(), "No condition-value pairs found in case() function.");
#             c.unwrap()
#         }),


@enforce_types
def case(*args: Expr) -> Expr:
    if len(args) % 2 != 0:
        raise ValueError(
            "case() function requires an even number of arguments (conditions and values)."
        )

    if not args:
        raise ValueError("No arguments found in case() function.")

    result = F
    for condition, value in zip(args[::2], args[1::2]):
        result.when(condition.to_pyspark_expr(), value.to_pyspark_expr())

    return result


#         // cidrmatch(<cidr>,<ip>)                              	 Returns TRUE when an IP address, <ip>, belongs to a particular CIDR subnet, <cidr>.
#         "cidrmatch" => function_transform!(cidrmatch [args] (cidr: String, col: String) {
#             column_like!(expr("cidr_match('{}', {})", cidr, col))
#         }),


@enforce_types
def cidrmatch(cidr: Expr, ip: Expr) -> Expr:
    return udfs.cidr_match(cidr.to_pyspark_expr(), ip.to_pyspark_expr())


#         // coalesce(<values>)                                  	 Takes one or more values and returns the first value that is not NULL.
#         "coalesce" => function_transform!(coalesce [args -> mapped_args] () {
#             column_like!(coalesce(mapped_args))
#         }),


@enforce_types
def coalesce(*args: Expr) -> Expr:
    return F.coalesce(*(arg.to_pyspark_expr() for arg in args))


#         // false()                                             	 Returns FALSE.
#         "false" => function_transform!(false [args] () { column_like!(lit(false)) }),


@enforce_types
def false() -> Expr:
    return F.lit(False)


#         // if(<predicate>,<true_value>,<false_value>)          	 If the <predicate> expression evaluates to TRUE, returns the <true_value>, otherwise the function returns the <false_value>.
#         "if" => function_transform!(if [args](condition, then_expr, else_expr) {
#             column_like!([when([condition], [then_expr])].otherwise([else_expr]))
#         }),


@enforce_types
def if_(condition: Expr, true_value: Expr, false_value: Expr) -> Expr:
    return F.when(condition.to_pyspark_expr(), true_value.to_pyspark_expr()).otherwise(
        false_value.to_pyspark_expr()
    )


#         // in(<field>,<list>)                                  	 Returns TRUE if one of the values in the list matches a value that you specify.
#         "in" => function_transform!(in [args] (x, vals) { column_like!([x].isin([vals])) }),


@enforce_types
def in_(field: Expr, values: list) -> Expr:
    return field.to_pyspark_expr().isin(values)


#         // like(<str>,<pattern>)                               	 Returns TRUE only if <str> matches <pattern>.
#         "like" => {
#             function_transform!(like [args] (x, pattern) { column_like!(like([x], [pattern])) })
#         }


@enforce_types
def like(string: Expr, pattern: Expr) -> Expr:
    return F.like(string.to_pyspark_expr(), pattern.to_pyspark_expr())


#         // lookup(<lookup_table>, <json_object>, <json_array>) 	 Performs a CSV lookup. Returns the output field or fields in the form of a JSON object. The lookup() function is available only to Splunk Enterprise users.
#         "lookup" => {
#             bail!("UNIMPLEMENTED: Unsupported function: {}", name)
#         }


@enforce_types
def lookup(lookup_table: Expr, json_object: Expr, json_array: Expr) -> Expr:
    raise NotImplementedError("lookup() function is not implemented.")


#         // match(<str>, <regex>)                               	 Returns TRUE if the regular expression <regex> finds a match against any substring of the string value <str>. Otherwise returns FALSE.
#         "match" => {
#             function_transform!(match [args] (x, regex) { column_like!(regexp_like([x], [regex])) })
#         }


@enforce_types
def match(string: Expr, regex: Expr) -> Expr:
    return F.regexp_like(string.to_pyspark_expr(), regex.to_pyspark_expr())


#         // null()                                              	 This function takes no arguments and returns NULL.
#         "null" => function_transform!(null [args] () { column_like!(lit(None)) }),


@enforce_types
def null() -> Expr:
    return F.lit(None)


#         // nullif(<field1>,<field2>)                           	 Compares the values in two fields and returns NULL if the value in <field1> is equal to the value in <field2>. Otherwise returns the value in <field1>.
#         "nullif" => function_transform!(nullif [args] (x, y) { column_like!(nullif([x], [y])) }),


@enforce_types
def nullif(field1: Expr, field2: Expr) -> Expr:
    return F.nullif(field1.to_pyspark_expr(), field2.to_pyspark_expr())


#         // searchmatch(<search_str>)                           	 Returns TRUE if the event matches the search string.
#         "searchmatch" => {
#             bail!("UNIMPLEMENTED: Unsupported function: {}", name)
#         }


@enforce_types
def searchmatch(search_str: Expr) -> Expr:
    raise NotImplementedError("searchmatch() function is not implemented in PySpark.")


#         // true()                                              	 Returns TRUE.
#         "true" => function_transform!(true [args] () { column_like!(lit(true)) }),


@enforce_types
def true() -> Expr:
    return F.lit(True)


#         // validate(<condition>, <value>,...)                  	 Takes a list of conditions and values and returns the value that corresponds to the condition that evaluates to FALSE. This function defaults to NULL if all conditions evaluate to TRUE. This function is the opposite of the case function.
#         "validate" => function_transform!(validate [args -> mapped_args] () {
#             ensure!(mapped_args.len() % 2 == 0, "case() function requires an even number of arguments (conditions and values).");
#             let mut c = None;
#             for i in (0..mapped_args.len()).step_by(2) {
#                 let condition = mapped_args[i].clone();
#                 let value = mapped_args[i + 1].clone();
#                 c = match c {
#                     None => Some(column_like!(when([~[condition]], [value]))),
#                     Some(c) => Some(column_like!([c].when([~[condition]], [value])))
#                 }
#             }
#             ensure!(c.is_some(), "No condition-value pairs found in case() function.");
#             c.unwrap()
#         }),


@enforce_types
def validate(*args) -> Expr:
    if len(args) % 2 != 0:
        raise ValueError(
            "validate() function requires an even number of arguments (conditions and values)."
        )

    if not args:
        raise ValueError("No arguments found in validate() function.")

    result = F

    for condition, value in zip(args[::2], args[1::2]):
        result = result.when(~condition.to_pyspark_expr(), value.to_pyspark_expr())

    return result


#
#         // Conversion functions
#         // ipmask(<mask>,<ip>)                                 	 Generates a new masked IP address by applying a mask to an IP address using a bitwise AND operation.
#         "ipmask" => {
#             bail!("UNIMPLEMENTED: Unsupported function: {}", name)
#         }


@enforce_types
def ipmask(mask: Expr, ip: Expr) -> Expr:
    return udfs.ipmask(mask.to_pyspark_expr(), ip.to_pyspark_expr())


#         // printf(<format>,<arguments>)                        	 Creates a formatted string based on a format description that you provide.
#         "printf" => bail!("UNIMPLEMENTED: Unsupported function: {}", name),


@enforce_types
def printf(format_str: Expr, *args: Expr) -> Expr:
    return udfs.printf(
        format_str.to_pyspark_expr(), [arg.to_pyspark_expr() for arg in args]
    )


#         // tonumber(<str>,<base>)                              	 Converts a string to a number.
#         "tonumber" => function_transform!(tonumber [args -> mapped_args] (x) {
#             ensure!(mapped_args.is_empty(), "UMIMPLEMENTED: Unsupported `base` argument provided to `tonumber`");
#             column_like!([x].cast([py_lit("double")]))
#         }),


@enforce_types
def tonumber(value: Expr, base: int = 10) -> Expr:
    return udfs.tonumber(value.to_pyspark_expr(), base=base)


#         // tostring(<value>,<format>)                          	 Converts the input, such as a number or a Boolean value, to a string.
#         "tostring" => function_transform!(tostring [args -> mapped_args] (x) {
#             ensure!(mapped_args.is_empty(), "UMIMPLEMENTED: Unsupported `format` argument provided to `tostring`");
#             column_like!([x].cast([py_lit("string")]))
#         }),


@enforce_types
def tostring(value, format=None):
    return udfs.tostring(value.to_pyspark_expr(), format=format)


#
#         // Cryptographic functions
#         // md5(<str>)                                          	 Computes the md5 hash for the string value.
#         "md5" => function_transform!(md5 [args] (x) { column_like!(md5([x])) }),


@enforce_types
def md5(value: Expr) -> Expr:
    return F.md5(value.to_pyspark_expr())


#         // sha1(<str>)                                         	 Computes the sha1 hash for the string value.
#         "sha1" => function_transform!(sha1 [args] (x) { column_like!(sha1([x])) }),


@enforce_types
def sha1(value: Expr) -> Expr:
    return F.sha1(value.to_pyspark_expr())


#         // sha256(<str>)                                       	 Computes the sha256 hash for the string value.
#         "sha256" => {
#             function_transform!(sha256 [args] (x) { column_like!(sha2([x], [py_lit(256)])) })
#         }


@enforce_types
def sha256(value: Expr) -> Expr:
    return F.sha2(value.to_pyspark_expr(), 256)


#         // sha512(<str>)                                       	 Computes the sha512 hash for the string value.
#         "sha512" => {
#             function_transform!(sha512 [args] (x) { column_like!(sha2([x], [py_lit(512)])) })
#         }


@enforce_types
def sha512(value: Expr) -> Expr:
    return F.sha2(value.to_pyspark_expr(), 512)


#
#         // Date and Time functions
#         // now()                                               	 Returns the time that the search was started.
#         "now" => function_transform!(now [args] () { column_like!(current_timestamp()) }),


@enforce_types
def now() -> Expr:
    return F.current_timestamp()


#         // relative_time(<time>,<specifier>)                   	 Adjusts the time by a relative time specifier.
#         "relative_time" => function_transform!(now [args ...] (time) {
#             let relative_time_spec = match args.get(1) {
#                 Some(ast::Expr::Leaf(ast::LeafExpr::Constant(ast::Constant::SnapTime(snap_time)))) => snap_time,
#                 Some(v) => bail!("`relative_time` function requires a second argument of type SnapTime, got {:?}", v),
#                 None => bail!("`relative_time` function requires a second argument"),
#             };
#             let time = column_like!(to_timestamp([time]));
#             let time = match relative_time_spec.span.clone() {
#                 None => time,
#                 Some(time_span) => offset_time(time, time_span)
#             };
#             let time = round_time(time, relative_time_spec.snap.clone())?;
#
#             match relative_time_spec.snap_offset.clone() {
#                 None => time,
#                 Some(snap_offset) => offset_time(time, snap_offset)
#             }
#         }),


@enforce_types
def relative_time(
    time: Expr,
    span: TimeSpan | None = None,
    snap: TimeScale | None = None,
    snap_offset: TimeSpan | None = None,
) -> Expr:
    time = F.to_timestamp(time.to_pyspark_expr())
    if span is not None:
        time = utils.offset_time(time, span)
    if snap is not None:
        time = utils.round_time(time, snap)
    if snap_offset is not None:
        time = utils.offset_time(time, snap_offset)
    return time


#         // strftime(<time>,<format>)                           	 Takes a UNIX time and renders it into a human readable format.
#         "strftime" => function_transform!(strftime [args] (date, format: String) {
#             let converted_time_format = convert_time_format(format)?;
#             column_like!(date_format([date], [py_lit(converted_time_format)]))
#         }),


@enforce_types
def strftime(date: Expr, format: str, is_spl_format=True) -> Expr:
    if is_spl_format:
        format = utils.convert_time_format(format)
    return F.date_format(date.to_pyspark_expr(), format)


#         // strptime(<str>,<format>)                            	 Takes a human readable time and renders it into UNIX time.
#         "strptime" => function_transform!(strptime [args] (date, format: String) {
#             let converted_time_format = convert_time_format(format)?;
#             column_like!(to_timestamp([date], [py_lit(converted_time_format)]))
#         }),


@enforce_types
def strptime(date: str, format: str, is_spl_format=True) -> Expr:
    if is_spl_format:
        format = utils.convert_time_format(format)
    return F.to_timestamp(date, format)


#         // time()                                              	 The time that eval function was computed. The time will be different for each event, based on when the event was processed.
#         "time" => function_transform!(time [args] () { column_like!(current_timestamp()) }),


@enforce_types
def time() -> Expr:
    return F.current_timestamp()


#
#         // Informational functions
#         // isbool(<value>)                                     	 Returns TRUE if the field value is Boolean.
#         "isbool" => {
#             bail!("UNIMPLEMENTED: Unsupported function: {}", name)
#         }


@enforce_types
def isbool(value: Expr) -> Expr:
    raise NotImplementedError("`isbool` function is not implemented")


#         // isint(<value>)                                      	 Returns TRUE if the field value is an integer.
#         "isint" => {
#             bail!("UNIMPLEMENTED: Unsupported function: {}", name)
#         }


@enforce_types
def isint(value: Expr) -> Expr:
    raise NotImplementedError("`isint` function is not implemented")


#         // isnotnull(<value>)                                  	 Returns TRUE if the field value is not NULL.
#         "isnotnull" => function_transform!(isnotnull [args] (x) { column_like!(isnotnull([x])) }),


@enforce_types
def isnotnull(value: Expr) -> Expr:
    return F.isnotnull(value.to_pyspark_expr())


#         // isnull(<value>)                                     	 Returns TRUE if the field value is NULL.
#         "isnull" => function_transform!(isnull [args] (x) { column_like!(isnull([x])) }),


@enforce_types
def isnull(value: Expr) -> Expr:
    return F.isnull(value.to_pyspark_expr())


#         // isnum(<value>)                                      	 Returns TRUE if the field value is a number.
#         "isnum" => {
#             bail!("UNIMPLEMENTED: Unsupported function: {}", name)
#         }


@enforce_types
def isnum(value: Expr) -> Expr:
    raise NotImplementedError("`isnum` function is not implemented")


#         // isstr(<value>)                                      	 Returns TRUE if the field value is a string.
#         "isstr" => {
#             bail!("UNIMPLEMENTED: Unsupported function: {}", name)
#         }


@enforce_types
def isstr(value: Expr) -> Expr:
    raise NotImplementedError("`isstr` function is not implemented")


#         // typeof(<value>)                                     	 Returns a string that indicates the field type, such as Number, String, Boolean, and so forth
#         "typeof" => {
#             bail!("UNIMPLEMENTED: Unsupported function: {}", name)
#         }


@enforce_types
def typeof(value: Expr) -> Expr:
    raise NotImplementedError("`typeof` function is not implemented")


#
#         // JSON functions
#         // json_object(<members>)                              	 Creates a new JSON object from members of key-value pairs.
#         "json_object" => {
#             bail!("UNIMPLEMENTED: Unsupported function: {}", name)
#         }


@enforce_types
def json_object(*members) -> Expr:
    raise NotImplementedError("`json_object` function is not implemented")


#         // json_append(<json>, <path_value_pairs>)             	 Appends values to the ends of indicated arrays within a JSON document.
#         "json_append" => {
#             bail!("UNIMPLEMENTED: Unsupported function: {}", name)
#         }


@enforce_types
def json_append(json: Expr, **values: Expr) -> Expr:
    raise NotImplementedError("`json_append` function is not implemented")


#         // json_array(<values>)                                	 Creates a JSON array using a list of values.
#         "json_array" => {
#             bail!("UNIMPLEMENTED: Unsupported function: {}", name)
#         }


@enforce_types
def json_array(*values: Expr) -> Expr:
    raise NotImplementedError("`json_array` function is not implemented")


#         // json_array_to_mv(<json_array>, <boolean>)           	 Maps the elements of a proper JSON array into a multivalue field.
#         "json_array_to_mv" => {
#             bail!("UNIMPLEMENTED: Unsupported function: {}", name)
#         }


@enforce_types
def json_array_to_mv(json_array: Expr, boolean: bool) -> Expr:
    raise NotImplementedError("`json_array_to_mv` function is not implemented")


#         // json_extend(<json>, <path_value_pairs>)             	 Flattens arrays into their component values and appends those values to the ends of indicated arrays within a valid JSON document.
#         "json_extend" => {
#             bail!("UNIMPLEMENTED: Unsupported function: {}", name)
#         }


@enforce_types
def json_extend(json: Expr, **values: Expr) -> Expr:
    raise NotImplementedError("`json_extend` function is not implemented")


#         // json_extract(<json>, <paths>)                       	 This function returns a value from a piece JSON and zero or more paths. The value is returned in either a JSON array, or a Splunk software native type value.
#         "json_extract" => {
#             bail!("UNIMPLEMENTED: Unsupported function: {}", name)
#         }


@enforce_types
def json_extract(json: Expr, *paths: str) -> Expr:
    raise NotImplementedError("`json_extract` function is not implemented")


#         // json_extract_exact(<json>,<keys>)                   	 Returns Splunk software native type values from a piece of JSON by matching literal strings in the event and extracting them as keys.
#         "json_extract_exact" => {
#             bail!("UNIMPLEMENTED: Unsupported function: {}", name)
#         }


@enforce_types
def json_extract_exact(json: Expr, *keys: str) -> Expr:
    raise NotImplementedError("`json_extract_exact` function is not implemented")


#         // json_keys(<json>)                                   	 Returns the keys from the key-value pairs in a JSON object as a JSON array.
#         "json_keys" => function_transform!(json_keys [args] (x) {
#             column_like!(json_object_keys([x]))
#         }),


@enforce_types
def json_keys(json: Expr) -> Expr:
    return F.json_object_keys(json.to_pyspark_expr())


#         // json_set(<json>, <path_value_pairs>)                	 Inserts or overwrites values for a JSON node with the values provided and returns an updated JSON object.
#         "json_set" => {
#             bail!("UNIMPLEMENTED: Unsupported function: {}", name)
#         }


@enforce_types
def json_set(json: Expr, **values: Expr) -> Expr:
    raise NotImplementedError("`json_set` function is not implemented")


#         // json_set_exact(<json>,<key_value_pairs>)            	 Uses provided key-value pairs to generate or overwrite a JSON object.
#         "json_set_exact" => {
#             bail!("UNIMPLEMENTED: Unsupported function: {}", name)
#         }


@enforce_types
def json_set_exact(json: Expr, **values: Expr) -> Expr:
    raise NotImplementedError("`json_set_exact` function is not implemented")


#         // json_valid(<json>)                                  	 Evaluates whether piece of JSON uses valid JSON syntax and returns either TRUE or FALSE.
#         "json_valid" => function_transform!(json_valid [args] (x) {
#             column_like!([get_json_object([x], [py_lit("$")])].isNotNull())
#         }),


@enforce_types
def json_valid(json: Expr) -> Expr:
    return F.get_json_object(json.to_pyspark_expr(), "$").isNotNull()


#
#         // Mathematical functions
#         // abs(<num>)                                          	 Returns the absolute value.
#         "abs" => function_transform!(abs [args] (x) { column_like!(abs([x])) }),


@enforce_types
def abs(num: Expr) -> Expr:
    return F.abs(num.to_pyspark_expr())


#         // ceiling(<num>)                                      	 Rounds the value up to the next highest integer.
#         "ceiling" => function_transform!(ceiling [args] (x) { column_like!(ceil([x])) }),


@enforce_types
def ceiling(num: Expr) -> Expr:
    return F.ceil(num.to_pyspark_expr())


#         // exact(<expression>)                                 	 Returns the result of a numeric eval calculation with a larger amount of precision in the formatted output.
#         "exact" => function_transform!(exact [args] (x) {
#             warn!("`exact()` is meaningless in Spark since significant figures are not tracked, use proper precision for your expression inputs instead");
#             column_like!([x].cast([py_lit("double")]))
#         }),


@enforce_types
def exact(expression: Expr) -> Expr:
    warnings.warn(
        "`exact()` is meaningless in Spark since significant figures are not tracked, use proper precision for your expression inputs instead"
    )
    return expression.to_pyspark_expr().cast("double")


#         // exp(<num>)                                          	 Returns the exponential function eN.
#         "exp" => function_transform!(exp [args] (x) { column_like!(exp([x])) }),


@enforce_types
def exp(num: Expr) -> Expr:
    return F.exp(num.to_pyspark_expr())


#         // floor(<num>)                                        	 Rounds the value down to the next lowest integer.
#         "floor" => function_transform!(floor [args] (x) { column_like!(floor([x])) }),


@enforce_types
def floor(num: Expr) -> Expr:
    return F.floor(num.to_pyspark_expr())


#         // ln(<num>)                                           	 Returns the natural logarithm.
#         "ln" => function_transform!(ln [args] (x) { column_like!(log([x])) }),


@enforce_types
def ln(num: Expr) -> Expr:
    return F.log(num.to_pyspark_expr())


#         // log(<num>,<base>)                                   	 Returns the logarithm of <num> using <base> as the base. If <base> is omitted, base 10 is used.
#         "log" => function_transform!(log [args] (x, base) { column_like!(log([x], [base])) }),


@enforce_types
def log(num: Expr, base: int = 10) -> Expr:
    return F.log(num.to_pyspark_expr(), base)


#         // pi()                                                	 Returns the constant pi to 11 digits of precision.
#         "pi" => function_transform!(pi [args] () { column_like!(lit(3.141592653589793)) }),


@enforce_types
def pi() -> Expr:
    import math

    return F.lit(math.pi)


#         // pow(<num>,<exp>)                                    	 Returns <num> to the power of <exp>, <num><exp>.
#         "pow" => function_transform!(pow [args] (x, exp) { column_like!(pow([x], [exp])) }),


@enforce_types
def pow(num: Expr, exp: Expr) -> Expr:
    return F.pow(num.to_pyspark_expr(), exp.to_pyspark_expr())


#         // round(<num>,<precision>)                            	 Returns <num> rounded to the amount of decimal places specified by <precision>. The default is to round to an integer.
#         "round" => function_transform!(round [args] (x, precision: i64) {
#             column_like!(round([x], [py_lit(precision)]))
#         }),


@enforce_types
def round(num: Expr, precision: int = 0) -> Expr:
    return F.round(num.to_pyspark_expr(), precision)


#         // sigfig(<num>)                                       	 Rounds <num> to the appropriate number of significant figures.
#         "sigfig" => {
#             bail!("Unsupported function `{}`: significant figures are not tracked in Spark, use proper precision for your expression inputs instead", name);
#         }

#         // sqrt(<num>)                                         	 Returns the square root of the value.
#         "sqrt" => function_transform!(sqrt [args] (x) { column_like!(sqrt([x])) }),


@enforce_types
def sqrt(num: Expr) -> Expr:
    return F.sqrt(num.to_pyspark_expr())


#         // sum(<num>,...)                                      	 Returns the sum of numerical values as an integer.
#         "sum" => function_transform!(sum [args] (x) { column_like!(sum([x])) }),


@enforce_types
def sum_(*nums: Expr) -> Expr:
    # Adds the columns together, creating a new column whose value is the sum of the others
    return sum(*(n.to_pyspark_expr() for n in nums))


#
#         // Multivalue eval functions
#         // commands(<value>)                                   	 Returns a multivalued field that contains a list of the commands used in <value>.
#         "commands" => {
#             bail!("UNIMPLEMENTED: Unsupported function: {}", name)
#         }


@enforce_types
def commands(value: Expr) -> Expr:
    raise NotImplementedError("commands() function is not implemented.")


#         // mvappend(<values>)                                  	 Returns a multivalue result based on all of values specified.
#         "mvappend" => function_transform!(mvappend [args -> mapped_args] () {
#             column_like!(concat(mapped_args))
#         }),


@enforce_types
def mvappend(values: Expr) -> Expr:
    return F.concat(*[v.to_pyspark_expr() for v in values])


#         // mvcount(<mv>)                                       	 Returns the count of the number of values in the specified field.
#         "mvcount" => function_transform!(mvcount [args] (column) {
#             column_like!(size([column]))
#         }),


@enforce_types
def mvcount(column: Expr) -> Expr:
    return F.size(column.to_pyspark_expr())


#         // mvdedup(<mv>)                                       	 Removes all of the duplicate values from a multivalue field.
#         "mvdedup" => {
#             bail!("UNIMPLEMENTED: Unsupported function: {}", name)
#         }


@enforce_types
def mvdedup(column: Expr) -> Expr:
    raise NotImplementedError("mvdedup() function is not implemented.")


#         // mvfilter(<predicate>)                               	 Filters a multivalue field based on an arbitrary Boolean expression.
#         "mvfilter" => {
#             bail!("UNIMPLEMENTED: Unsupported function: {}", name)
#         }


@enforce_types
def mvfilter(predicate: Expr) -> Expr:
    raise NotImplementedError("mvfilter() function is not implemented.")


#         // mvfind(<mv>,<regex>)                                	 Finds the index of a value in a multivalue field that matches the regular expression.
#         "mvfind" => {
#             bail!("UNIMPLEMENTED: Unsupported function: {}", name)
#         }


@enforce_types
def mvfind(column: Expr, regex: Expr) -> Expr:
    raise NotImplementedError("mvfind() function is not implemented.")


#         // mvindex(<mv>,<start>,<end>)                         	 Returns a subset of the multivalue field using the start and end index values.
#         "mvindex" => function_transform!(mvindex [args ...] (x, start: i64) {
#             match args.get(2) {
#                 None => column_like!(get([x], [py_lit(start)])),
#                 Some(end) => {
#                     let end: i64 = map_arg(end) ?;
#                     column_like!(slice([x], [py_lit(start + 1)], [py_lit(end - start + 1)]))
#                 },
#             }
#         }),


@enforce_types
def mvindex(column: Expr, start: int, end: int | None = None) -> Expr:
    if end is None:
        return F.get(column.to_pyspark_expr(), start)
    else:
        return F.slice(column.to_pyspark_expr(), start + 1, end - start + 1)


#         // mvjoin(<mv>,<delim>)                                	 Takes all of the values in a multivalue field and appends the values together using a delimiter.
#         "mvjoin" => {
#             bail!("UNIMPLEMENTED: Unsupported function: {}", name)
#         }


@enforce_types
def mvjoin(column: Expr, delimiter: str) -> Expr:
    raise NotImplementedError("mvjoin() function is not implemented.")


#         // mvmap(<mv>,<expression>)                            	 This function iterates over the values of a multivalue field, performs an operation using the <expression> on each value, and returns a multivalue field with the list of results.
#         "mvmap" => {
#             bail!("UNIMPLEMENTED: Unsupported function: {}", name)
#         }


@enforce_types
def mvmap(column: Expr, expression: Expr) -> Expr:
    raise NotImplementedError("mvmap() function is not implemented.")


#         // mvrange(<start>,<end>,<step>)                       	 Creates a multivalue field based on a range of specified numbers.
#         "mvrange" => {
#             bail!("UNIMPLEMENTED: Unsupported function: {}", name)
#         }


@enforce_types
def mvrange(start: int, end: int, step: int) -> Expr:
    raise NotImplementedError("mvrange() function is not implemented.")


#         // mvsort(<mv>)                                        	 Returns the values of a multivalue field sorted lexicographically.
#         "mvsort" => {
#             bail!("UNIMPLEMENTED: Unsupported function: {}", name)
#         }


@enforce_types
def mvsort(column: Expr) -> Expr:
    raise NotImplementedError("mvsort() function is not implemented.")


#         // mvzip(<mv_left>,<mv_right>,<delim>)                 	 Combines the values in two multivalue fields. The delimiter is used to specify a delimiting character to join the two values.
#         "mvzip" => function_transform!(mvzip [args -> _mapped_args] (left, right) {
#             let delim: String = match args.get(2) {
#                 None => ",".into(),
#                 Some(delim) => map_arg(delim) ?,
#             };
#             let zip_function: Expr = PyLiteral(
#                 format !(r#"lambda left_, right_: F.concat_ws(r"{}", left_, right_)"#, delim)
#             ).into();
#             column_like!(
#                 zip_with(
#                     [left],
#                     [right],
#                     [zip_function]
#                 )
#             )
#         }),


@enforce_types
def mvzip(left: Expr, right: Expr, delimiter: str = ",") -> Expr:
    return F.zip_with(
        left, right, lambda left_, right_: F.concat_ws(delimiter, left_, right_)
    )


#         // mv_to_json_array(<field>, <inver_types>)            	 Maps the elements of a multivalue field to a JSON array.
#         "mv_to_json_array" => {
#             bail!("UNIMPLEMENTED: Unsupported function: {}", name)
#         }


@enforce_types
def mv_to_json_array(column: Expr, invert_types: bool = False) -> Expr:
    raise NotImplementedError("mv_to_json_array() function is not implemented.")


#         // split(<str>,<delim>)                                	 Splits the string values on the delimiter and returns the string values as a multivalue field.
#         "split" => function_transform!(split [args] (x, delim: String) {
#             column_like!(split([x], [py_lit(delim)]))
#         }),


@enforce_types
def split(column: Expr, delimiter: str) -> Expr:
    return F.split(column.to_pyspark_expr(), delimiter)


#
#         // Statistical eval functions
#         // avg(<values>)                                       	 Returns the average of numerical values as an integer.
#         "avg" => function_transform!(avg [args -> mapped_args] () {
#             ensure!(!mapped_args.is_empty(), "Must provide at least one column to average together");
#             let mut out: Option<Expr> = None;
#             let num_args = mapped_args.len() as i64;
#             for c in mapped_args {
#                 out = match out {
#                     Some(v) => Some(column_like!([v] + [c]).into()),
#                     None => Some(c),
#                 }
#             };
#             assert!(out.is_some(), "Failed to compute average");
#             column_like!([out.unwrap()] / [py_lit(num_args)])
#         }),


@enforce_types
def avg(*args: Expr) -> Expr:
    return sum(*(a.to_pyspark_expr() for a in args)) / len(args)


#         // max(<values>)                                       	 Returns the maximum of a set of string or numeric values.
#         "max" => function_transform!(max [args -> mapped_args] () {
#             ensure!(!mapped_args.is_empty(), "Must provide at least one column to max together");
#             let mut out: Option<Expr> = None;
#             for c in mapped_args {
#                 out = match out {
#                     Some(v) => Some(column_like!(greatest([v], [c])).into()),
#                     None => Some(c),
#                 }
#             };
#             assert!(out.is_some(), "Failed to compute average");
#             out.unwrap()
#         }),


@enforce_types
def max_(*args: Expr) -> Expr:
    return reduce(
        lambda a, b: F.greatest(a.to_pyspark_expr(), b.to_pyspark_expr()),
        args[1:],
        args[0].to_pyspark_expr(),
    )


#         // min(<values>)                                       	 Returns the minimum of a set of string or numeric values.
#         "min" => function_transform!(min [args -> mapped_args] () {
#             ensure!(!mapped_args.is_empty(), "Must provide at least one column to max together");
#             let mut out: Option<Expr> = None;
#             for c in mapped_args {
#                 out = match out {
#                     Some(v) => Some(column_like!(least([v], [c])).into()),
#                     None => Some(c),
#                 }
#             };
#             assert!(out.is_some(), "Failed to compute average");
#             out.unwrap()
#         }),


@enforce_types
def min_(*args: Expr) -> Expr:
    return reduce(
        lambda a, b: F.least(a.to_pyspark_expr(), b.to_pyspark_expr()),
        args[1:],
        args[0].to_pyspark_expr(),
    )


#         // random()                                            	 Returns a pseudo-random integer ranging from zero to 2^31-1.
#         "random" => function_transform!(random [args] () { column_like!(rand()) }),


@enforce_types
def random() -> Expr:
    return F.rand()


#
#         // Text functions
#         // len(<str>)                                          	 Returns the count of the number of characters, not bytes, in the string.
#         "len" => function_transform!(len [args] (x) { column_like!(length([x])) }),


@enforce_types
def len_(x: Expr):
    return F.length(x.to_pyspark_expr())


#         // lower(<str>)                                        	 Converts the string to lowercase.
#         "lower" => function_transform!(lower [args] (x) { column_like!(lower([x])) }),


@enforce_types
def lower(x: Expr):
    return F.lower(x.to_pyspark_expr())


#         // ltrim(<str>,<trim_chars>)                           	 Removes characters from the left side of a string.
#         "ltrim" => {
#             function_transform!(ltrim [args] (x, chars) { column_like!(ltrim([x], [chars])) })
#         }


@enforce_types
def ltrim(x: Expr, chars: str):
    return F.ltrim(x.to_pyspark_expr(), chars)


#         // replace(<str>,<regex>,<replacement>)                	 Substitutes the replacement string for every occurrence of the regular expression in the string.
#         "replace" => {
#             function_transform!(replace [args] (input, regex: String, replacement: String) {
#                 column_like!(regexp_replace([input.clone()], [py_lit(regex)], [py_lit(replacement)]))
#             })
#         }


@enforce_types
def replace(input: Expr, regex: str, replacement: str):
    return F.regexp_replace(input.to_pyspark_expr(), regex, replacement)


#         // rtrim(<str>,<trim_chars>)                           	 Removes the trim characters from the right side of the string.
#         "rtrim" => {
#             function_transform!(rtrim [args] (x, chars) { column_like!(rtrim([x], [chars])) })
#         }


@enforce_types
def rtrim(x: Expr, chars: str):
    return F.rtrim(x.to_pyspark_expr(), chars)


#         // spath(<value>,<path>)                               	 Extracts information from the structured data formats XML and JSON.
#         "spath" => {
#             bail!("UNIMPLEMENTED: Unsupported function: {}", name)
#         }


@enforce_types
def spath(value: Expr, path: str):
    raise NotImplementedError("Unsupported function: spath")


#         // substr(<str>,<start>,<length>)                      	 Returns a substring of a string, beginning at the start index. The length of the substring specifies the number of character to return.
#         "substr" => function_transform!(substr [args] (column, start: i64, length: i64) {
#             column_like!(substring([column], [py_lit(start)], [py_lit(length)]))
#         }),


@enforce_types
def substr(column: Expr, start: int, length: int):
    return F.substring(column.to_pyspark_expr(), start, length)


#         // trim(<str>,<trim_chars>)                            	 Trim characters from both sides of a string.
#         "trim" => function_transform!(trim [args] (x, chars) { column_like!(trim([x], [chars])) }),


@enforce_types
def trim(x: Expr, chars: str):
    return F.trim(x.to_pyspark_expr(), chars)


#         // upper(<str>)                                        	 Returns the string in uppercase.
#         "upper" => function_transform!(upper [args] (x) { column_like!(upper([x])) }),


@enforce_types
def upper(x: Expr):
    return F.upper(x.to_pyspark_expr())


#         // urldecode(<url>)                                    	 Replaces URL escaped characters with the original characters.
#         "urldecode" => function_transform!(urldecode [args] (x) { column_like!(url_decode([x])) }),


@enforce_types
def urldecode(x: Expr):
    return F.url_decode(x.to_pyspark_expr())


#
#         // Trigonometry and Hyperbolic functions
#         // acos(X)                                             	 Computes the arc cosine of X.
#         "acos" => function_transform!(acos [args] (x) { column_like!(acos([x])) }),


@enforce_types
def acos(x: Expr) -> Expr:
    return F.acos(x.to_pyspark_expr())


#         // acosh(X)                                            	 Computes the arc hyperbolic cosine of X.
#         "acosh" => function_transform!(acosh [args] (x) { column_like!(acosh([x])) }),


@enforce_types
def acosh(x: Expr) -> Expr:
    return F.acosh(x.to_pyspark_expr())


#         // asin(X)                                             	 Computes the arc sine of X.
#         "asin" => function_transform!(asin [args] (x) { column_like!(asin([x])) }),


@enforce_types
def asin(x: Expr) -> Expr:
    return F.asin(x.to_pyspark_expr())


#         // asinh(X)                                            	 Computes the arc hyperbolic sine of X.
#         "asinh" => function_transform!(asinh [args] (x) { column_like!(asinh([x])) }),


@enforce_types
def asinh(x: Expr) -> Expr:
    return F.asinh(x.to_pyspark_expr())


#         // atan(X)                                             	 Computes the arc tangent of X.
#         "atan" => function_transform!(atan [args] (x) { column_like!(atan([x])) }),


@enforce_types
def atan(x: Expr) -> Expr:
    return F.atan(x.to_pyspark_expr())


#         // atan2(X,Y)                                          	 Computes the arc tangent of X,Y.
#         "atan2" => function_transform!(atan2 [args] (x, y) { column_like!(atan2([x], [y])) }),


@enforce_types
def atan2(x: Expr, y: Expr) -> Expr:
    return F.atan2(x.to_pyspark_expr(), y.to_pyspark_expr())


#         // atanh(X)                                            	 Computes the arc hyperbolic tangent of X.
#         "atanh" => function_transform!(atanh [args] (x) { column_like!(atanh([x])) }),


@enforce_types
def atanh(x: Expr) -> Expr:
    return F.atanh(x.to_pyspark_expr())


#         // cos(X)                                              	 Computes the cosine of an angle of X radians.
#         "cos" => function_transform!(cos [args] (x) { column_like!(cos([x])) }),


@enforce_types
def cos(x: Expr) -> Expr:
    return F.cos(x.to_pyspark_expr())


#         // cosh(X)                                             	 Computes the hyperbolic cosine of X radians.
#         "cosh" => function_transform!(cosh [args] (x) { column_like!(cosh([x])) }),


@enforce_types
def cosh(x: Expr) -> Expr:
    return F.cosh(x.to_pyspark_expr())


#         // hypot(X,Y)                                          	 Computes the hypotenuse of a triangle.
#         "hypot" => function_transform!(hypot [args] (x, y) { column_like!(hypot([x], [y])) }),


@enforce_types
def hypot(x: Expr, y: Expr) -> Expr:
    return F.hypot(x.to_pyspark_expr(), y.to_pyspark_expr())


#         // sin(X)                                              	 Computes the sine of X.
#         "sin" => function_transform!(sin [args] (x) { column_like!(sin([x])) }),


@enforce_types
def sin(x: Expr) -> Expr:
    return F.sin(x.to_pyspark_expr())


#         // sinh(X)                                             	 Computes the hyperbolic sine of X.
#         "sinh" => function_transform!(sinh [args] (x) { column_like!(sinh([x])) }),


@enforce_types
def sinh(x: Expr) -> Expr:
    return F.sinh(x.to_pyspark_expr())


#         // tan(X)                                              	 Computes the tangent of X.
#         "tan" => function_transform!(tan [args] (x) { column_like!(tan([x])) }),


@enforce_types
def tan(x: Expr) -> Expr:
    return F.tan(x.to_pyspark_expr())


#         // tanh(X)                                             	 Computes the hyperbolic tangent of X.
#         "tanh" => function_transform!(tanh [args] (x) { column_like!(tanh([x])) }),


@enforce_types
def tanh(x: Expr) -> Expr:
    return F.tanh(x.to_pyspark_expr())
