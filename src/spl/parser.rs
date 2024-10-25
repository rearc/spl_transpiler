use crate::commands::cmd_add_totals::spl::AddTotalsParser;
use crate::commands::cmd_bin::spl::BinParser;
use crate::commands::cmd_collect::spl::CollectParser;
use crate::commands::cmd_convert::spl::ConvertParser;
use crate::commands::cmd_data_model::spl::DataModelParser;
use crate::commands::cmd_dedup::spl::DedupParser;
use crate::commands::cmd_eval::spl::EvalParser;
use crate::commands::cmd_event_stats::spl::EventStatsParser;
use crate::commands::cmd_fields::spl::FieldsParser;
use crate::commands::cmd_fill_null::spl::FillNullParser;
use crate::commands::cmd_format::spl::FormatParser;
use crate::commands::cmd_head::spl::HeadParser;
use crate::commands::cmd_input_lookup::spl::InputLookupParser;
use crate::commands::cmd_join::spl::JoinParser;
use crate::commands::cmd_lookup::spl::LookupParser;
use crate::commands::cmd_make_results::spl::MakeResultsParser;
use crate::commands::cmd_map::spl::MapParser;
use crate::commands::cmd_multi_search::spl::MultiSearchParser;
use crate::commands::cmd_mv_combine::spl::MvCombineParser;
use crate::commands::cmd_mv_expand::spl::MvExpandParser;
use crate::commands::cmd_rare::spl::RareParser;
use crate::commands::cmd_regex::spl::RegexParser;
use crate::commands::cmd_rename::spl::RenameParser;
use crate::commands::cmd_return::spl::ReturnParser;
use crate::commands::cmd_rex::spl::RexParser;
use crate::commands::cmd_s_path::spl::SPathParser;
use crate::commands::cmd_search::spl::SearchParser;
use crate::commands::cmd_sort::spl::SortParser;
use crate::commands::cmd_stats::spl::StatsParser;
use crate::commands::cmd_stream_stats::spl::StreamStatsParser;
use crate::commands::cmd_t_stats::spl::TStatsParser;
use crate::commands::cmd_table::spl::TableParser;
use crate::commands::cmd_tail::spl::TailParser;
use crate::commands::cmd_top::spl::TopParser;
use crate::commands::cmd_where::spl::WhereParser;
use crate::commands::spl::SplCommand;
use crate::spl::operators::{OperatorSymbol, OperatorSymbolTrait};
use crate::spl::{ast, operators};
use nom::bytes::complete::{tag_no_case, take, take_while, take_while1};
use nom::character::complete::{alphanumeric1, anychar, multispace0, multispace1, one_of};
use nom::combinator::{all_consuming, eof, into, map_parser, peek, value, verify};
use nom::error::ParseError;
use nom::multi::{many0, separated_list0, separated_list1};
use nom::sequence::{delimited, preceded, separated_pair, terminated};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{digit1, none_of},
    combinator::{map, opt, recognize},
    multi::many1,
    sequence::{pair, tuple},
    IResult, Parser,
};
use paste::paste;
use std::fmt::Debug;
use std::net::Ipv6Addr;

// https://github.com/rust-bakery/nom/blob/main/doc/nom_recipes.md#wrapper-combinators-that-eat-whitespace-before-and-after-a-parser
pub fn ws<'a, O, E: ParseError<&'a str>, F>(inner: F) -> impl Parser<&'a str, O, E>
where
    F: Parser<&'a str, O, E>,
{
    delimited(multispace0, inner, multispace0)
}

// https://github.com/rust-bakery/nom/blob/main/doc/nom_recipes.md#wrapper-combinators-that-eat-whitespace-before-and-after-a-parser
pub fn ws_left<'a, O, E: ParseError<&'a str>, F>(inner: F) -> impl Parser<&'a str, O, E>
where
    F: Parser<&'a str, O, E>,
{
    preceded(multispace0, inner)
}

// https://github.com/rust-bakery/nom/blob/main/doc/nom_recipes.md#wrapper-combinators-that-eat-whitespace-before-and-after-a-parser
pub fn ws_right<'a, O, E: ParseError<&'a str>, F>(inner: F) -> impl Parser<&'a str, O, E>
where
    F: Parser<&'a str, O, E>,
{
    terminated(inner, multispace0)
}

// https://github.com/rust-bakery/nom/blob/main/doc/nom_recipes.md#wrapper-combinators-that-eat-whitespace-before-and-after-a-parser
pub fn parenthesized<'a, O, E: ParseError<&'a str>, F>(inner: F) -> impl Parser<&'a str, O, E>
where
    F: Parser<&'a str, O, E>,
{
    delimited(ws_right(tag("(")), inner, ws_left(tag(")")))
}

pub fn unwrapped<'a, O, InnerE, E: ParseError<&'a str>, F>(inner: F) -> impl Parser<&'a str, O, E>
where
    F: Parser<&'a str, Result<O, InnerE>, E>,
    InnerE: Debug,
{
    map(
        verify(inner, |res| res.is_ok()),
        |res| res.unwrap(), // This is safe at this point since we checked is_ok
    )
}

#[allow(dead_code)]
pub fn unwrapped_option<'a, O, E: ParseError<&'a str>, F>(inner: F) -> impl Parser<&'a str, O, E>
where
    F: Parser<&'a str, Option<O>, E>,
{
    map(
        verify(inner, |res| res.is_some()),
        |res| res.unwrap(), // This is safe at this point since we checked is_ok
    )
}

macro_rules! make_specialized_separated_list {
    ($name:ident, $separator:expr) => { paste! {
        #[allow(dead_code)]
        pub fn [<$name 1>]<'a, O, E: ParseError<&'a str>, F>(inner: F) -> impl Parser<&'a str, Vec<O>, E>
        where
            F: Parser<&'a str, O, E>,
        {
            separated_list1($separator, inner)
        }

        #[allow(dead_code)]
        pub fn [<$name 0>]<'a, O, E: ParseError<&'a str>, F>(inner: F) -> impl Parser<&'a str, Vec<O>, E>
        where
            F: Parser<&'a str, O, E>,
        {
            separated_list0($separator, inner)
        }
    }};
}

make_specialized_separated_list!(comma_separated_list, ws(tag(",")));
make_specialized_separated_list!(space_separated_list, multispace1);
make_specialized_separated_list!(
    comma_or_space_separated_list,
    alt((ws(tag(",")), multispace1))
);

macro_rules! unit_normalizer_alt_list {
    ($name: literal) => { tag_no_case($name) };
    ($($remaining: literal),+) => {
        ($( unit_normalizer_alt_list!($remaining) ),+)
    }
}

macro_rules! unit_normalizer {
    ($normalized_name: ident, $($vals: literal),+) => {
        fn $normalized_name(input: &str) -> IResult<&str, &str> {
            map(
                alt(
                    unit_normalizer_alt_list!($($vals),+)
                ),
                |_| stringify!($normalized_name),
            )(input)
        }
    };
}

//   private def letter[_: P] = P( lowercase | uppercase )
//   private def lowercase[_: P] = P( CharIn("a-z") )
//   private def uppercase[_: P] = P( CharIn("A-Z") )
//   private def digit[_: P] = CharIn("0-9")
//
//   def W[_: P](s: String): P[Unit] = IgnoreCase(s)
//   private[spl] def bool[_: P] =
//     ("true" | "t").map(_ => Bool(true)) |
//       ("false" | "f").map(_ => Bool(false))
//
//   // TODO: add interval parsing for: us | ms | cs | ds
//   def seconds[_: P]: P[String] = ("seconds" | "second" | "secs" | "sec" | "s").map(_ => "seconds")
unit_normalizer!(seconds, "seconds", "second", "secs", "sec", "s");

//   def minutes[_: P]: P[String] = ("minutes" | "minute" | "mins" | "min" | "m").map(_ => "minutes")
unit_normalizer!(minutes, "minutes", "minute", "mins", "min", "m");

//   def hours[_: P]: P[String] = ("hours" | "hour" | "hrs" | "hr" | "h").map(_ => "hours")
unit_normalizer!(hours, "hours", "hour", "hrs", "hr", "h");

//   def days[_: P]: P[String] = ("days" | "day" | "d").map(_ => "days")
unit_normalizer!(days, "days", "day", "d");

//   def weeks[_: P]: P[String] = ("weeks" | "week" | "w7" | "w0" | "w").map(_ => "weeks")
unit_normalizer!(weeks, "weeks", "week", "w7", "w0", "w");

//   def months[_: P]: P[String] = ("months" | "month" | "mon").map(_ => "months")
unit_normalizer!(months, "months", "month", "mon");

//   def timeUnit[_: P]: P[String] = months|days|hours|minutes|weeks|seconds
pub fn time_unit(input: &str) -> IResult<&str, &str> {
    alt((months, days, hours, minutes, weeks, seconds))(input)
}

//   def timeSpan[_: P]: P[TimeSpan] = int ~~ timeUnit map {
//     case (IntValue(v), interval) => TimeSpan(v, interval)
//   }
pub fn time_span(input: &str) -> IResult<&str, ast::TimeSpan> {
    map(pair(int, time_unit), |(num, unit)| ast::TimeSpan {
        value: num.0,
        scale: unit.to_string(),
    })(input)
}

//   def timeSpanOne[_: P]: P[TimeSpan] = "-".!.? ~~ timeUnit map {
//     case (Some("-"), interval) => TimeSpan(-1, interval)
//     case (None, interval) => TimeSpan(1, interval)
//     case a: Any => throw new IllegalArgumentException(s"timeSpan $a")
//   }
pub fn time_span_one(input: &str) -> IResult<&str, ast::TimeSpan> {
    map(pair(opt(tag("-")), time_unit), |(sign, unit)| {
        ast::TimeSpan {
            value: if sign.is_some() { -1 } else { 1 },
            scale: unit.to_string(),
        }
    })(input)
}

//   def relativeTime[_: P]: P[SnapTime] = (
//       (timeSpan|timeSpanOne).? ~~ "@" ~~ timeUnit ~~ timeSpan.?).map(SnapTime.tupled)
fn _relative_time_fmt(input: &str) -> IResult<&str, ast::SnapTime> {
    map(
        tuple((
            opt(alt((time_span, time_span_one))),
            tag("@"),
            time_unit,
            opt(time_span),
        )),
        |(span, _, rhs_unit, rhs_span)| ast::SnapTime {
            span,
            snap: rhs_unit.into(),
            snap_offset: rhs_span,
        },
    )(input)
}

pub fn relative_time(input: &str) -> IResult<&str, ast::SnapTime> {
    alt((
        _relative_time_fmt,
        delimited(tag("\""), _relative_time_fmt, tag("\"")),
    ))(input)
}

//   def token[_: P]: P[String] = ("_"|"*"|letter|digit).repX(1).!
/*
after	and,
AND

apply	as,
AS

asc,
ASC

before	between,
BETWEEN

bin	branch	by,
BY

dedup	desc,
DESC

distinct,
DISTINCT

eval	eventstats
exists,
EXISTS

export	false	fit	from,
FROM

function	group,
GROUP

groupby,
GROUPBY

having,
HAVING

head
histperc	import	in,
IN

inner,
INNER

into
is,
IS

join,
JOIN

left,
LEFT

like,
LIKE

limit,
LIMIT

lookup	not,
NOT

null,
NULL

offset,
OFFSET

on,
ON

onchange	or,
OR

order,
ORDER

orderby,
ORDERBY

outer,
OUTER

OUTPUT	OUTPUTNEW	rename	reset	return
rex	search	select,
SELECT

sort	stats
streamstats	thru,
through

timechart	timewrap	true
type	union,
UNION

where,
WHERE

while	xor,
XOR
 */
fn _is_not_reserved_word(input: &str) -> bool {
    !matches!(
        input.to_ascii_uppercase().as_str(),
        "AFTER"
            | "AND"
            | "APPLY"
            | "AS"
            | "ASC"
            | "BEFORE"
            | "BETWEEN"
            | "BIN"
            | "BRANCH"
            | "BY"
            | "DEDUP"
            | "DISTINCT"
            | "EVENTSTATS"
            | "EXISTS"
            | "EXPORT"
            | "FIT"
            | "FROM"
            | "FUNCTION"
            | "GROUP"
            | "GROUPBY"
            | "HAVING"
            | "HEAD"
            | "HISTPERC"
            | "IMPORT"
            | "IN"
            | "INNER"
            | "INTO"
            | "IS"
            | "JOIN"
            | "LIKE"
            | "LIMIT"
            | "LOOKUP"
            | "NOT"
            | "OFFSET"
            | "ON"
            | "ONCHANGE"
            | "OR"
            | "ORDER"
            | "ORDERBY"
            | "OUTER"
            | "OUTPUTNEW"
            | "RENAME"
            | "RESET"
            | "RETURN"
            | "REX"
            | "SEARCH"
            | "SELECT"
            | "SORT"
            | "STATS"
            | "STREAMSTATS"
            | "THRU"
            | "THROUGH"
            | "TIMECHART"
            | "TIMEWRAP"
            | "UNION"
            | "WHERE"
            | "WHILE"
            | "XOR" // | "TRUE" | "FALSE" | "TYPE" | "LEFT" | "NULL" | "DESC" | "EVAL"
    )
}

pub fn token(input: &str) -> IResult<&str, &str> {
    verify(
        recognize(many1(alt((
            recognize(one_of("_*:/-%{}@")),
            alphanumeric1,
            recognize(pair(tag("\\"), take(1usize))),
        )))),
        _is_not_reserved_word,
    )(input)
}

pub fn token_with_extras(input: &str) -> IResult<&str, &str> {
    verify(
        recognize(many1(alt((
            recognize(one_of("_*:/-%{}@\\$.")),
            alphanumeric1,
            recognize(pair(tag("\\"), take(1usize))),
        )))),
        _is_not_reserved_word,
    )(input)
}

// Boolean parser
pub fn bool_(input: &str) -> IResult<&str, ast::BoolValue> {
    map(
        terminated(
            alt((
                map(tag_no_case("true"), |_| true),
                map(tag_no_case("t"), |_| true),
                map(tag_no_case("false"), |_| false),
                map(tag_no_case("f"), |_| false),
            )),
            peek(alt((eof, multispace1, recognize(one_of("().:"))))),
        ),
        ast::BoolValue::from,
    )(input)
}

//   def doubleQuoted[_: P]: P[String] = P( "\"" ~ (CharsWhile(!"\"".contains(_)) | "\\" ~~ AnyChar | !"\"").rep.! ~ "\"" )
fn _double_quoted_content(input: &str) -> IResult<&str, &str> {
    ws(recognize(many0(alt((
        recognize(pair(tag(r#"\"#), anychar)),
        recognize(none_of(r#""\"#)),
    )))))
    .parse(input)
}

pub fn double_quoted(input: &str) -> IResult<&str, &str> {
    alt((
        delimited(
            tag("\""),
            recognize(tuple((tag("\""), _double_quoted_content, tag("\"")))),
            tag("\""),
        ),
        delimited(tag("\""), _double_quoted_content, tag("\"")),
    ))(input)
}

pub fn single_quoted(input: &str) -> IResult<&str, &str> {
    delimited(
        tag("\'"),
        ws(recognize(many0(alt((
            recognize(pair(tag(r#"\"#), anychar)),
            recognize(none_of(r#"'\"#)),
        ))))),
        tag("\'"),
    )(input)
}

//   def wildcard[_: P]: P[Wildcard] = (
//       doubleQuoted.filter(_.contains("*")) | token.filter(_.contains("*"))) map Wildcard
pub fn wildcard(input: &str) -> IResult<&str, ast::Wildcard> {
    map(
        verify(alt((double_quoted, token_with_extras)), |v: &str| {
            v.contains("*")
        }),
        |v| ast::Wildcard(v.into()),
    )(input)
}

//   def doubleQuotedAlt[_: P]: P[String] = P(
//     "\"" ~ (CharsWhile(!"\"\\".contains(_: Char)) | "\\\"").rep.! ~ "\"")
pub fn double_quoted_alt(input: &str) -> IResult<&str, &str> {
    delimited(
        tag(r#"""#),
        ws(recognize(many0(alt((
            tag(r#"\""#),
            tag(r#"\\"#),
            recognize(none_of(r#""\"#)),
        ))))),
        tag(r#"""#),
    )(input)
}

//   def strValue[_: P]: P[StrValue] = doubleQuoted map StrValue
pub fn str_value(input: &str) -> IResult<&str, ast::StrValue> {
    map(double_quoted, ast::StrValue::from)(input)
}

pub fn _token_not_t_f(input: &str) -> IResult<&str, &str> {
    verify(token, |v: &str| v != "t" && v != "f")(input)
}

//   def field[_: P]: P[Field] = token.filter(!Seq("t", "f").contains(_)) map Field
fn _field_fmt(input: &str) -> IResult<&str, ast::Field> {
    map(
        recognize(separated_list1(
            tag("."),
            alt((_token_not_t_f, double_quoted)),
        )),
        |v: &str| ast::Field(v.into()),
    )(input)
}

pub fn field(input: &str) -> IResult<&str, ast::Field> {
    alt((_field_fmt, into(single_quoted)))(input)
}

pub fn string(input: &str) -> IResult<&str, &str> {
    alt((
        double_quoted,
        recognize(many1(alt((token, tag("."), tag(":"))))),
    ))(input)
}

//   def variable[_: P]: P[Variable] =
//     "$" ~~ token.filter(!Seq("t", "f").contains(_)) ~~ "$" map Variable
pub fn variable(input: &str) -> IResult<&str, ast::Variable> {
    map(delimited(tag("$"), _token_not_t_f, tag("$")), |name| {
        ast::Variable(name.into())
    })(input)
}

//   def byte[_: P]: P[String] = digit.rep(min = 1, max = 3).!
pub fn byte(input: &str) -> IResult<&str, &str> {
    recognize(verify(digit1, |s: &str| s.len() <= 3))(input)
}

//   def cidr[_: P]: P[IPv4CIDR] = (byte.rep(sep = ".", exactly = 4) ~ "/" ~ byte).! map IPv4CIDR
pub fn ipv4_addr(input: &str) -> IResult<&str, &str> {
    recognize(tuple((
        byte,
        tag("."),
        byte,
        tag("."),
        byte,
        tag("."),
        byte,
    )))(input)
}

pub fn ipv4_cidr(input: &str) -> IResult<&str, ast::IPv4CIDR> {
    map(
        recognize(separated_pair(ipv4_addr, ws(tag("/")), byte)),
        |v: &str| ast::IPv4CIDR(v.into()),
    )(input)
}

pub fn ipv6_addr(input: &str) -> IResult<&str, &str> {
    verify(
        recognize(separated_list1(
            tag(":"),
            opt(unwrapped(map(digit1, str::parse::<u16>))),
        )),
        // Check that Rust's built-in IPv6 parser recognizes it
        |v: &str| v.parse::<Ipv6Addr>().is_ok(),
    )(input)
}

//   // syntax: -?/d+(?!/.)
//   private[spl] def int[_: P]: P[IntValue] = ("+" | "-").?.! ~~ digit.rep(1).! map {
//     case (sign, i) => IntValue(if (sign.equals("-")) -1 * i.toInt else i.toInt)
//   }
pub fn int(input: &str) -> IResult<&str, ast::IntValue> {
    map(
        pair(
            opt(alt((tag("+"), tag("-")))),
            unwrapped(map(digit1, str::parse::<i64>)),
        ),
        |(sign, numeric_value)| {
            let sign_multiplier = if sign == Some("-") { -1 } else { 1 };
            ast::IntValue(sign_multiplier * numeric_value)
        },
    )(input)
}

//   private[spl] def double[_: P]: P[DoubleValue] =
//     ("+" | "-").?.! ~~ digit.rep(1).! ~~ "." ~~ digit.rep(1).! map {
//       case (sign, i, j) => DoubleValue(
//         if (sign.equals("-")) -1 * (i + "." + j).toDouble else (i + "." + j).toDouble
//       )
//   }
pub fn double(input: &str) -> IResult<&str, ast::DoubleValue> {
    map(
        pair(
            opt(alt((tag("+"), tag("-")))),
            separated_pair(digit1, tag("."), digit1),
        ),
        |(sign, (whole_part, decimal_part))| {
            let sign_multiplier = if sign == Some("-") { -1 } else { 1 };
            let raw_value = format!("{}.{}", whole_part, decimal_part).parse::<f64>();
            ast::DoubleValue(sign_multiplier as f64 * raw_value.unwrap())
        },
    )(input)
}

//   def constant[_: P]: P[Constant] = cidr | wildcard | strValue | variable |
//       relativeTime | timeSpan | double | int | field | bool
pub fn literal(input: &str) -> IResult<&str, ast::Constant> {
    let (remaining, content) =
        recognize(alt((double_quoted, single_quoted, token_with_extras)))(input)?;
    let (_, parsed_value) = alt((
        map(all_consuming(ipv4_cidr), ast::Constant::IPv4CIDR),
        map(all_consuming(ipv4_addr), |s| ast::Constant::Str(s.into())),
        map(all_consuming(ipv6_addr), |s| ast::Constant::Str(s.into())),
        map(all_consuming(wildcard), ast::Constant::Wildcard),
        map(all_consuming(str_value), ast::Constant::Str),
        map(all_consuming(variable), ast::Constant::Variable),
        map(all_consuming(relative_time), ast::Constant::SnapTime),
        map(all_consuming(time_span), ast::Constant::TimeSpan),
        map(all_consuming(double), ast::Constant::Double),
        map(all_consuming(int), ast::Constant::Int),
        map(all_consuming(bool_), ast::Constant::Bool),
        map(all_consuming(string), |s| ast::Constant::Str(s.into())),
    ))(content)?;
    Ok((remaining, parsed_value))
}

pub fn constant(input: &str) -> IResult<&str, ast::Constant> {
    let (remaining, content) =
        recognize(alt((double_quoted, single_quoted, token_with_extras)))(input)?;
    let (_, parsed_value) = alt((
        into(all_consuming(ipv4_cidr)),
        map(all_consuming(ipv4_addr), |s| ast::Constant::Str(s.into())),
        map(all_consuming(ipv6_addr), |s| ast::Constant::Str(s.into())),
        into(all_consuming(relative_time)),
        into(all_consuming(time_span)),
        into(all_consuming(wildcard)),
        into(all_consuming(str_value)),
        into(all_consuming(variable)),
        into(all_consuming(double)),
        into(all_consuming(int)),
        into(all_consuming(field)),
        into(all_consuming(bool_)),
    ))(content)?;
    Ok((remaining, parsed_value))
}

//   def fieldAndValue[_: P]: P[FV] = (
//       token ~ "=" ~ (doubleQuoted|token)) map { case (k, v) => FV(k, v) }
pub fn field_and_value(input: &str) -> IResult<&str, ast::FV> {
    map(
        separated_pair(token, ws(tag("=")), alt((double_quoted, token))),
        |(k, v)| ast::FV {
            field: k.into(),
            value: v.into(),
        },
    )(input)
}

//   def fieldAndConstant[_: P]: P[FC] = (token ~ "=" ~ constant) map { case (k, v) => FC(k, v) }
pub fn field_and_constant(input: &str) -> IResult<&str, ast::FC> {
    map(separated_pair(token, ws(tag("=")), constant), |(k, v)| {
        ast::FC {
            field: k.into(),
            value: v,
        }
    })(input)
}

//   def quotedSearch[_: P]: P[Pipeline] = ("search" ~ "=" ~ doubleQuotedAlt) map { subSearch => {
//     val unescapedSearch = StringContext treatEscapes subSearch
//     parse(unescapedSearch, pipeline(_)) match {
//       case Parsed.Success(value, _) => value
//       case f: Parsed.Failure =>
//         // scalastyle:off throwerror
//         throw new IllegalArgumentException(f.msg)
//         // scalastyle:on throwerror
//     }
//   }}
pub fn quoted_search(input: &str) -> IResult<&str, ast::Pipeline> {
    preceded(
        pair(ws(tag_no_case("search")), ws(tag("="))),
        map_parser(double_quoted_alt, |s: &str| {
            let s = s
                .to_string()
                .replace(r#"\""#, r#"""#)
                .replace(r#"\\"#, r#"\"#);
            let result = all_consuming(ws(pipeline))(s.as_str());
            match result {
                Ok(("", res)) => Ok(("", res)),
                Err(nom::Err::Error(e)) => Err(nom::Err::Error(nom::error::Error::new(
                    "Failed to parse quoted search",
                    e.code,
                ))),
                _ => panic!("`all_consuming` returning remaining text or unexpected error type"),
            }
        }),
    )(input)
}

//   def commandOptions[_: P]: P[CommandOptions] = fieldAndConstant.rep map CommandOptions
pub fn command_options(input: &str) -> IResult<&str, ast::CommandOptions> {
    map(many0(ws(field_and_constant)), |options| {
        ast::CommandOptions { options }
    })(input)
}

//   def fieldList[_: P]: P[Seq[Field]] = field.rep(sep = ",")
pub fn field_list0(input: &str) -> IResult<&str, Vec<ast::Field>> {
    comma_or_space_separated_list0(field).parse(input)
}

pub fn field_list1(input: &str) -> IResult<&str, Vec<ast::Field>> {
    comma_or_space_separated_list1(field).parse(input)
}

#[allow(dead_code)]
//   def filename[_: P]: P[String] = term
pub fn filename(input: &str) -> IResult<&str, &str> {
    term(input)
}

#[allow(dead_code)]
//   def term[_: P]: P[String] = CharsWhile(!" ".contains(_)).!
pub fn term(input: &str) -> IResult<&str, &str> {
    recognize(take_while1(|c: char| c != ' '))(input)
}

//   private def ALL[_: P]: P[OperatorSymbol] = (Or.P | And.P | LessThan.P | GreaterThan.P
//     | GreaterEquals.P | LessEquals.P | Equals.P | NotEquals.P | InList.P | Add.P | Subtract.P
//     | Multiply.P | Divide.P | Concatenate.P)
/// Matches any operator that compares two expressions
pub fn comparison_operator(input: &str) -> IResult<&str, OperatorSymbol> {
    alt((
        map(operators::LessEquals::pattern, |_| {
            OperatorSymbol::LessEquals(operators::LessEquals {})
        }),
        map(operators::LessThan::pattern, |_| {
            OperatorSymbol::LessThan(operators::LessThan {})
        }),
        map(operators::GreaterEquals::pattern, |_| {
            OperatorSymbol::GreaterEquals(operators::GreaterEquals {})
        }),
        map(operators::GreaterThan::pattern, |_| {
            OperatorSymbol::GreaterThan(operators::GreaterThan {})
        }),
        map(operators::StrictlyEquals::pattern, |_| {
            OperatorSymbol::StrictlyEquals(operators::StrictlyEquals {})
        }),
        map(operators::Equals::pattern, |_| {
            OperatorSymbol::Equals(operators::Equals {})
        }),
        map(operators::NotEquals::pattern, |_| {
            OperatorSymbol::NotEquals(operators::NotEquals {})
        }),
    ))(input)
}

/// Matches any operator that combines two expressions into a logical/boolean result
// pub fn logical_operator(input: &str) -> IResult<&str, OperatorSymbol> {
//     alt((
//         comparison_operator,
//         map(operators::Or::pattern, |_| {
//             OperatorSymbol::Or(operators::Or {})
//         }),
//         map(operators::And::pattern, |_| {
//             OperatorSymbol::And(operators::And {})
//         }),
//         map(operators::InList::pattern, |_| {
//             OperatorSymbol::InList(operators::InList {})
//         }),
//     ))(input)
// }

/// Matches any operator that combines two expressions into a new expression
// pub fn operator(input: &str) -> IResult<&str, OperatorSymbol> {
//     alt((
//         logical_operator,
//         map(operators::Add::pattern, |_| {
//             OperatorSymbol::Add(operators::Add {})
//         }),
//         map(operators::Subtract::pattern, |_| {
//             OperatorSymbol::Subtract(operators::Subtract {})
//         }),
//         map(operators::Multiply::pattern, |_| {
//             OperatorSymbol::Multiply(operators::Multiply {})
//         }),
//         map(operators::Divide::pattern, |_| {
//             OperatorSymbol::Divide(operators::Divide {})
//         }),
//         map(operators::Concatenate::pattern, |_| {
//             OperatorSymbol::Concatenate(operators::Concatenate {})
//         }),
//     ))(input)
// }

//   private def binaryOf[_: P](a: => P[Expr], b: => P[OperatorSymbol]): P[Expr] =
//     (a ~ (b ~ a).rep).map {
//       case (expr, tuples) => climb(expr, tuples)
//     }

//   private def unaryOf[_: P](expr: => P[Expr]): P[Unary] = UnaryNot.P ~ expr map Unary.tupled

//
//   private def climb(left: Expr, rights: Seq[(OperatorSymbol, Expr)], prec: Int = 100): Expr =
//     rights.headOption match {
//       case None => left
//       case Some((sym, next)) =>
//         if (sym.precedence < prec) left match {
//           case Binary(first, prevSymbol, right) =>
//             Binary(first, prevSymbol,
//               climb(Binary(right, sym, next),
//                 rights.tail, sym.precedence + 1))
//           case _ => climb(Binary(left, sym, next),
//             rights.tail, sym.precedence + 1)
//         } else Binary(left, sym,
//           climb(next, rights.tail, sym.precedence + 1))
//     }
// pub fn climb(left: ast::Expr, rights: Vec<(OperatorSymbol, ast::Expr)>, prec: i32) -> ast::Expr {
//     if rights.is_empty() {
//         left
//     } else {
//         let (sym, next): (OperatorSymbol, ast::Expr) = rights[0].clone();
//         let remainder: Vec<_> = rights.into_iter().skip(1).collect();
//         let symbol = sym.symbol_string().into();
//         let precedence = sym.precedence();
//         if precedence < prec {
//             match left {
//                 ast::Expr::Binary(ast::Binary {
//                                       left: first,
//                                       symbol: prev_symbol,
//                                       right,
//                                   }) => ast::Expr::Binary(ast::Binary {
//                     left: first,
//                     symbol: prev_symbol,
//                     right: Box::new(climb(
//                         ast::Expr::Binary(ast::Binary {
//                             left: right,
//                             symbol,
//                             right: Box::new(next),
//                         }),
//                         remainder,
//                         precedence + 1,
//                     )),
//                 }),
//                 _ => climb(
//                     ast::Expr::Binary(ast::Binary {
//                         left: Box::new(left.unaliased()),
//                         symbol,
//                         right: Box::new(next),
//                     }),
//                     remainder,
//                     precedence + 1,
//                 ),
//             }
//         } else {
//             ast::Expr::Binary(ast::Binary {
//                 left: Box::new(left.unaliased()),
//                 symbol,
//                 right: Box::new(climb(next, remainder, precedence + 1)),
//             })
//         }
//     }
// }

//   def fieldIn[_: P]: P[FieldIn] =
//     token ~ "IN" ~ "(" ~ constant.rep(sep = ",".?) ~ ")" map FieldIn.tupled
pub fn field_in(input: &str) -> IResult<&str, ast::FieldIn> {
    map(
        separated_pair(
            field,
            ws(tag_no_case("IN")),
            parenthesized(comma_or_space_separated_list0(constant)),
        ),
        |(field, constants)| ast::FieldIn {
            field: field.0,
            exprs: constants
                .into_iter()
                .map(|c| ast::Expr::Leaf(ast::LeafExpr::Constant(c)))
                .collect(),
        },
    )(input)
}

//   def call[_: P]: P[Call] = (token ~~ "(" ~~ expr.rep(sep = ",") ~~ ")").map(Call.tupled)
pub fn call(input: &str) -> IResult<&str, ast::Call> {
    map(
        pair(token, parenthesized(comma_separated_list0(expr))),
        |(token, exprs)| ast::Call {
            name: token.into(),
            args: exprs,
        },
    )(input)
}

//   def termCall[_: P]: P[Call] = (W("TERM") ~ "(" ~ CharsWhile(!")".contains(_)).! ~ ")").map(
//     term => Call("TERM", Seq(Field(term)))
//   )
pub fn term_call(input: &str) -> IResult<&str, ast::Call> {
    map(
        preceded(tag_no_case("TERM"), parenthesized(take_while(|c| c != ')'))),
        |term| ast::Call {
            name: "TERM".into(),
            args: vec![ast::Expr::Leaf(ast::LeafExpr::Constant(
                ast::Constant::Field(ast::Field(term.into())),
            ))],
        },
    )(input)
}

//   def argu[_: P]: P[Expr] = termCall | call | constant
// pub fn argu(input: &str) -> IResult<&str, ast::Expr> {
//     alt((
//         map(term_call, ast::Expr::Call),
//         map(call, ast::Expr::Call),
//         map(constant, |v| ast::Expr::Leaf(ast::LeafExpr::Constant(v))),
//     ))(input)
// }
//
// //   def parens[_: P]: P[Expr] = "(" ~ expr ~ ")"
// pub fn parens(input: &str) -> IResult<&str, ast::Expr> {
//     parenthesized(expr).parse(input)
// }
//
// //   def primary[_: P]: P[Expr] = unaryOf(expr) | fieldIn | parens | argu
// pub fn primary(input: &str) -> IResult<&str, ast::Expr> {
//     alt((
//         map(
//             preceded(pair(operators::UnaryNot::pattern, multispace1), expr),
//             |e| {
//                 ast::Expr::Unary(ast::Unary {
//                     symbol: operators::UnaryNot::SYMBOL.into(),
//                     right: Box::new(e),
//                 })
//             },
//         ),
//         into(field_in),
//         parens,
//         argu,
//     ))(input)
// }
//
// //   def expr[_: P]: P[Expr] = binaryOf(primary, ALL)
// pub fn expr(input: &str) -> IResult<&str, ast::Expr> {
//     map(
//         pair(primary, many0(pair(ws(operator), primary))),
//         |(expr, tuples)| climb(expr, tuples, 100),
//     )(input)
// }

/*
   Operator precedence for search expressions, lower numbers mean higher precedence
   precedence 2: UnaryNot
   precedence 3: Multiply, Divide
   precedence 4: Add, Subtract
   precedence 5: Concatenate
   precedence 7: InList, Equals, StrictlyEquals, NotEquals, LessThan, GreaterThan, GreaterEquals, LessEquals
   precedence 8: And
   precedence 9: Or
*/

pub fn expr_base(input: &str) -> IResult<&str, ast::Expr> {
    alt((
        parenthesized(expr9_with_whitespace),
        map(term_call, ast::Expr::Call),
        map(call, ast::Expr::Call),
        map(constant, |v| ast::Expr::Leaf(ast::LeafExpr::Constant(v))),
        map(sub_search, ast::Expr::SubSearch),
    ))(input)
}

pub fn expr2(input: &str) -> IResult<&str, ast::Expr> {
    alt((expr_base,))(input)
}

pub fn expr3(input: &str) -> IResult<&str, ast::Expr> {
    alt((map(
        pair(
            expr2,
            opt(pair(
                ws(alt((
                    operators::Multiply::pattern,
                    operators::Divide::pattern,
                ))),
                expr3,
            )),
        ),
        |(left, maybe_op_right)| match maybe_op_right {
            None => left,
            Some((op, right)) => ast::Binary::new(left, op, right).into(),
        },
    ),))(input)
}

pub fn expr4(input: &str) -> IResult<&str, ast::Expr> {
    alt((map(
        pair(
            expr3,
            opt(pair(
                ws(alt((operators::Add::pattern, operators::Subtract::pattern))),
                expr4,
            )),
        ),
        |(left, maybe_op_right)| match maybe_op_right {
            None => left,
            Some((op, right)) => ast::Binary::new(left, op, right).into(),
        },
    ),))(input)
}

pub fn expr5(input: &str) -> IResult<&str, ast::Expr> {
    alt((map(
        pair(
            expr4,
            opt(pair(ws(alt((operators::Concatenate::pattern,))), expr5)),
        ),
        |(left, maybe_op_right)| match maybe_op_right {
            None => left,
            Some((op, right)) => ast::Binary::new(left, op, right).into(),
        },
    ),))(input)
}

pub fn expr6(input: &str) -> IResult<&str, ast::Expr> {
    alt((
        into(field_in),
        map(
            pair(
                expr5,
                opt(pair(
                    ws(alt((
                        operators::StrictlyEquals::pattern,
                        operators::Equals::pattern,
                        operators::NotEquals::pattern,
                        operators::LessEquals::pattern,
                        operators::LessThan::pattern,
                        operators::GreaterEquals::pattern,
                        operators::GreaterThan::pattern,
                    ))),
                    expr6,
                )),
            ),
            |(left, maybe_op_right)| match maybe_op_right {
                None => left,
                Some((op, right)) => ast::Binary::new(left, op, right).into(),
            },
        ),
    ))(input)
}

pub fn expr7(input: &str) -> IResult<&str, ast::Expr> {
    alt((
        map(
            alt((
                separated_pair(operators::UnaryNot::pattern, multispace1, expr7),
                pair(
                    ws(operators::UnaryNot::pattern),
                    parenthesized(expr9_with_whitespace),
                ),
            )),
            |(op, right)| ast::Unary::new(op, right).into(),
        ),
        expr6,
    ))(input)
}

pub fn expr8(input: &str) -> IResult<&str, ast::Expr> {
    alt((map(
        pair(expr7, opt(pair(ws(operators::Or::pattern), expr8))),
        |(left, maybe_op_right)| match maybe_op_right {
            None => left,
            Some((op, right)) => ast::Binary::new(left, op, right).into(),
        },
    ),))(input)
}

pub fn expr9_no_whitespace(input: &str) -> IResult<&str, ast::Expr> {
    map(
        pair(
            expr8,
            opt(pair(ws(operators::And::pattern), expr9_no_whitespace)),
        ),
        |(left, maybe_op_right)| match maybe_op_right {
            None => left,
            Some((op, right)) => ast::Binary::new(left, op, right).into(),
        },
    )(input)
}

pub fn expr9_with_whitespace(input: &str) -> IResult<&str, ast::Expr> {
    map(
        pair(
            expr8,
            opt(pair(
                alt((ws(operators::And::pattern), value("AND", multispace1))),
                expr9_with_whitespace,
            )),
        ),
        |(left, maybe_op_right)| match maybe_op_right {
            None => left,
            Some((op, right)) => ast::Binary::new(left, op, right).into(),
        },
    )(input)
}

pub fn expr(input: &str) -> IResult<&str, ast::Expr> {
    expr9_no_whitespace(input)
}

//
//   // lookup <lookup-dataset> (<lookup-field> [AS <event-field>] )...
//   // [ (OUTPUT | OUTPUTNEW) ( <lookup-destfield> [AS <event-destfield>] )...]
//   def aliasedField[_: P]: P[Alias] = field ~ W("AS") ~ (token|doubleQuoted) map Alias.tupled
pub fn aliased_field(input: &str) -> IResult<&str, ast::AliasedField> {
    map(
        separated_pair(field, ws(tag_no_case("AS")), alt((token, double_quoted))),
        |(field, alias)| ast::AliasedField {
            field,
            alias: alias.into(),
        },
    )(input)
}

//   def aliasedCall[_: P]: P[Alias] = call ~ W("as") ~ token map Alias.tupled
pub fn aliased_call(input: &str) -> IResult<&str, ast::Alias> {
    map(
        separated_pair(call, ws(tag_no_case("as")), token),
        |(expr, name)| ast::Alias {
            expr: Box::new(expr.into()),
            name: name.into(),
        },
    )(input)
}

//   def statsCall[_: P]: P[Seq[Expr with Product with Serializable]] = (aliasedCall | call |
//     token.filter(!_.toLowerCase(Locale.ROOT).equals("by")).map(Call(_))).rep(1, ",".?)
pub fn stats_call(input: &str) -> IResult<&str, Vec<ast::Expr>> {
    comma_or_space_separated_list1(alt((
        into(aliased_call),
        into(call),
        map(
            aliased_field,
            |ast::AliasedField {
                 field: ast::Field(name),
                 alias,
             }| {
                ast::Expr::Alias(ast::Alias {
                    expr: Box::new(ast::Call { name, args: vec![] }.into()),
                    name: alias,
                })
            },
        ),
        map(
            verify(token, |v: &str| v.to_ascii_lowercase() != "by"),
            |v| {
                ast::Call {
                    name: v.into(),
                    args: vec![],
                }
                .into()
            },
        ),
    )))
    .parse(input)
}

//
//   def command[_: P]: P[Command] = (stats
//     | table
//     | where
//     | lookup
//     | collect
//     | convert
//     | eval
//     | head
//     | fields
//     | sort
//     | rex
//     | rename
//     | _regex
//     | join
//     | _return
//     | fillNull
//     | eventStats
//     | streamStats
//     | dedup
//     | inputLookup
//     | format
//     | mvcombine
//     | mvexpand
//     | bin
//     | makeResults
//     | cAddtotals
//     | multiSearch
//     | _map
//     | impliedSearch)
pub fn command(input: &str) -> IResult<&str, ast::Command> {
    alt((
        // `alt` has a hard count limit, so we just break it up into sub-alts
        alt((
            into(AddTotalsParser::parse),
            into(BinParser::parse),
            into(CollectParser::parse),
            into(ConvertParser::parse),
            into(DataModelParser::parse),
            into(DedupParser::parse),
            into(EvalParser::parse),
            into(EventStatsParser::parse),
            into(FieldsParser::parse),
            into(FillNullParser::parse),
            into(FormatParser::parse),
            into(HeadParser::parse),
            into(InputLookupParser::parse),
            into(JoinParser::parse),
            into(LookupParser::parse),
            into(MakeResultsParser::parse),
            into(MapParser::parse),
            into(MultiSearchParser::parse),
            into(MvCombineParser::parse),
            into(MvExpandParser::parse),
        )),
        alt((
            into(RareParser::parse),
            into(RegexParser::parse),
            into(RenameParser::parse),
            into(ReturnParser::parse),
            into(RexParser::parse),
            into(SortParser::parse),
            into(SPathParser::parse),
            into(StatsParser::parse),
            into(StreamStatsParser::parse),
            into(TableParser::parse),
            into(TailParser::parse),
            into(TopParser::parse),
            into(TStatsParser::parse),
            into(WhereParser::parse),
        )),
        into(SearchParser::parse),
    ))(input)
}
//
//   def subSearch[_: P]: P[Pipeline] = "[" ~ (command rep(sep = "|")) ~ "]" map Pipeline
pub fn sub_search(input: &str) -> IResult<&str, ast::Pipeline> {
    map(
        delimited(
            ws(tag("[")),
            preceded(opt(ws(tag("|"))), separated_list0(ws(tag("|")), command)),
            ws(tag("]")),
        ),
        |commands| ast::Pipeline { commands },
    )(input)
}

//   def pipeline[_: P]: P[Pipeline] = (command rep(sep = "|")) ~ End map Pipeline
pub fn pipeline(input: &str) -> IResult<&str, ast::Pipeline> {
    map(
        preceded(
            opt(ws(tag("|"))),
            all_consuming(ws(separated_list0(ws(tag("|")), command))),
        ),
        |commands| ast::Pipeline { commands },
    )(input)
}

// <expression>
// Syntax: <logical-expression> | <time-opts> | <search-modifier> | NOT <logical-expression> | <index-expression> | <comparison-expression> | <logical-expression> [OR] <logical-expression>
// pub fn search_expression(input: &str) -> IResult<&str, ast::Expr> {
//     logical_expression(input)
// }

pub fn combine_all_expressions(exprs: Vec<ast::Expr>, symbol: impl ToString) -> Option<ast::Expr> {
    let mut final_expr = None;

    exprs.into_iter().for_each(|expr| {
        final_expr = match (final_expr.clone(), expr) {
            (None, expr) => Some(expr),
            (Some(final_expr), expr) => Some(ast::Expr::Binary(ast::Binary::new(
                final_expr,
                symbol.to_string(),
                expr,
            ))),
        }
    });

    final_expr
}

// <logical-expression> | <time-opts> | <search-modifier> | NOT <logical-expression> | <index-expression> | <comparison-expression> | <logical-expression> [OR] <logical-expression>
pub fn logical_expression(input: &str) -> IResult<&str, ast::Expr> {
    map(
        separated_list1(ws(tag_no_case("OR")), logical_expression_group),
        |exprs| combine_all_expressions(exprs, operators::Or::SYMBOL).unwrap(),
    )(input)
}

pub fn logical_expression_group(input: &str) -> IResult<&str, ast::Expr> {
    map(
        separated_list1(
            alt((ws(tag_no_case("AND")), multispace1)),
            logical_expression_term,
        ),
        |exprs| combine_all_expressions(exprs, operators::And::SYMBOL).unwrap(),
    )(input)
}

pub fn logical_expression_term(input: &str) -> IResult<&str, ast::Expr> {
    alt((
        parenthesized(logical_expression),
        map(
            preceded(ws(tag_no_case("NOT")), logical_expression_term),
            |expr| ast::Unary::new(operators::UnaryNot::SYMBOL, expr).into(),
        ),
        map(time_opts, |opts| {
            combine_all_expressions(opts, operators::Or::SYMBOL).unwrap()
        }),
        into(call),
        comparison_expression,
        search_modifier,
        // index_expression,
    ))(input)
}

// <comparison-expression>
// Syntax: <field><comparison-operator><value> | <field> IN (<value-list>)
pub fn comparison_expression(input: &str) -> IResult<&str, ast::Expr> {
    alt((
        map(
            tuple((
                field,
                ws(comparison_operator),
                alt((into(call), into(literal))),
            )),
            |(left, symbol, right): (_, _, ast::Expr)| {
                ast::Expr::Binary(ast::Binary::new(left, symbol.symbol_string(), right))
            },
        ),
        into(field_in),
    ))(input)
}

// <index-expression>
// Syntax: "<string>" | <term> | <search-modifier>
#[allow(dead_code)]
pub fn index_expression(input: &str) -> IResult<&str, ast::Expr> {
    alt((
        map(double_quoted, |t| ast::StrValue::from(t).into()),
        map(term, |t| ast::StrValue::from(t).into()),
        search_modifier,
    ))(input)
}

// <search-modifier>
// Syntax: <sourcetype-specifier> | <host-specifier> | <hosttag-specifier> | <source-specifier> | <savedsplunk-specifier> | <eventtype-specifier> | <eventtypetag-specifier> | <splunk_server-specifier>
// <sourcetype-specifier>
// Syntax: sourcetype=<string>
// Description: Search for events from the specified sourcetype field.
// <host-specifier>
// Syntax: host=<string>
// Description: Search for events from the specified host field.
// <hosttag-specifier>
// Syntax: hosttag=<string>
// Description: Search for events that have hosts that are tagged by the string.
// <eventtype-specifier>
// Syntax: eventtype=<string>
// Description: Search for events that match the specified event type.
// <eventtypetag-specifier>
// Syntax: eventtypetag=<string>
// Description: Search for events that would match all eventtypes tagged by the string.
// <savedsplunk-specifier>
// Syntax: savedsearch=<string> | savedsplunk=<string>
// Description: Search for events that would be found by the specified saved search.
// <source-specifier>
// Syntax: source=<string>
// Description: Search for events from the specified source field.
// <splunk_server-specifier>
// Syntax: splunk_server=<string>
// Description: Search for events from a specific server. Use "local" to refer to the search head.
pub fn search_modifier(input: &str) -> IResult<&str, ast::Expr> {
    alt((
        // sourcetype_specifier,
        map(preceded(tag_no_case("sourcetype="), string), |s| {
            ast::SearchModifier::SourceType(s.into()).into()
        }),
        // host_specifier,
        map(preceded(tag_no_case("host="), string), |s| {
            ast::SearchModifier::Host(s.into()).into()
        }),
        // hosttag_specifier,
        map(preceded(tag_no_case("hosttag="), string), |s| {
            ast::SearchModifier::HostTag(s.into()).into()
        }),
        // eventtype_specifier,
        map(preceded(tag_no_case("eventtype="), string), |s| {
            ast::SearchModifier::EventType(s.into()).into()
        }),
        // eventtypetag_specifier,
        map(preceded(tag_no_case("eventtypetag="), string), |s| {
            ast::SearchModifier::EventTypeTag(s.into()).into()
        }),
        // savedsplunk_specifier,
        map(
            preceded(
                alt((tag_no_case("savedsearch="), tag_no_case("savedsplunk="))),
                string,
            ),
            |s| ast::SearchModifier::SavedSplunk(s.into()).into(),
        ),
        // source_specifier,
        map(preceded(tag_no_case("source="), string), |s| {
            ast::SearchModifier::Source(s.into()).into()
        }),
        // splunk_server_specifier,
        map(preceded(tag_no_case("splunk_server="), string), |s| {
            ast::SearchModifier::SplunkServer(s.into()).into()
        }),
    ))(input)
}

// <time-opts>
// Syntax: [<timeformat>] (<time-modifier>)...
pub fn time_opts(input: &str) -> IResult<&str, Vec<ast::Expr>> {
    map(
        pair(time_format, space_separated_list1(time_modifier)),
        |(fmt, mods)| {
            mods.into_iter()
                .map(|time_modifier| {
                    ast::FormattedTimeModifier {
                        format: fmt.into(),
                        time_modifier,
                    }
                    .into()
                })
                .collect()
        },
    )(input)
}

// <timeformat>
// Syntax: timeformat=<string>
// Default: timeformat=%m/%d/%Y:%H:%M:%S.
pub fn time_format(input: &str) -> IResult<&str, &str> {
    map(
        opt(preceded(tag_no_case("timeformat="), double_quoted)),
        |fmt| fmt.unwrap_or("%m/%d/%Y:%H:%M:%S."),
    )(input)
}

// <time-modifier>
// Syntax: starttime=<string> | endtime=<string> | earliest=<time_modifier> | latest=<time_modifier>
pub fn time_modifier(input: &str) -> IResult<&str, ast::TimeModifier> {
    alt((
        map(preceded(tag_no_case("starttime="), double_quoted), |v| {
            ast::TimeModifier::StartTime(v.into())
        }),
        map(preceded(tag_no_case("endtime="), double_quoted), |v| {
            ast::TimeModifier::EndTime(v.into())
        }),
        map(preceded(tag_no_case("earliest="), relative_time), |v| {
            ast::TimeModifier::Earliest(v)
        }),
        map(preceded(tag_no_case("latest="), relative_time), |v| {
            ast::TimeModifier::Latest(v)
        }),
    ))(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::cmd_collect::spl::CollectCommand;
    use crate::commands::cmd_eval::spl::{EvalCommand, EvalParser};
    use crate::commands::cmd_fields::spl::{FieldsCommand, FieldsParser};
    use crate::commands::cmd_search::spl::SearchCommand;
    use crate::commands::cmd_sort::spl::{SortCommand, SortParser};
    use crate::spl::utils::test::*;
    use rstest::rstest;

    #[rstest]
    fn test_time_span() {
        let input = "5minutes";
        let result = time_span(input);
        assert_eq!(
            result,
            Ok((
                "",
                ast::TimeSpan {
                    value: 5,
                    scale: "minutes".to_string()
                }
            ))
        );
    }

    #[rstest]
    fn test_token() {
        let input = "token123";
        let result = token(input);
        assert_eq!(result, Ok(("", "token123")));
    }

    #[rstest]
    fn test_double_quoted() {
        assert_eq!(
            double_quoted("\"double quoted string\" and then some"),
            Ok((" and then some", "double quoted string"))
        );
        assert!(double_quoted("double quoted \"string\"").is_err())
    }

    //     test("false") {
    //   p(bool(_), Bool(false))
    // }
    //
    // test("f") {
    //   p(bool(_), Bool(false))
    // }
    //
    // test("from") {
    //   p(expr(_), Field("from"))
    // }
    //
    // test("true") {
    //   p(bool(_), Bool(true))
    // }
    //
    // test("t") {
    //   p(bool(_), Bool(true))
    // }

    #[rstest]
    fn test_bool() {
        assert_eq!(bool_("false"), Ok(("", false.into())));
        assert_eq!(bool_("f"), Ok(("", false.into())));
        assert_eq!(bool_("true"), Ok(("", true.into())));
        assert_eq!(bool_("t"), Ok(("", true.into())));
    }
    #[rstest]
    fn test_wildcard() {
        assert_eq!(wildcard("*"), Ok(("", ast::Wildcard("*".into()))));
        assert_eq!(wildcard("g*"), Ok(("", ast::Wildcard("g*".into()))));
        assert_eq!(wildcard("foo*"), Ok(("", ast::Wildcard("foo*".into()))));
        assert_eq!(
            wildcard("foo*bar"),
            Ok(("", ast::Wildcard("foo*bar".into())))
        );
        assert_eq!(wildcard("*foo*"), Ok(("", ast::Wildcard("*foo*".into()))));
        assert_eq!(wildcard("\"str*\""), Ok(("", ast::Wildcard("str*".into()))));
        assert!(wildcard("str").is_err());
        assert!(wildcard("foo bar").is_err());
        assert!(wildcard("\"str\"").is_err());
    }

    // test("tree") {
    //   p(expr(_), Field("tree"))
    // }
    //
    // test("left") {
    //   p(field(_), Field("left"))
    // }
    #[rstest]
    fn test_field() {
        assert_eq!(
            expr("junk"),
            Ok((
                "",
                ast::Expr::Leaf(ast::LeafExpr::Constant(ast::Constant::Field(ast::Field(
                    "junk".into()
                ))))
            ))
        );
    }

    // test("foo   = bar") {
    //   p(fieldAndValue(_), FV("foo", "bar"))
    // }
    #[rstest]
    fn test_field_and_value() {
        assert_eq!(
            field_and_value("foo   = bar"),
            Ok((
                "",
                ast::FV {
                    field: "foo".into(),
                    value: "bar".into()
                }
            ))
        );
    }

    #[rstest]
    fn test_field_and_constant() {
        assert_eq!(
            field_and_constant("f=g*"),
            Ok((
                "",
                ast::FC {
                    field: "f".into(),
                    value: ast::Constant::Wildcard(ast::Wildcard("g*".into()))
                }
            ))
        );
    }

    //   test("foo=bar bar=baz") {
    //     p(commandOptions(_), CommandOptions(Seq(
    //       FC("foo", Field("bar")),
    //       FC("bar", Field("baz"))
    //     )))
    //   }
    #[rstest]
    fn test_command_options() {
        assert_eq!(
            command_options("foo=bar bar=baz"),
            Ok((
                "",
                ast::CommandOptions {
                    options: vec![
                        ast::FC {
                            field: "foo".into(),
                            value: ast::Constant::Field(ast::Field("bar".into()))
                        },
                        ast::FC {
                            field: "bar".into(),
                            value: ast::Constant::Field(ast::Field("baz".into()))
                        }
                    ]
                }
            ))
        );
    }

    //   test("a ,   b,c, d") {
    //     p(fieldList(_), Seq(
    //       Field("a"),
    //       Field("b"),
    //       Field("c"),
    //       Field("d")
    //     ))
    //   }
    #[rstest]
    fn test_field_list() {
        assert_eq!(
            field_list0("a ,   b,c, d"),
            Ok((
                "",
                vec![
                    ast::Field("a".into()),
                    ast::Field("b".into()),
                    ast::Field("c".into()),
                    ast::Field("d".into())
                ]
            ))
        );
    }

    //   test("D:\\Work\\Stuff.xls") {
    //     p(filename(_), "D:\\Work\\Stuff.xls")
    //   }
    #[rstest]
    fn test_filename() {
        assert_eq!(
            filename("D:\\Work\\Stuff.xls"),
            Ok(("", "D:\\Work\\Stuff.xls"))
        );
    }

    //   test("-100500") {
    //     p(int(_), IntValue(-100500))
    //   }
    #[rstest]
    fn test_int() {
        assert_eq!(int("-100500"), Ok(("", ast::IntValue(-100500))));
    }

    //   test("1sec") {
    //     p(timeSpan(_), TimeSpan(1, "seconds"))
    //   }
    #[rstest]
    fn test_time_span_1sec() {
        assert_eq!(
            time_span("1sec"),
            Ok((
                "",
                ast::TimeSpan {
                    value: 1,
                    scale: "seconds".to_string()
                }
            ))
        );
    }

    //   test("5s") {
    //     p(timeSpan(_), TimeSpan(5, "seconds"))
    //   }
    #[rstest]
    fn test_time_span_5s() {
        assert_eq!(
            time_span("5s"),
            Ok((
                "",
                ast::TimeSpan {
                    value: 5,
                    scale: "seconds".to_string()
                }
            ))
        );
    }

    //   test("5second") {
    //     p(timeSpan(_), TimeSpan(5, "seconds"))
    //   }
    #[rstest]
    fn test_time_span_5second() {
        assert_eq!(
            time_span("5second"),
            Ok((
                "",
                ast::TimeSpan {
                    value: 5,
                    scale: "seconds".to_string()
                }
            ))
        );
    }

    //   test("5sec") {
    //     p(timeSpan(_), TimeSpan(5, "seconds"))
    //   }
    #[rstest]
    fn test_time_span_5sec() {
        assert_eq!(
            time_span("5sec"),
            Ok((
                "",
                ast::TimeSpan {
                    value: 5,
                    scale: "seconds".to_string()
                }
            ))
        );
    }

    //   test("5m") {
    //     p(timeSpan(_), TimeSpan(5, "minutes"))
    //   }
    #[rstest]
    fn test_time_span_5m() {
        assert_eq!(
            time_span("5m"),
            Ok((
                "",
                ast::TimeSpan {
                    value: 5,
                    scale: "minutes".to_string()
                }
            ))
        );
    }

    //   test("5mins") {
    //     p(timeSpan(_), TimeSpan(5, "minutes"))
    //   }
    #[rstest]
    fn test_time_span_5mins() {
        assert_eq!(
            time_span("5mins"),
            Ok((
                "",
                ast::TimeSpan {
                    value: 5,
                    scale: "minutes".to_string()
                }
            ))
        );
    }

    //   test("-5mon") {
    //     p(timeSpan(_), TimeSpan(-5, "months"))
    //   }
    #[rstest]
    fn test_time_span_minus5mon() {
        assert_eq!(
            time_span("-5mon"),
            Ok((
                "",
                ast::TimeSpan {
                    value: -5,
                    scale: "months".to_string()
                }
            ))
        );
    }

    //   test("-5d@w-2h") {
    //     p(constant(_), SnapTime(
    //       Some(TimeSpan(-5, "days")),
    //       "weeks",
    //       Some(TimeSpan(-2, "hours"))))
    //   }
    #[rstest]
    fn test_constant_snap_time_1() {
        assert_eq!(
            constant("-5d@w-2h"),
            Ok((
                "",
                ast::Constant::SnapTime(ast::SnapTime {
                    span: Some(ast::TimeSpan {
                        value: -5,
                        scale: "days".to_string()
                    }),
                    snap: "weeks".to_string(),
                    snap_offset: Some(ast::TimeSpan {
                        value: -2,
                        scale: "hours".to_string()
                    })
                })
            ))
        );
    }

    //   test("-5d@w0-2h") {
    //     p(constant(_), SnapTime(
    //       Some(TimeSpan(-5, "days")),
    //       "weeks",
    //       Some(TimeSpan(-2, "hours"))))
    //   }
    #[rstest]
    fn test_constant_snap_time_2() {
        assert_eq!(
            constant("-5d@w0-2h"),
            Ok((
                "",
                ast::Constant::SnapTime(ast::SnapTime {
                    span: Some(ast::TimeSpan {
                        value: -5,
                        scale: "days".to_string()
                    }),
                    snap: "weeks".to_string(),
                    snap_offset: Some(ast::TimeSpan {
                        value: -2,
                        scale: "hours".to_string()
                    })
                })
            ))
        );
    }

    //   test("-5d@w") {
    //     p(constant(_), SnapTime(
    //       Some(TimeSpan(-5, "days")),
    //       "weeks",
    //       None))
    //   }
    #[rstest]
    fn test_constant_snap_time_3() {
        assert_eq!(
            constant("-5d@w"),
            Ok((
                "",
                ast::Constant::SnapTime(ast::SnapTime {
                    span: Some(ast::TimeSpan {
                        value: -5,
                        scale: "days".to_string()
                    }),
                    snap: "weeks".to_string(),
                    snap_offset: None
                })
            ))
        );
    }

    //   test("@w") {
    //     p(constant(_), SnapTime(None, "weeks", None))
    //   }
    #[rstest]
    fn test_constant_snap_time_4() {
        assert_eq!(
            constant("@w"),
            Ok((
                "",
                ast::Constant::SnapTime(ast::SnapTime {
                    span: None,
                    snap: "weeks".to_string(),
                    snap_offset: None
                })
            ))
        );
    }

    //   test("@w-1d") {
    //     p(constant(_), SnapTime(None, "weeks", Some(TimeSpan(-1, "days"))))
    //   }
    #[rstest]
    fn test_constant_snap_time_5() {
        assert_eq!(
            constant("@w-1d"),
            Ok((
                "",
                ast::Constant::SnapTime(ast::SnapTime {
                    span: None,
                    snap: "weeks".to_string(),
                    snap_offset: Some(ast::TimeSpan {
                        value: -1,
                        scale: "days".to_string()
                    })
                })
            ))
        );
    }

    //   test("-1h@h") {
    //     p(constant(_), SnapTime(Some(TimeSpan(-1, "hours")), "hours", None))
    //   }
    #[rstest]
    fn test_constant_snap_time_6() {
        assert_eq!(
            constant("-1h@h"),
            Ok((
                "",
                ast::Constant::SnapTime(ast::SnapTime {
                    span: Some(ast::TimeSpan {
                        value: -1,
                        scale: "hours".to_string()
                    }),
                    snap: "hours".to_string(),
                    snap_offset: None
                })
            ))
        );
    }

    //   test("-h@h") {
    //     p(constant(_), SnapTime(Some(TimeSpan(-1, "hours")), "hours", None))
    //   }
    #[rstest]
    fn test_constant_snap_time_7() {
        assert_eq!(
            constant("-h@h"),
            Ok((
                "",
                ast::Constant::SnapTime(ast::SnapTime {
                    span: Some(ast::TimeSpan {
                        value: -1,
                        scale: "hours".to_string()
                    }),
                    snap: "hours".to_string(),
                    snap_offset: None
                })
            ))
        );
    }

    //   test("h@h") {
    //     p(constant(_), SnapTime(Some(TimeSpan(1, "hours")), "hours", None))
    //   }
    #[rstest]
    fn test_constant_snap_time_8() {
        assert_eq!(
            constant("h@h"),
            Ok((
                "",
                ast::Constant::SnapTime(ast::SnapTime {
                    span: Some(ast::TimeSpan {
                        value: 1,
                        scale: "hours".to_string()
                    }),
                    snap: "hours".to_string(),
                    snap_offset: None
                })
            ))
        );
    }

    //   test("a=b c=1 d=\"e\" f=g* h=-15m i=10.0.0.0/8 k=f") {
    //     p(commandOptions(_), CommandOptions(Seq(
    //       FC("a", Field("b")),
    //       FC("c", IntValue(1)),
    //       FC("d", StrValue("e")),
    //       FC("f", Wildcard("g*")),
    //       FC("h", TimeSpan(-15, "minutes")),
    //       FC("i", IPv4CIDR("10.0.0.0/8")),
    //       FC("k", Bool(false))
    //     )))
    //   }
    #[rstest]
    fn test_command_options_1() {
        assert_eq!(
            command_options("a=b c=1 d=\"e\" f=g* h=-15m i=10.0.0.0/8 k=f"),
            Ok((
                "",
                ast::CommandOptions {
                    options: vec![
                        ast::FC {
                            field: "a".into(),
                            value: ast::Constant::Field(ast::Field("b".to_string()))
                        },
                        ast::FC {
                            field: "c".into(),
                            value: ast::Constant::Int(ast::IntValue(1_i64))
                        },
                        ast::FC {
                            field: "d".into(),
                            value: ast::Constant::Str(ast::StrValue("e".into()))
                        },
                        ast::FC {
                            field: "f".into(),
                            value: ast::Constant::Wildcard(ast::Wildcard("g*".to_string()))
                        },
                        ast::FC {
                            field: "h".into(),
                            value: ast::Constant::TimeSpan(ast::TimeSpan {
                                value: -15,
                                scale: "minutes".to_string()
                            })
                        },
                        ast::FC {
                            field: "i".into(),
                            value: ast::Constant::IPv4CIDR(ast::IPv4CIDR("10.0.0.0/8".to_string()))
                        },
                        ast::FC {
                            field: "k".into(),
                            value: ast::Constant::Bool(ast::BoolValue(false))
                        }
                    ]
                }
            ))
        )
    }

    //   test("a OR b") {
    //     p(expr(_), Binary(
    //       Field("a"),
    //       Or,
    //       Field("b")
    //     ))
    //   }
    #[rstest]
    fn test_expr_or_1() {
        assert_eq!(
            expr("a OR b"),
            Ok((
                "",
                _or(ast::Field("a".to_string()), ast::Field("b".to_string()),)
            ))
        )
    }

    //   // TODO: add wildcard AST transformation
    //   test("productID=\"S*G01\"") {
    //     p(expr(_), Binary(
    //       Field("productID"),
    //       Equals,
    //       Wildcard("S*G01")
    //     ))
    //   }
    #[rstest]
    fn test_expr_wildcard_1() {
        assert_eq!(
            expr("productID=\"S*G01\""),
            Ok((
                "",
                _field_equals("productID", ast::Wildcard("S*G01".to_string()).into())
            ))
        )
    }

    //   test("(event_id=12 OR event_id=13 OR event_id=14)") {
    //     p(expr(_), Binary(
    //       Binary(
    //         Field("event_id"),
    //         Equals,
    //         IntValue(12)
    //       ),
    //       Or,
    //       Binary(
    //         Binary(
    //           Field("event_id"),
    //           Equals,
    //           IntValue(13)
    //         ),
    //         Or,
    //         Binary(
    //           Field("event_id"),
    //           Equals,
    //           IntValue(14)
    //         )
    //       )
    //     ))
    //   }
    #[rstest]
    fn test_expr_nested_or_1() {
        assert_eq!(
            expr("(event_id=12 OR event_id=13 OR event_id=14)"),
            Ok((
                "",
                _or(
                    _field_equals("event_id", ast::IntValue(12).into()),
                    _or(
                        _field_equals("event_id", ast::IntValue(13).into()),
                        _field_equals("event_id", ast::IntValue(14).into())
                    )
                )
            ))
        )
    }

    #[rstest]
    fn test_not() {
        assert_eq!(expr("NOT x"), Ok(("", _not(ast::Field::from("x")))));
    }

    //
    //   test("fields column_a, column_b, column_c") {
    //     p(fields(_),
    //       FieldsCommand(
    //         removeFields = false,
    //         Seq(
    //           Field("column_a"),
    //           Field("column_b"),
    //           Field("column_c")
    //         )
    //       )
    //     )
    //   }
    #[rstest]
    fn test_fields_1() {
        assert_eq!(
            FieldsParser::parse("fields column_a, column_b, column_c"),
            Ok((
                "",
                FieldsCommand {
                    remove_fields: false,
                    fields: vec![
                        ast::Field("column_a".to_string()),
                        ast::Field("column_b".to_string()),
                        ast::Field("column_c".to_string()),
                    ],
                }
            ))
        )
    }

    //
    //   test("fields + column_a, column_b") {
    //     p(fields(_),
    //       FieldsCommand(
    //         removeFields = false,
    //         Seq(
    //           Field("column_a"),
    //           Field("column_b")
    //         )
    //       )
    //     )
    //   }
    #[rstest]
    fn test_fields_2() {
        assert_eq!(
            FieldsParser::parse("fields + column_a, column_b"),
            Ok((
                "",
                FieldsCommand {
                    remove_fields: false,
                    fields: vec![
                        ast::Field("column_a".to_string()),
                        ast::Field("column_b".to_string()),
                    ],
                }
            ))
        )
    }

    //
    //   test("fields - column_a, column_b") {
    //     p(fields(_),
    //       FieldsCommand(
    //         removeFields = true,
    //         Seq(
    //           Field("column_a"),
    //           Field("column_b")
    //         )
    //       )
    //     )
    //   }
    #[rstest]
    fn test_fields_3() {
        assert_eq!(
            FieldsParser::parse("fields - column_a, column_b"),
            Ok((
                "",
                FieldsCommand {
                    remove_fields: true,
                    fields: vec![
                        ast::Field("column_a".to_string()),
                        ast::Field("column_b".to_string()),
                    ],
                }
            ))
        )
    }

    //
    //   test("sort A, -B, +num(C)") {
    //     p(sort(_),
    //       SortCommand(
    //         Seq(
    //           (None, Field("A")),
    //           (Some("-"), Field("B")),
    //           (Some("+"), Call("num", Seq(Field("C"))))
    //         )
    //       )
    //     )
    //   }
    #[rstest]
    fn test_sort_1() {
        assert_eq!(
            SortParser::parse("sort A, -B, +num(C)"),
            Ok((
                "",
                SortCommand {
                    fields_to_sort: vec![
                        (None, ast::Field::from("A").into()),
                        (Some("-".into()), ast::Field::from("B").into()),
                        (Some("+".into()), _call!(num(ast::Field::from("C"))).into()),
                    ],
                    ..Default::default()
                }
            ))
        )
    }

    //
    //   test("TERM(XXXXX*\\\\XXXXX*)") {
    //     p(pipeline(_), Pipeline(Seq(
    //       SearchCommand(
    //         Call("TERM", List(Field("XXXXX*\\\\XXXXX*")))
    //       )
    //     )))
    //   }
    #[rstest]
    fn test_pipeline_term_call_1() {
        assert_eq!(
            pipeline("TERM(XXXXX*\\\\XXXXX*)"),
            Ok((
                "",
                ast::Pipeline {
                    commands: vec![SearchCommand {
                        expr: _call!(TERM(ast::Field::from("XXXXX*\\\\XXXXX*"))).into(),
                    }
                    .into()],
                }
            ))
        )
    }

    //
    //   test("values(eval(mvappend(\"a: \" . a, \"b: \" . b)))") {
    //     p(pipeline(_), Pipeline(Seq(
    //       SearchCommand(
    //         Call("values", Seq(
    //           Call("eval", Seq(
    //             Call("mvappend",
    //               Seq(
    //                 Binary(
    //                   StrValue("a: "),
    //                   Concatenate,
    //                   Field("a")
    //                 ),
    //                 Binary(
    //                   StrValue("b: "),
    //                   Concatenate,
    //                   Field("b")
    //                 )))))))))))
    //   }
    #[rstest]
    fn test_pipeline_values_eval_mvappend_1() {
        assert_eq!(
            pipeline("values(eval(mvappend(\"a: \" . a, \"b: \" . b)))"),
            Ok((
                "",
                ast::Pipeline {
                    commands: vec![SearchCommand {
                        expr: _call!(values(_call!(eval(_call!(mvappend(
                            _binop::<operators::Concatenate>(
                                ast::StrValue::from("a: "),
                                ast::Field::from("a")
                            ),
                            _binop::<operators::Concatenate>(
                                ast::StrValue::from("b: "),
                                ast::Field::from("b")
                            )
                        ))))))
                        .into()
                    }
                    .into()]
                }
            ))
        )
    }

    //
    //   test("sort A") {
    //     p(sort(_),
    //       SortCommand(
    //         Seq(
    //           (None, Field("A"))
    //         )
    //       )
    //     )
    //   }
    #[rstest]
    fn test_sort_2() {
        assert_eq!(
            SortParser::parse("sort A"),
            Ok((
                "",
                SortCommand {
                    fields_to_sort: vec![(None, ast::Field::from("A").into())],
                    ..Default::default()
                }
            ))
        )
    }

    //
    //   test("index=foo bar=baz | eval foo=bar | collect index=newer") {
    //     p(pipeline(_), Pipeline(Seq(
    //       SearchCommand(
    //         Binary(
    //           Binary(
    //             Field("index"),
    //             Equals,
    //             Field("foo")
    //           ),
    //           And,
    //           Binary(
    //             Field("bar"),
    //             Equals,
    //             Field("baz")
    //           )
    //         )
    //       ),
    //       EvalCommand(Seq(
    //         (Field("foo"),Field("bar"))
    //       )),
    //       CollectCommand(
    //         index = "newer",
    //         fields = Seq(),
    //         addTime = true,
    //         file = null,
    //         host = null,
    //         marker = null,
    //         outputFormat = "raw",
    //         runInPreview = false,
    //         spool = true,
    //         source = null,
    //         sourceType = null,
    //         testMode = false
    //       )
    //     )))
    //   }
    #[rstest]
    fn test_pipeline_index_eval_collect_4() {
        assert_eq!(
            pipeline("index=foo bar=baz | eval foo=bar | collect index=newer"),
            Ok((
                "",
                ast::Pipeline {
                    commands: vec![
                        SearchCommand {
                            expr: _and(
                                _eq(ast::Field::from("index"), ast::Field::from("foo")),
                                _eq(ast::Field::from("bar"), ast::Field::from("baz"))
                            )
                        }
                        .into(),
                        EvalCommand {
                            fields: vec![(ast::Field::from("foo"), ast::Field::from("bar").into())]
                        }
                        .into(),
                        CollectCommand {
                            index: "newer".to_string(),
                            fields: vec![],
                            add_time: true,
                            file: None,
                            host: None,
                            marker: None,
                            output_format: "raw".to_string(),
                            run_in_preview: false,
                            spool: true,
                            source: None,
                            source_type: None,
                            test_mode: false,
                        }
                        .into()
                    ],
                }
            ))
        )
    }

    #[rstest]
    fn test_aliased_call() {
        assert_eq!(
            aliased_call("first(startTime) AS startTime"),
            Ok((
                "",
                _alias("startTime", _call!(first(ast::Field::from("startTime"))))
            ))
        );
    }

    #[rstest]
    fn test_stats_call() {
        assert_eq!(
            stats_call("first(startTime) AS startTime, last(histID) AS lastPassHistId"),
            Ok((
                "",
                vec![
                    _alias("startTime", _call!(first(ast::Field::from("startTime")))).into(),
                    _alias("lastPassHistId", _call!(last(ast::Field::from("histID")))).into(),
                ]
            ))
        )
    }

    #[rstest]
    fn test_case_call_1() {
        assert_eq!(
            expr(r#"status==200"#),
            Ok((
                "",
                _binop::<operators::StrictlyEquals>(ast::Field::from("status"), ast::IntValue(200))
            ))
        );

        let _call_ast = ast::Call {
            name: "case".to_string(),
            args: vec![
                _binop::<operators::StrictlyEquals>(ast::Field::from("status"), ast::IntValue(200)),
                ast::StrValue::from("OK").into(),
                _binop::<operators::StrictlyEquals>(ast::Field::from("status"), ast::IntValue(404)),
                ast::StrValue::from("Not found").into(),
                _binop::<operators::StrictlyEquals>(ast::Field::from("status"), ast::IntValue(500)),
                ast::StrValue::from("Internal Server Error").into(),
            ],
        };
        assert_eq!(
            call(
                r#"case(status==200, "OK", status==404, "Not found", status==500, "Internal Server Error")"#
            ),
            Ok(("", _call_ast.clone()))
        );
        assert_eq!(
            EvalParser::parse(
                r#"eval description=case(status==200, "OK", status==404, "Not found", status==500, "Internal Server Error")"#
            ),
            Ok((
                "",
                EvalCommand {
                    fields: vec![(ast::Field::from("description"), _call_ast.into())],
                }
            ))
        );
    }

    #[rstest]
    fn test_logical_expression_1() {
        assert_eq!(
            field(r#"Processes.process_name=wmic.exe"#),
            Ok(("=wmic.exe", ast::Field::from("Processes.process_name")))
        );
        assert_eq!(
            literal(r#"wmic.exe"#),
            Ok(("", ast::StrValue::from("wmic.exe").into()))
        );
        assert_eq!(
            comparison_expression(r#"Processes.process_name=wmic.exe"#),
            Ok((
                "",
                _eq(
                    ast::Field::from("Processes.process_name"),
                    ast::StrValue::from("wmic.exe")
                )
            ))
        );
        assert_eq!(
            logical_expression(r#"Processes.process_name=wmic.exe"#),
            Ok((
                "",
                _eq(
                    ast::Field::from("Processes.process_name"),
                    ast::StrValue::from("wmic.exe")
                )
            ))
        );
    }

    #[rstest]
    fn test_logical_expression_2() {
        assert_eq!(
            logical_expression(
                r#"Processes.process_name=wmic.exe OR Processes.original_file_name=wmic.exe"#
            ),
            Ok((
                "",
                _or(
                    _eq(
                        ast::Field::from("Processes.process_name"),
                        ast::StrValue::from("wmic.exe")
                    ),
                    _eq(
                        ast::Field::from("Processes.original_file_name"),
                        ast::StrValue::from("wmic.exe")
                    )
                )
            ))
        );
    }

    #[rstest]
    fn test_logical_expression_3() {
        assert_eq!(
            logical_expression(r#"Processes.process = "*os get*""#),
            Ok((
                "",
                _eq(
                    ast::Field::from("Processes.process"),
                    ast::Wildcard::from("*os get*")
                )
            ))
        );
    }

    #[rstest]
    fn test_logical_expression_4() {
        assert_eq!(
            logical_expression(
                r#"Processes.process_name=wmic.exe OR Processes.original_file_name=wmic.exe"#
            ),
            logical_expression(
                r#"(Processes.process_name=wmic.exe OR Processes.original_file_name=wmic.exe)"#
            ),
        );
        assert_eq!(
            logical_expression(
                r#"(Processes.process_name=wmic.exe OR Processes.original_file_name=wmic.exe)"#
            ),
            Ok((
                "",
                _or(
                    _eq(
                        ast::Field::from("Processes.process_name"),
                        ast::StrValue::from("wmic.exe")
                    ),
                    _eq(
                        ast::Field::from("Processes.original_file_name"),
                        ast::StrValue::from("wmic.exe")
                    )
                )
            ))
        );
    }

    #[rstest]
    fn test_logical_expression_5() {
        assert_eq!(
            logical_expression(
                r#"(Processes.process_name=wmic.exe OR Processes.original_file_name=wmic.exe) Processes.process = "*os get*" Processes.process="*/format:*" Processes.process = "*.xsl*""#
            ),
            Ok((
                "",
                _and(
                    _and(
                        _and(
                            _or(
                                _eq(
                                    ast::Field::from("Processes.process_name"),
                                    ast::StrValue::from("wmic.exe")
                                ),
                                _eq(
                                    ast::Field::from("Processes.original_file_name"),
                                    ast::StrValue::from("wmic.exe")
                                )
                            ),
                            _eq(
                                ast::Field::from("Processes.process"),
                                ast::Wildcard::from("*os get*")
                            ),
                        ),
                        _eq(
                            ast::Field::from("Processes.process"),
                            ast::Wildcard::from("*/format:*")
                        ),
                    ),
                    _eq(
                        ast::Field::from("Processes.process"),
                        ast::Wildcard::from("*.xsl*")
                    )
                )
            ))
        );
    }

    #[rstest]
    fn test_logical_expression_6() {
        assert_eq!(
            logical_expression(r#"NOT x=y"#),
            Ok((
                "",
                _not(_eq(ast::Field::from("x"), ast::StrValue::from("y")))
            ))
        );
    }

    #[rstest]
    fn test_logical_expression_7() {
        assert_eq!(
            literal(r#"127.0.0.1"#),
            Ok(("", ast::StrValue::from("127.0.0.1").into()))
        );
        assert_eq!(
            literal(r#"10.0.0.0/8"#),
            Ok(("", ast::IPv4CIDR::from("10.0.0.0/8").into()))
        );
        assert_eq!(
            literal(r#"172.16.0.0/12"#),
            Ok(("", ast::IPv4CIDR::from("172.16.0.0/12").into()))
        );
        assert_eq!(
            literal(r#"192.168.0.0/16"#),
            Ok(("", ast::IPv4CIDR::from("192.168.0.0/16").into()))
        );
        assert_eq!(
            literal(r#"0:0:0:0:0:0:0:1"#),
            Ok(("", ast::StrValue::from("0:0:0:0:0:0:0:1").into()))
        );

        assert_eq!(
            logical_expression(
                r#"All_Traffic.dest IN (127.0.0.1,10.0.0.0/8,172.16.0.0/12, 192.168.0.0/16, 0:0:0:0:0:0:0:1)"#
            ),
            Ok((
                "",
                _isin(
                    "All_Traffic.dest",
                    vec![
                        ast::StrValue::from("127.0.0.1").into(),
                        ast::IPv4CIDR::from("10.0.0.0/8").into(),
                        ast::IPv4CIDR::from("172.16.0.0/12").into(),
                        ast::IPv4CIDR::from("192.168.0.0/16").into(),
                        ast::StrValue::from("0:0:0:0:0:0:0:1").into(),
                    ]
                )
            ))
        );
        assert_eq!(
            logical_expression(
                r#"NOT (All_Traffic.dest IN (127.0.0.1,10.0.0.0/8,172.16.0.0/12, 192.168.0.0/16, 0:0:0:0:0:0:0:1))"#
            ),
            Ok((
                "",
                _not(_isin(
                    "All_Traffic.dest",
                    vec![
                        ast::StrValue::from("127.0.0.1").into(),
                        ast::IPv4CIDR::from("10.0.0.0/8").into(),
                        ast::IPv4CIDR::from("172.16.0.0/12").into(),
                        ast::IPv4CIDR::from("192.168.0.0/16").into(),
                        ast::StrValue::from("0:0:0:0:0:0:0:1").into(),
                    ]
                ))
            ))
        );
    }

    #[rstest]
    fn test_logical_expression_8() {
        assert_eq!(
            logical_expression(r#"a=1 OR b=2 AND c=3 OR (d=4 e=5)"#),
            Ok((
                "",
                _or(
                    _or(
                        _eq(ast::Field::from("a"), ast::IntValue::from(1)),
                        _and(
                            _eq(ast::Field::from("b"), ast::IntValue::from(2)),
                            _eq(ast::Field::from("c"), ast::IntValue::from(3))
                        ),
                    ),
                    _and(
                        _eq(ast::Field::from("d"), ast::IntValue::from(4)),
                        _eq(ast::Field::from("e"), ast::IntValue::from(5))
                    )
                )
            ))
        );
    }

    #[rstest]
    fn test_logical_expression_9() {
        assert_eq!(
            logical_expression(r#"a=1 b=2"#),
            Ok((
                "",
                _and(
                    _eq(ast::Field::from("a"), ast::IntValue::from(1)),
                    _eq(ast::Field::from("b"), ast::IntValue::from(2)),
                )
            ))
        );
    }

    #[rstest]
    fn test_int_in_string_1() {
        assert_eq!(expr("3"), Ok(("", ast::IntValue::from(3).into())));
        assert_eq!(expr("3x"), Ok(("", ast::Field::from("3x").into())));
    }

    #[rstest]
    fn test_field_starting_with_not() {
        assert_eq!(
            expr("note!=ESCU*"),
            Ok((
                "",
                _neq(ast::Field::from("note"), ast::Wildcard::from("ESCU*"))
            ))
        );
    }

    #[rstest]
    fn test_logical_expression_10() {
        assert_eq!(
            expr("a=1 AND (b=2 OR c=3)"),
            Ok((
                "",
                _and(
                    _eq(ast::Field::from("a"), ast::IntValue::from(1)),
                    _or(
                        _eq(ast::Field::from("b"), ast::IntValue::from(2)),
                        _eq(ast::Field::from("c"), ast::IntValue::from(3))
                    )
                )
            ))
        );
    }

    #[rstest]
    fn test_logical_expression_11() {
        assert_eq!(
            logical_expression("isnull(firstTimeSeenUserApiCall) OR firstTimeSeenUserApiCall > relative_time(now(),\"-24h@h\")"),
            Ok((
                "",
                _or(
                    _call!(isnull(ast::Field::from("firstTimeSeenUserApiCall"))),
                    _gt(
                        ast::Field::from("firstTimeSeenUserApiCall"),
                        _call!(
                            relative_time(
                                _call!(now()),
                                ast::SnapTime {
                                    span: Some(ast::TimeSpan {
                                        value: -24,
                                        scale: "hours".to_string(),
                                    }),
                                    snap: "hours".to_string(),
                                    snap_offset: None,
                                }
                            )
                        )
                    )
                )
            ))
        );
    }

    #[rstest]
    fn test_string_double_double_quotes() {
        // "xyz" => "xyz"
        assert_eq!(double_quoted(r#""xyz""#), Ok(("", r#"xyz"#)));
        // ""xyz"" => "\"xyz\""
        assert_eq!(double_quoted(r#"""xyz"""#), Ok(("", r#""xyz""#)));
        // "" => ""
        assert_eq!(double_quoted(r#""""#), Ok(("", r#""#)));
    }
}
