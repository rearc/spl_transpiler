use crate::commands::cmd_add_totals::spl::AddTotalsParser;
use crate::commands::cmd_bin::spl::BinParser;
use crate::commands::cmd_collect::spl::CollectParser;
use crate::commands::cmd_convert::spl::ConvertParser;
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
use crate::commands::cmd_regex::spl::RegexParser;
use crate::commands::cmd_rename::spl::RenameParser;
use crate::commands::cmd_return::spl::ReturnParser;
use crate::commands::cmd_rex::spl::RexParser;
use crate::commands::cmd_search::spl::SearchParser;
use crate::commands::cmd_sort::spl::SortParser;
use crate::commands::cmd_stats::spl::StatsParser;
use crate::commands::cmd_stream_stats::spl::StreamStatsParser;
use crate::commands::cmd_table::spl::TableParser;
use crate::commands::cmd_top::spl::TopParser;
use crate::commands::cmd_where::spl::WhereParser;
use crate::commands::spl::SplCommand;
use crate::spl::operators::OperatorSymbolTrait;
use crate::spl::{ast, operators, operators::OperatorSymbol};
use nom::bytes::complete::{tag_no_case, take_while};
use nom::character::complete::{alphanumeric1, anychar, multispace0, multispace1};
use nom::combinator::{all_consuming, into, map_parser, verify};
use nom::error::ParseError;
use nom::multi::{many0, separated_list0, separated_list1};
use nom::sequence::{delimited, preceded, separated_pair};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{digit1, none_of},
    combinator::{map, opt, recognize},
    multi::many1,
    sequence::{pair, tuple},
    IResult, Parser,
};
use std::fmt::Debug;

// https://github.com/rust-bakery/nom/blob/main/doc/nom_recipes.md#wrapper-combinators-that-eat-whitespace-before-and-after-a-parser
pub fn ws<'a, O, E: ParseError<&'a str>, F>(inner: F) -> impl Parser<&'a str, O, E>
where
    F: Parser<&'a str, O, E>,
{
    delimited(multispace0, inner, multispace0)
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
pub fn relative_time(input: &str) -> IResult<&str, ast::SnapTime> {
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

//   def token[_: P]: P[String] = ("_"|"*"|letter|digit).repX(1).!
pub fn token(input: &str) -> IResult<&str, &str> {
    recognize(many1(alt((tag("_"), tag("*"), alphanumeric1))))(input)
}

// Boolean parser
pub fn bool_(input: &str) -> IResult<&str, ast::BoolValue> {
    map(
        alt((
            map(tag("true"), |_| true),
            map(tag("t"), |_| true),
            map(tag("false"), |_| false),
            map(tag("f"), |_| false),
        )),
        ast::BoolValue::from,
    )(input)
}

//   def doubleQuoted[_: P]: P[String] = P( "\"" ~ (CharsWhile(!"\"".contains(_)) | "\\" ~~ AnyChar | !"\"").rep.! ~ "\"" )
pub fn double_quoted(input: &str) -> IResult<&str, &str> {
    delimited(
        tag("\""),
        ws(recognize(many0(alt((
            recognize(pair(tag(r#"\"#), anychar)),
            recognize(none_of(r#""\"#)),
        ))))),
        tag("\""),
    )(input)
}

//   def wildcard[_: P]: P[Wildcard] = (
//       doubleQuoted.filter(_.contains("*")) | token.filter(_.contains("*"))) map Wildcard
pub fn wildcard(input: &str) -> IResult<&str, ast::Wildcard> {
    map(
        verify(alt((double_quoted, token)), |v: &str| v.contains("*")),
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
pub fn field(input: &str) -> IResult<&str, ast::Field> {
    map(_token_not_t_f, |v: &str| ast::Field(v.into()))(input)
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
pub fn cidr(input: &str) -> IResult<&str, ast::IPv4CIDR> {
    map(
        recognize(tuple((
            byte,
            tag("."),
            byte,
            tag("."),
            byte,
            tag("."),
            byte,
            ws(tag("/")),
            byte,
        ))),
        |v: &str| ast::IPv4CIDR(v.into()),
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
            map(
                verify(map(digit1, str::parse::<i64>), Result::is_ok),
                Result::unwrap,
            ),
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
pub fn constant(input: &str) -> IResult<&str, ast::Constant> {
    alt((
        map(cidr, ast::Constant::IPv4CIDR),
        map(wildcard, ast::Constant::Wildcard),
        map(str_value, ast::Constant::Str),
        map(variable, ast::Constant::Variable),
        map(relative_time, ast::Constant::SnapTime),
        map(time_span, |v| {
            ast::Constant::SplSpan(ast::SplSpan::TimeSpan(v))
        }),
        map(double, ast::Constant::Double),
        map(int, ast::Constant::Int),
        map(field, ast::Constant::Field),
        map(bool_, ast::Constant::Bool),
    ))(input)
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
pub fn field_list(input: &str) -> IResult<&str, Vec<ast::Field>> {
    separated_list0(tag(","), ws(field))(input)
}

#[allow(dead_code)]
//   def filename[_: P]: P[String] = term
pub fn filename(input: &str) -> IResult<&str, &str> {
    term(input)
}

#[allow(dead_code)]
//   def term[_: P]: P[String] = CharsWhile(!" ".contains(_)).!
pub fn term(input: &str) -> IResult<&str, &str> {
    recognize(take_while(|c: char| c != ' '))(input)
}

//   private def ALL[_: P]: P[OperatorSymbol] = (Or.P | And.P | LessThan.P | GreaterThan.P
//     | GreaterEquals.P | LessEquals.P | Equals.P | NotEquals.P | InList.P | Add.P | Subtract.P
//     | Multiply.P | Divide.P | Concatenate.P)
pub fn all(input: &str) -> IResult<&str, OperatorSymbol> {
    alt((
        map(operators::Or::pattern, |_| {
            OperatorSymbol::Or(operators::Or {})
        }),
        map(operators::And::pattern, |_| {
            OperatorSymbol::And(operators::And {})
        }),
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
        map(operators::Equals::pattern, |_| {
            OperatorSymbol::Equals(operators::Equals {})
        }),
        map(operators::NotEquals::pattern, |_| {
            OperatorSymbol::NotEquals(operators::NotEquals {})
        }),
        map(operators::InList::pattern, |_| {
            OperatorSymbol::InList(operators::InList {})
        }),
        map(operators::Add::pattern, |_| {
            OperatorSymbol::Add(operators::Add {})
        }),
        map(operators::Subtract::pattern, |_| {
            OperatorSymbol::Subtract(operators::Subtract {})
        }),
        map(operators::Multiply::pattern, |_| {
            OperatorSymbol::Multiply(operators::Multiply {})
        }),
        map(operators::Divide::pattern, |_| {
            OperatorSymbol::Divide(operators::Divide {})
        }),
        map(operators::Concatenate::pattern, |_| {
            OperatorSymbol::Concatenate(operators::Concatenate {})
        }),
    ))(input)
}

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
pub fn climb(left: ast::Expr, rights: Vec<(OperatorSymbol, ast::Expr)>, prec: i32) -> ast::Expr {
    if rights.is_empty() {
        left
    } else {
        let (sym, next): (OperatorSymbol, ast::Expr) = rights[0].clone();
        let remainder: Vec<_> = rights.into_iter().skip(1).collect();
        let symbol = sym.symbol_string().into();
        let precedence = sym.precedence();
        if precedence < prec {
            match left {
                ast::Expr::Binary(ast::Binary {
                    left: first,
                    symbol: prev_symbol,
                    right,
                }) => ast::Expr::Binary(ast::Binary {
                    left: first,
                    symbol: prev_symbol,
                    right: Box::new(climb(
                        ast::Expr::Binary(ast::Binary {
                            left: right,
                            symbol,
                            right: Box::new(next),
                        }),
                        remainder,
                        precedence + 1,
                    )),
                }),
                _ => climb(
                    ast::Expr::Binary(ast::Binary {
                        left: Box::new(left),
                        symbol,
                        right: Box::new(next),
                    }),
                    remainder,
                    precedence + 1,
                ),
            }
        } else {
            ast::Expr::Binary(ast::Binary {
                left: Box::new(left),
                symbol,
                right: Box::new(climb(next, remainder, precedence + 1)),
            })
        }
    }
}

//   def fieldIn[_: P]: P[FieldIn] =
//     token ~ "IN" ~ "(" ~ constant.rep(sep = ",".?) ~ ")" map FieldIn.tupled
pub fn field_in(input: &str) -> IResult<&str, ast::FieldIn> {
    map(
        separated_pair(
            token,
            delimited(multispace1, tag_no_case("IN"), multispace0),
            delimited(
                ws(tag("(")),
                separated_list0(ws(tag(",")), constant),
                ws(tag(")")),
            ),
        ),
        |(token, constants)| ast::FieldIn {
            field: token.into(),
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
        pair(
            token,
            delimited(tag("("), separated_list0(ws(tag(",")), expr), tag(")")),
        ),
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
        preceded(
            tag_no_case("TERM"),
            delimited(ws(tag("(")), take_while(|c| c != ')'), ws(tag(")"))),
        ),
        |term| ast::Call {
            name: "TERM".into(),
            args: vec![ast::Expr::Leaf(ast::LeafExpr::Constant(
                ast::Constant::Field(ast::Field(term.into())),
            ))],
        },
    )(input)
}

//   def argu[_: P]: P[Expr] = termCall | call | constant
pub fn argu(input: &str) -> IResult<&str, ast::Expr> {
    alt((
        map(term_call, ast::Expr::Call),
        map(call, ast::Expr::Call),
        map(constant, |v| ast::Expr::Leaf(ast::LeafExpr::Constant(v))),
    ))(input)
}

//   def parens[_: P]: P[Expr] = "(" ~ expr ~ ")"
pub fn parens(input: &str) -> IResult<&str, ast::Expr> {
    delimited(ws(tag("(")), expr, ws(tag(")")))(input)
}

//   def primary[_: P]: P[Expr] = unaryOf(expr) | fieldIn | parens | argu
pub fn primary(input: &str) -> IResult<&str, ast::Expr> {
    alt((
        map(
            preceded(pair(operators::UnaryNot::pattern, multispace1), expr),
            |e| {
                ast::Expr::Unary(ast::Unary {
                    symbol: operators::UnaryNot::SYMBOL.into(),
                    right: Box::new(e),
                })
            },
        ),
        map(field_in, ast::Expr::FieldIn),
        parens,
        argu,
    ))(input)
}

//   def expr[_: P]: P[Expr] = binaryOf(primary, ALL)
pub fn expr(input: &str) -> IResult<&str, ast::Expr> {
    map(
        pair(primary, many0(pair(ws(all), primary))),
        |(expr, tuples)| climb(expr, tuples, 100),
    )(input)
}

//
//   // lookup <lookup-dataset> (<lookup-field> [AS <event-field>] )...
//   // [ (OUTPUT | OUTPUTNEW) ( <lookup-destfield> [AS <event-destfield>] )...]
//   def aliasedField[_: P]: P[Alias] = field ~ W("AS") ~ (token|doubleQuoted) map Alias.tupled
pub fn aliased_field(input: &str) -> IResult<&str, ast::Alias> {
    map(
        separated_pair(field, ws(tag_no_case("AS")), alt((token, double_quoted))),
        |(field, alias)| ast::Alias {
            expr: Box::new(ast::Expr::Leaf(ast::LeafExpr::Constant(
                ast::Constant::Field(field),
            ))),
            name: alias.into(),
        },
    )(input)
}
//   def fieldRep[_: P]: P[Seq[FieldLike]] = (aliasedField | field).filter {
//     case Alias(Field(field), _) => field.toLowerCase() != "output"
//     case Field(v) => v.toLowerCase(Locale.ROOT) != "output"
//     case _ => false
//   }.rep(1)
pub fn field_rep(input: &str) -> IResult<&str, Vec<ast::FieldLike>> {
    separated_list1(
        multispace1,
        alt((
            map(
                verify(aliased_field, |v| v.name.to_ascii_lowercase() != "output"),
                ast::FieldLike::Alias,
            ),
            map(
                verify(field, |v| v.0.to_ascii_lowercase() != "output"),
                ast::FieldLike::Field,
            ),
        )),
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
    separated_list1(
        alt((ws(tag(",")), multispace1)),
        alt((
            map(aliased_call, |v| v.into()),
            map(call, |v| v.into()),
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
        )),
    )(input)
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
            into(StatsParser::parse),
            into(TableParser::parse),
            into(WhereParser::parse),
            into(InputLookupParser::parse),
            into(CollectParser::parse),
            into(ConvertParser::parse),
            into(EvalParser::parse),
            into(HeadParser::parse),
            into(FieldsParser::parse),
            into(SortParser::parse),
            into(RexParser::parse),
            into(RenameParser::parse),
            into(RegexParser::parse),
            into(JoinParser::parse),
            into(ReturnParser::parse),
            into(FillNullParser::parse),
            into(EventStatsParser::parse),
            into(StreamStatsParser::parse),
            into(DedupParser::parse),
            into(LookupParser::parse),
            into(FormatParser::parse),
        )),
        alt((
            into(MvCombineParser::parse),
            into(MvExpandParser::parse),
            into(BinParser::parse),
            into(MakeResultsParser::parse),
            into(AddTotalsParser::parse),
            into(MultiSearchParser::parse),
            into(MapParser::parse),
            into(TopParser::parse),
            into(SearchParser::parse),
        )),
    ))(input)
}
//
//   def subSearch[_: P]: P[Pipeline] = "[" ~ (command rep(sep = "|")) ~ "]" map Pipeline
pub fn sub_search(input: &str) -> IResult<&str, ast::Pipeline> {
    map(
        delimited(
            ws(tag("[")),
            separated_list0(ws(tag("|")), command),
            ws(tag("]")),
        ),
        |commands| ast::Pipeline { commands },
    )(input)
}

//   def pipeline[_: P]: P[Pipeline] = (command rep(sep = "|")) ~ End map Pipeline
pub fn pipeline(input: &str) -> IResult<&str, ast::Pipeline> {
    map(
        all_consuming(ws(separated_list0(ws(tag("|")), command))),
        |commands| ast::Pipeline { commands },
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::cmd_add_totals::spl::AddTotals;
    use crate::commands::cmd_bin::spl::BinCommand;
    use crate::commands::cmd_collect::spl::CollectCommand;
    use crate::commands::cmd_convert::spl::{ConvertCommand, ConvertParser, FieldConversion};
    use crate::commands::cmd_dedup::spl::{DedupCommand, DedupParser};
    use crate::commands::cmd_eval::spl::{EvalCommand, EvalParser};
    use crate::commands::cmd_event_stats::spl::EventStatsCommand;
    use crate::commands::cmd_fields::spl::{FieldsCommand, FieldsParser};
    use crate::commands::cmd_fill_null::spl::{FillNullCommand, FillNullParser};
    use crate::commands::cmd_format::spl::{FormatCommand, FormatParser};
    use crate::commands::cmd_head::spl::{HeadCommand, HeadParser};
    use crate::commands::cmd_input_lookup::spl::{InputLookup, InputLookupParser};
    use crate::commands::cmd_join::spl::{JoinCommand, JoinParser};
    use crate::commands::cmd_lookup::spl::{LookupCommand, LookupOutput, LookupParser};
    use crate::commands::cmd_make_results::spl::MakeResults;
    use crate::commands::cmd_map::spl::{MapCommand, MapParser};
    use crate::commands::cmd_mv_combine::spl::{MvCombineCommand, MvCombineParser};
    use crate::commands::cmd_regex::spl::{RegexCommand, RegexParser};
    use crate::commands::cmd_rename::spl::{RenameCommand, RenameParser};
    use crate::commands::cmd_return::spl::{ReturnCommand, ReturnParser};
    use crate::commands::cmd_rex::spl::RexCommand;
    use crate::commands::cmd_search::spl::{SearchCommand, SearchParser};
    use crate::commands::cmd_sort::spl::{SortCommand, SortParser};
    use crate::commands::cmd_stats::spl::StatsCommand;
    use crate::commands::cmd_table::spl::TableCommand;
    use crate::commands::cmd_where::spl::WhereCommand;

    fn _field_equals(field: &str, value: ast::Constant) -> ast::Expr {
        ast::Expr::Binary(ast::Binary {
            left: Box::new(ast::Expr::Leaf(ast::LeafExpr::Constant(
                ast::Constant::Field(ast::Field(field.into())),
            ))),
            symbol: operators::Equals::SYMBOL.into(),
            right: Box::new(ast::Expr::Leaf(ast::LeafExpr::Constant(value))),
        })
    }

    fn _binop<Op: OperatorSymbolTrait>(
        left: impl Into<ast::Expr>,
        right: impl Into<ast::Expr>,
    ) -> ast::Expr {
        ast::Expr::Binary(ast::Binary {
            left: Box::new(left.into()),
            symbol: Op::SYMBOL.into(),
            right: Box::new(right.into()),
        })
    }

    fn _or(left: impl Into<ast::Expr>, right: impl Into<ast::Expr>) -> ast::Expr {
        _binop::<operators::Or>(left, right)
    }

    fn _and(left: impl Into<ast::Expr>, right: impl Into<ast::Expr>) -> ast::Expr {
        _binop::<operators::And>(left, right)
    }

    fn _eq(left: impl Into<ast::Expr>, right: impl Into<ast::Expr>) -> ast::Expr {
        _binop::<operators::Equals>(left, right)
    }

    fn _neq(left: impl Into<ast::Expr>, right: impl Into<ast::Expr>) -> ast::Expr {
        _binop::<operators::NotEquals>(left, right)
    }
    fn _lt(left: impl Into<ast::Expr>, right: impl Into<ast::Expr>) -> ast::Expr {
        _binop::<operators::LessThan>(left, right)
    }
    fn _gt(left: impl Into<ast::Expr>, right: impl Into<ast::Expr>) -> ast::Expr {
        _binop::<operators::GreaterThan>(left, right)
    }
    fn _gte(left: impl Into<ast::Expr>, right: impl Into<ast::Expr>) -> ast::Expr {
        _binop::<operators::GreaterEquals>(left, right)
    }
    fn _lte(left: impl Into<ast::Expr>, right: impl Into<ast::Expr>) -> ast::Expr {
        _binop::<operators::LessEquals>(left, right)
    }
    fn _not(right: impl Into<ast::Expr>) -> ast::Expr {
        ast::Unary {
            symbol: operators::UnaryNot::SYMBOL.into(),
            right: Box::new(right.into()),
        }
        .into()
    }

    fn _alias(name: impl ToString, expr: impl Into<ast::Expr>) -> ast::Alias {
        ast::Alias {
            name: name.to_string(),
            expr: Box::new(expr.into()),
        }
    }

    // fn _call(name: impl ToString, args: Vec<ast::Expr>) -> ast::Call {
    //     ast::Call {
    //         name: name.to_string(),
    //         args,
    //     }
    // }
    // macro_rules! unit_normalizer_alt_list {
    //     ($name: literal) => { tag_no_case($name) };
    //     ($($remaining: literal),+) => {
    //         ($( unit_normalizer_alt_list!($remaining) ),+)
    //     }
    // }

    macro_rules! _call_args {
        // () => {};
        ($arg: expr) => {
            $arg.into()
        }; // ($arg: expr, $($rest: expr),+) => {
           //
           // };
    }

    macro_rules! _call {
        ($func: ident ()) => {
            ast::Call {
                name: stringify!($func).to_string(),
                args: vec![],
            }
        };
        ($func: ident ($($args: expr),*)) => {
            ast::Call {
                name: stringify!($func).to_string(),
                args: vec![$( _call_args!($args) ),*],
            }
        };
    }

    #[test]
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

    #[test]
    fn test_token() {
        let input = "token123";
        let result = token(input);
        assert_eq!(result, Ok(("", "token123")));
    }

    #[test]
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

    #[test]
    fn test_bool() {
        assert_eq!(bool_("false"), Ok(("", false.into())));
        assert_eq!(bool_("f"), Ok(("", false.into())));
        assert_eq!(bool_("true"), Ok(("", true.into())));
        assert_eq!(bool_("t"), Ok(("", true.into())));
    }
    #[test]
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
    #[test]
    fn test_field() {
        assert_eq!(
            expr("from"),
            Ok((
                "",
                ast::Expr::Leaf(ast::LeafExpr::Constant(ast::Constant::Field(ast::Field(
                    "from".into()
                ))))
            ))
        );
    }

    // test("foo   = bar") {
    //   p(fieldAndValue(_), FV("foo", "bar"))
    // }
    #[test]
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

    #[test]
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
    #[test]
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

    //   test("search index=dummy host=$host_var$") {
    //     p(search(_), SearchCommand(
    //       Binary(
    //         Binary(
    //           Field("index"),
    //           Equals,
    //           Field("dummy")
    //         ),
    //         And,
    //         Binary(
    //           Field("host"),
    //           Equals,
    //           Variable("host_var")
    //         )
    //       )]
    #[test]
    fn test_search() {
        assert_eq!(
            SearchParser::parse("search index=dummy host=$host_var$"),
            Ok((
                "",
                SearchCommand {
                    expr: _and(
                        _eq(ast::Field::from("index"), ast::Field::from("dummy")),
                        _eq(ast::Field::from("host"), ast::Variable::from("host_var"))
                    )
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
    #[test]
    fn test_field_list() {
        assert_eq!(
            field_list("a ,   b,c, d"),
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
    #[test]
    fn test_filename() {
        assert_eq!(
            filename("D:\\Work\\Stuff.xls"),
            Ok(("", "D:\\Work\\Stuff.xls"))
        );
    }

    //   test("-100500") {
    //     p(int(_), IntValue(-100500))
    //   }
    #[test]
    fn test_int() {
        assert_eq!(int("-100500"), Ok(("", ast::IntValue(-100500))));
    }

    //   test("1sec") {
    //     p(timeSpan(_), TimeSpan(1, "seconds"))
    //   }
    #[test]
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
    #[test]
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
    #[test]
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
    #[test]
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
    #[test]
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
    #[test]
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
    #[test]
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
    #[test]
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
    #[test]
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
    #[test]
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
    #[test]
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
    #[test]
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
    #[test]
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
    #[test]
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
    #[test]
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
    #[test]
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
                            value: ast::Constant::SplSpan(ast::SplSpan::TimeSpan(ast::TimeSpan {
                                value: -15,
                                scale: "minutes".to_string()
                            }))
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
    #[test]
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
    #[test]
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
    #[test]
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

    //   test("a=b b=c (c=f OR d=t)") {
    //     p(impliedSearch(_), SearchCommand(Binary(
    //       Binary(
    //         Binary(
    //           Field("a"),
    //           Equals,
    //           Field("b")
    //         ),
    //         And,
    //         Binary(
    //           Field("b"),
    //           Equals,
    //           Field("c")
    //         )
    //       ),
    //       And,
    //       Binary(
    //         Binary(
    //           Field("c"),
    //           Equals,
    //           Bool(false)
    //         ),
    //         Or,
    //         Binary(
    //           Field("d"),
    //           Equals,
    //           Bool(true)
    //         )
    //       )
    //     )))
    //   }
    #[test]
    fn test_implied_search_1() {
        assert_eq!(
            SearchParser::parse("a=b b=c (c=f OR d=t)"),
            Ok((
                "",
                SearchCommand {
                    expr: _and(
                        _and(
                            _field_equals("a", ast::Field("b".to_string()).into()),
                            _field_equals("b", ast::Field("c".to_string()).into())
                        ),
                        _or(
                            _field_equals("c", ast::BoolValue(false).into()),
                            _field_equals("d", ast::BoolValue(true).into())
                        )
                    )
                }
            ))
        )
    }

    //   test("code IN(4*, 5*)") {
    //     p(impliedSearch(_), SearchCommand(
    //       FieldIn("code", Seq(
    //         Wildcard("4*"),
    //         Wildcard("5*")
    //       ))))
    //   }
    #[test]
    fn test_implied_search_in_1() {
        assert_eq!(
            SearchParser::parse("code IN(4*, 5*)"),
            Ok((
                "",
                SearchCommand {
                    expr: ast::FieldIn {
                        field: "code".into(),
                        exprs: vec![
                            ast::Wildcard("4*".to_string()).into(),
                            ast::Wildcard("5*".to_string()).into(),
                        ],
                    }
                    .into()
                }
            ))
        )
    }

    //
    //   test("var_5 IN (str_2 str_3)") {
    //     p(impliedSearch(_), SearchCommand(
    //       FieldIn("var_5", Seq(
    //         Field("str_2"),
    //         Field("str_3")
    //       ))))
    //   }
    #[test]
    fn test_implied_search_in_2() {
        assert_eq!(
            field_in("var_5 IN (str_2, str_3)"),
            Ok((
                "",
                ast::FieldIn {
                    field: "var_5".into(),
                    exprs: vec![
                        ast::Field("str_2".to_string()).into(),
                        ast::Field("str_3".to_string()).into(),
                    ],
                }
            ))
        );
        assert_eq!(
            SearchParser::parse("var_5 IN (str_2, str_3)"),
            Ok((
                "",
                SearchCommand {
                    expr: ast::FieldIn {
                        field: "var_5".into(),
                        exprs: vec![
                            ast::Field("str_2".to_string()).into(),
                            ast::Field("str_3".to_string()).into(),
                        ],
                    }
                    .into()
                }
            ))
        );
    }

    //
    //   test("NOT code IN(4*, 5*)") {
    //     p(impliedSearch(_), SearchCommand(
    //       Unary(UnaryNot,
    //         FieldIn("code", Seq(
    //           Wildcard("4*"),
    //           Wildcard("5*"))))
    //     ))
    //   }
    #[test]
    fn test_not() {
        assert_eq!(expr("NOT x"), Ok(("", _not(ast::Field::from("x")))));
    }

    #[test]
    fn test_implied_search_not_in_1() {
        assert_eq!(
            SearchParser::parse("NOT code IN(4*, 5*)"),
            Ok((
                "",
                SearchCommand {
                    expr: _not(ast::FieldIn {
                        field: "code".into(),
                        exprs: vec![
                            ast::Wildcard("4*".to_string()).into(),
                            ast::Wildcard("5*".to_string()).into(),
                        ],
                    }),
                }
            ))
        );
    }

    //
    //   test("code IN(10, 29, 43) host!=\"localhost\" xqp>5") {
    //     p(impliedSearch(_), SearchCommand(
    //       Binary(
    //         Binary(
    //           FieldIn("code", Seq(
    //             IntValue(10),
    //             IntValue(29),
    //             IntValue(43))),
    //           And,
    //           Binary(
    //             Field("host"),
    //             NotEquals,
    //             StrValue("localhost")
    //           )
    //         ),
    //         And,
    //         Binary(
    //           Field("xqp"),
    //           GreaterThan,
    //           IntValue(5)
    //         )
    //       )
    //     ))
    //   }
    #[test]
    fn test_implied_search_complex_1() {
        assert_eq!(
            SearchParser::parse("code IN(10, 29, 43) host!=\"localhost\" xqp>5"),
            Ok((
                "",
                SearchCommand {
                    expr: _and(
                        _and(
                            ast::FieldIn {
                                field: "code".into(),
                                exprs: vec![
                                    ast::IntValue(10).into(),
                                    ast::IntValue(29).into(),
                                    ast::IntValue(43).into(),
                                ],
                            },
                            _neq(
                                ast::Field("host".to_string()),
                                ast::StrValue("localhost".to_string()),
                            )
                        ),
                        _gt(ast::Field("xqp".to_string()), ast::IntValue(5),)
                    )
                }
            ))
        )
    }

    //
    //   test("head 20") {
    //     p(head(_),
    //       HeadCommand(
    //         IntValue(20),
    //         Bool(false),
    //         Bool(false)
    //       )
    //     )
    //   }
    #[test]
    fn test_head_limit_1() {
        assert_eq!(
            HeadParser::parse("head 20"),
            Ok((
                "",
                HeadCommand {
                    eval_expr: ast::IntValue(20).into(),
                    keep_last: ast::BoolValue(false),
                    null_option: ast::BoolValue(false),
                }
                .into()
            ))
        )
    }

    //
    //   test("head limit=400") {
    //     p(head(_),
    //       HeadCommand(
    //         IntValue(400),
    //         Bool(false),
    //         Bool(false))
    //     )
    //   }
    #[test]
    fn test_head_limit_2() {
        assert_eq!(
            HeadParser::parse("head limit=400"),
            Ok((
                "",
                HeadCommand {
                    eval_expr: ast::IntValue(400).into(),
                    keep_last: ast::BoolValue(false),
                    null_option: ast::BoolValue(false),
                }
                .into()
            ))
        )
    }

    //
    //   test("head limit=400 keeplast=true null=false") {
    //     p(head(_),
    //       HeadCommand(
    //         IntValue(400),
    //         Bool(true),
    //         Bool(false)
    //       )
    //     )
    //   }
    #[test]
    fn test_head_limit_3() {
        assert_eq!(
            HeadParser::parse("head limit=400 keeplast=true null=false"),
            Ok((
                "",
                HeadCommand {
                    eval_expr: ast::IntValue(400).into(),
                    keep_last: ast::BoolValue(true),
                    null_option: ast::BoolValue(false),
                }
                .into()
            ))
        )
    }

    //
    //   test("head count>10") {
    //     p(head(_),
    //       HeadCommand(
    //         Binary(
    //           Field("count"),
    //           GreaterThan,
    //           IntValue(10)
    //         ),
    //         Bool(false),
    //         Bool(false)
    //       )
    //     )
    //   }
    #[test]
    fn test_head_count_greater_than_10() {
        assert_eq!(
            HeadParser::parse("head count>10"),
            Ok((
                "",
                HeadCommand {
                    eval_expr: _gt(ast::Field("count".to_string()), ast::IntValue(10),),
                    keep_last: ast::BoolValue(false),
                    null_option: ast::BoolValue(false),
                }
                .into()
            ))
        )
    }

    #[test]
    fn test_head_count_greater_than_10_keeplast() {
        assert_eq!(
            HeadParser::parse("head count>=10 keeplast=true"),
            Ok((
                "",
                HeadCommand {
                    eval_expr: _gte(ast::Field("count".to_string()), ast::IntValue(10),),
                    keep_last: ast::BoolValue(true),
                    null_option: ast::BoolValue(false),
                }
                .into()
            ))
        )
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
    #[test]
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
    #[test]
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
    #[test]
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
    #[test]
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
    #[test]
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
    #[test]
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
    #[test]
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
    //   test("eval mitre_category=\"Discovery\"") {
    //     p(eval(_), EvalCommand(Seq(
    //       (Field("mitre_category"), StrValue("Discovery"))
    //     )))
    //   }
    #[test]
    fn test_eval_1() {
        assert_eq!(
            EvalParser::parse("eval mitre_category=\"Discovery\""),
            Ok((
                "",
                EvalCommand {
                    fields: vec![(
                        ast::Field::from("mitre_category"),
                        ast::StrValue::from("Discovery").into()
                    ),],
                }
            ))
        )
    }

    //
    //   test("eval email_lower=lower(email)") {
    //     p(eval(_), EvalCommand(Seq(
    //       (Field("email_lower"), Call("lower", Seq(Field("email"))))
    //     )))
    //   }
    #[test]
    fn test_eval_2() {
        assert_eq!(
            EvalParser::parse("eval email_lower=lower(email)"),
            Ok((
                "",
                EvalCommand {
                    fields: vec![(
                        ast::Field::from("email_lower"),
                        _call!(lower(ast::Field::from("email"))).into(),
                    ),],
                }
            ))
        )
    }

    //
    //   test("eval replaced=replace(email, \"@.+\", \"\")") {
    //     p(eval(_), EvalCommand(Seq(
    //       (Field("replaced"),
    //         Call("replace", Seq(Field("email"), StrValue("@.+"), StrValue(""))))
    //     )))
    //   }
    #[test]
    fn test_eval_3_args() {
        assert_eq!(
            call("replace(email, \"@.+\", \"\")"),
            Ok((
                "",
                _call!(replace(
                    ast::Field::from("email"),
                    ast::StrValue::from("@.+"),
                    ast::StrValue::from("")
                ))
            ))
        );
        assert_eq!(
            expr("replace(email, \"@.+\", \"\")"),
            Ok((
                "",
                _call!(replace(
                    ast::Field::from("email"),
                    ast::StrValue::from("@.+"),
                    ast::StrValue::from("")
                ))
                .into()
            ))
        );
    }

    #[test]
    fn test_eval_3() {
        assert_eq!(
            EvalParser::parse("eval replaced=replace(email, \"@.+\", \"\")"),
            Ok((
                "",
                EvalCommand {
                    fields: vec![(
                        ast::Field::from("replaced"),
                        _call!(replace(
                            ast::Field::from("email"),
                            ast::StrValue::from("@.+"),
                            ast::StrValue::from("")
                        ))
                        .into()
                    ),],
                }
            ))
        )
    }

    //
    //   test("eval hash_sha256= lower(hash_sha256), b=c") {
    //     p(eval(_), EvalCommand(Seq(
    //       (Field("hash_sha256"), Call("lower", Seq(Field("hash_sha256")))),
    //       (Field("b"), Field("c"))
    //     )))
    //   }
    #[test]
    fn test_eval_4() {
        assert_eq!(
            EvalParser::parse("eval hash_sha256= lower(hash_sha256), b=c"),
            Ok((
                "",
                EvalCommand {
                    fields: vec![
                        (
                            ast::Field::from("hash_sha256"),
                            _call!(lower(ast::Field::from("hash_sha256"))).into(),
                        ),
                        (ast::Field::from("b"), ast::Field::from("c").into()),
                    ],
                }
            ))
        )
    }

    //
    //   test("convert ctime(indextime)") {
    //     p(convert(_), ConvertCommand(convs = Seq(
    //       FieldConversion("ctime", Field("indextime"), None)
    //     )))
    //   }
    #[test]
    fn test_convert_1() {
        assert_eq!(
            ConvertParser::parse("convert ctime(indextime)"),
            Ok((
                "",
                ConvertCommand {
                    timeformat: "%m/%d/%Y %H:%M:%S".to_string(),
                    convs: vec![FieldConversion {
                        func: "ctime".into(),
                        field: ast::Field::from("indextime"),
                        alias: None,
                    },],
                }
                .into()
            ))
        )
    }

    //
    //   test("collect index=threathunting addtime=f x, y,  z") {
    //     p(pipeline(_), Pipeline(Seq(
    //       CollectCommand(
    //         index = "threathunting",
    //         fields = Seq(
    //           Field("x"),
    //           Field("y"),
    //           Field("z")
    //         ),
    //         addTime = false,
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
    #[test]
    fn test_pipeline_collect_3() {
        assert_eq!(
            pipeline("collect index=threathunting addtime=f x, y,  z"),
            Ok((
                "",
                ast::Pipeline {
                    commands: vec![CollectCommand {
                        index: "threathunting".to_string(),
                        fields: vec![
                            ast::Field("x".into()),
                            ast::Field("y".into()),
                            ast::Field("z".into()),
                        ],
                        add_time: false,
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
                    .into()],
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
    #[test]
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

    //
    //   test("lookup process_create_whitelist a b output reason") {
    //     p(pipeline(_), Pipeline(Seq(
    //       LookupCommand(
    //         "process_create_whitelist",
    //         Seq(
    //           Field("a"),
    //           Field("b")
    //         ),
    //         Some(
    //           LookupOutput(
    //             "output",
    //             Seq(
    //               Field("reason")
    //             )
    //           )
    //         )
    //       )
    //     )))
    //   }
    #[test]
    fn test_pipeline_lookup_5() {
        let _lookup_cmd = LookupCommand {
            dataset: "process_create_whitelist".to_string(),
            fields: vec![ast::Field::from("a").into(), ast::Field::from("b").into()],
            output: Some(LookupOutput {
                kv: "output".to_string(),
                fields: vec![ast::Field::from("reason").into()],
            }),
        };
        assert_eq!(
            LookupParser::parse("lookup process_create_whitelist a b output reason"),
            Ok(("", _lookup_cmd.clone()))
        );
        assert_eq!(
            pipeline("lookup process_create_whitelist a b output reason"),
            Ok((
                "",
                ast::Pipeline {
                    commands: vec![_lookup_cmd.clone().into()],
                }
            ))
        )
    }

    //
    //   test("where isnull(reason)") {
    //     p(pipeline(_), Pipeline(Seq(
    //       WhereCommand(
    //         Call(
    //           "isnull",Seq(
    //             Field("reason")
    //           )
    //         )
    //       )
    //     )))
    //   }
    #[test]
    fn test_pipeline_where_6() {
        assert_eq!(
            pipeline("where isnull(reason)"),
            Ok((
                "",
                ast::Pipeline {
                    commands: vec![WhereCommand {
                        expr: _call!(isnull(ast::Field::from("reason"))).into()
                    }
                    .into()],
                }
            ))
        )
    }

    //
    //   test("table foo bar baz*") {
    //     p(pipeline(_), Pipeline(Seq(
    //       TableCommand(Seq(
    //         Field("foo"),
    //         Field("bar"),
    //         Field("baz*")
    //       ))
    //     )))
    //   }
    #[test]
    fn test_pipeline_table_7() {
        assert_eq!(
            pipeline("table foo bar baz*"),
            Ok((
                "",
                ast::Pipeline {
                    commands: vec![TableCommand {
                        fields: vec![
                            ast::Field::from("foo"),
                            ast::Field::from("bar"),
                            ast::Field::from("baz*")
                        ]
                    }
                    .into()],
                }
            ))
        )
    }

    //
    //   test("stats first(startTime) AS startTime, last(histID) AS lastPassHistId BY testCaseId") {
    //     p(pipeline(_), Pipeline(Seq(
    //       StatsCommand(
    //         partitions = 1,
    //         allNum = false,
    //         delim = " ",
    //         funcs = Seq(
    //           Alias(
    //             Call("first", Seq(
    //               Field("startTime")
    //             )),
    //             "startTime"),
    //           Alias(
    //             Call("last", Seq(
    //               Field("histID")
    //             )),
    //             "lastPassHistId")
    //         ),
    //         by = Seq(
    //           Field("testCaseId")
    //         ))
    //     )))
    //   }
    #[test]
    fn test_aliased_call() {
        assert_eq!(
            aliased_call("first(startTime) AS startTime"),
            Ok((
                "",
                _alias("startTime", _call!(first(ast::Field::from("startTime"))))
            ))
        );
    }

    #[test]
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

    #[test]
    fn test_pipeline_stats_8() {
        assert_eq!(
            pipeline(
                "stats first(startTime) AS startTime, last(histID) AS lastPassHistId BY testCaseId"
            ),
            Ok((
                "",
                ast::Pipeline {
                    commands: vec![StatsCommand {
                        partitions: 1,
                        all_num: false,
                        delim: " ".to_string(),
                        funcs: vec![
                            _alias("startTime", _call!(first(ast::Field::from("startTime"))))
                                .into(),
                            _alias("lastPassHistId", _call!(last(ast::Field::from("histID"))))
                                .into(),
                        ],
                        by: vec![ast::Field::from("testCaseId")],
                        dedup_split_vals: false
                    }
                    .into()],
                }
                .into()
            ))
        )
    }

    //
    //   test("stats count(eval(status=404))") {
    //     p(pipeline(_), Pipeline(Seq(
    //       StatsCommand(
    //         partitions = 1,
    //         allNum = false,
    //         delim = " ",
    //         funcs = Seq(
    //           Call("count", Seq(
    //             Call("eval", Seq(
    //               Binary(
    //                 Field("status"),
    //                 Equals,
    //                 IntValue(404)
    //               )
    //             ))
    //           ))
    //         )
    //       ))
    //     ))
    //   }
    #[test]
    fn test_pipeline_stats_9() {
        assert_eq!(
            pipeline("stats count(eval(status=404))"),
            Ok((
                "",
                ast::Pipeline {
                    commands: vec![StatsCommand {
                        partitions: 1,
                        all_num: false,
                        delim: " ".to_string(),
                        funcs: vec![_call!(count(_call!(eval(_eq(
                            ast::Field::from("status"),
                            ast::IntValue(404)
                        )))))
                        .into()],
                        by: vec![],
                        dedup_split_vals: false
                    }
                    .into()],
                }
            ))
        )
    }

    //
    //   test("no-comma stats") {
    //     val query =
    //       """stats allnum=f delim=":" partitions=10 count
    //         |earliest(_time) as earliest latest(_time) as latest
    //         |values(var_2) as var_2
    //         |by var_1
    //         |""".stripMargin
    //     parses(query, stats(_), StatsCommand(
    //       partitions = 10,
    //       allNum = false,
    //       delim = ":",
    //       Seq(
    //         Call("count"),
    //         Alias(Call("earliest", Seq(Field("_time"))), "earliest"),
    //         Alias(Call("latest", Seq(Field("_time"))), "latest"),
    //         Alias(Call("values", Seq(Field("var_2"))), "var_2")
    //       ),
    //       Seq(
    //         Field("var_1")
    //       )
    //     ))
    //   }
    #[test]
    fn test_no_comma_stats() {
        assert_eq!(
            pipeline(
                r#"stats allnum=f delim=":" partitions=10 count earliest(_time) as earliest latest(_time) as latest values(var_2) as var_2 by var_1"#
            ),
            Ok((
                "",
                ast::Pipeline {
                    commands: vec![StatsCommand {
                        partitions: 10,
                        all_num: false,
                        delim: ":".to_string(),
                        funcs: vec![
                            _call!(count()).into(),
                            _alias("earliest", _call!(earliest(ast::Field::from("_time")))).into(),
                            _alias("latest", _call!(latest(ast::Field::from("_time")))).into(),
                            _alias("var_2", _call!(values(ast::Field::from("var_2")))).into(),
                        ],
                        by: vec![ast::Field::from("var_1")],
                        dedup_split_vals: false
                    }
                    .into()],
                }
                .into()
            ))
        )
    }

    //
    //   test("rex field=savedsearch_id max_match=10 " +
    //     "\"(?<user>\\w+);(?<app>\\w+);(?<SavedSearchName>\\w+)\"") {
    //     p(pipeline(_), Pipeline(Seq(
    //       RexCommand(
    //         Some("savedsearch_id"),
    //         10,
    //         None,
    //         None,
    //         "(?<user>\\w+);(?<app>\\w+);(?<SavedSearchName>\\w+)"
    //       )
    //     )))
    //   }
    #[test]
    fn test_pipeline_rex_1() {
        assert_eq!(double_quoted(r#""\d""#), Ok(("", r#"\d"#)));
        assert_eq!(
            double_quoted(r#""(?<user>\w+);(?<app>\w+);(?<SavedSearchName>\w+)""#),
            Ok(("", r#"(?<user>\w+);(?<app>\w+);(?<SavedSearchName>\w+)"#))
        );
        assert_eq!(
            pipeline(
                r#"rex field=savedsearch_id max_match=10 "(?<user>\w+);(?<app>\w+);(?<SavedSearchName>\w+)""#
            ),
            Ok((
                "",
                ast::Pipeline {
                    commands: vec![RexCommand {
                        field: "savedsearch_id".to_string(),
                        max_match: 10,
                        offset_field: None,
                        mode: None,
                        regex: "(?<user>\\w+);(?<app>\\w+);(?<SavedSearchName>\\w+)".to_string()
                    }
                    .into()],
                }
            ))
        )
    }

    //
    //   test("rex mode=sed \"s/(\\d{4}-){3}/XXXX-XXXX-XXXX-/g\"") {
    //     p(pipeline(_), Pipeline(Seq(
    //       RexCommand(
    //         None,
    //         1,
    //         None,
    //         Some("sed"),
    //         "s/(\\d{4}-){3}/XXXX-XXXX-XXXX-/g"
    //       )
    //     )))
    //   }
    #[test]
    fn test_pipeline_rex_2() {
        assert_eq!(
            pipeline(r#"rex mode=sed "s/(\d{4}-){3}/XXXX-XXXX-XXXX-/g""#),
            Ok((
                "",
                ast::Pipeline {
                    commands: vec![RexCommand {
                        field: "_raw".to_string(),
                        max_match: 1,
                        offset_field: None,
                        mode: Some("sed".to_string()),
                        regex: "s/(\\d{4}-){3}/XXXX-XXXX-XXXX-/g".to_string()
                    }
                    .into()],
                }
            ))
        )
    }

    //
    //   test("rename _ip AS IPAddress") {
    //     p(rename(_),
    //       RenameCommand(
    //         Seq(Alias(
    //           Field("_ip"),
    //           "IPAddress"
    //         )))
    //     )
    //   }
    #[test]
    fn test_rename_1() {
        assert_eq!(
            RenameParser::parse(r#"rename _ip AS IPAddress"#),
            Ok((
                "",
                RenameCommand {
                    alias: vec![_alias("IPAddress", ast::Field::from("_ip"))],
                }
                .into()
            ))
        )
    }

    //
    //   test("rename _ip AS IPAddress, _host AS host, _port AS port") {
    //     p(rename(_),
    //       RenameCommand(Seq(
    //         Alias(
    //           Field("_ip"),
    //           "IPAddress"
    //         ),
    //         Alias(
    //           Field("_host"),
    //           "host"
    //         ),
    //         Alias(
    //           Field("_port"),
    //           "port"
    //         )))
    //     )
    //   }
    #[test]
    fn test_rename_2() {
        assert_eq!(
            RenameParser::parse(r#"rename _ip AS IPAddress, _host AS host, _port AS port"#),
            Ok((
                "",
                RenameCommand {
                    alias: vec![
                        _alias("IPAddress", ast::Field::from("_ip")),
                        _alias("host", ast::Field::from("_host")),
                        _alias("port", ast::Field::from("_port")),
                    ],
                }
                .into()
            ))
        )
    }

    //
    //   // Regex not taken into account
    //   test("rename foo* AS bar*") {
    //     p(rename(_),
    //       RenameCommand(
    //         Seq(Alias(
    //           Field("foo*"),
    //           "bar*"
    //         )))
    //     )
    //   }
    #[test]
    fn test_rename_3() {
        assert_eq!(
            RenameParser::parse(r#"rename foo* AS bar*"#),
            Ok((
                "",
                RenameCommand {
                    alias: vec![_alias("bar*", ast::Field::from("foo*"))],
                }
                .into()
            ))
        )
    }

    //
    //   test("rename count AS \"Count of Events\"") {
    //     p(rename(_),
    //       RenameCommand(
    //         Seq(Alias(
    //           Field("count"),
    //           "Count of Events"
    //         )))
    //     )
    //   }
    #[test]
    fn test_rename_4() {
        assert_eq!(
            RenameParser::parse(r#"rename count AS "Count of Events""#),
            Ok((
                "",
                RenameCommand {
                    alias: vec![_alias("Count of Events", ast::Field::from("count"))],
                }
                .into()
            ))
        )
    }

    //
    //   test("join product_id [search vendors]") {
    //     p(join(_),
    //       JoinCommand(
    //         joinType = "inner",
    //         useTime = false,
    //         earlier = true,
    //         overwrite = false,
    //         max = 1,
    //         Seq(Field("product_id")),
    //         Pipeline(Seq(
    //           SearchCommand(Field("vendors"))))
    //       )
    //     )
    //   }
    #[test]
    fn test_join_1() {
        assert_eq!(
            JoinParser::parse(r#"join product_id [search vendors]"#),
            Ok((
                "",
                JoinCommand {
                    join_type: "inner".to_string(),
                    use_time: false,
                    earlier: true,
                    overwrite: false,
                    max: 1,
                    fields: vec![ast::Field::from("product_id")],
                    sub_search: ast::Pipeline {
                        commands: vec![SearchCommand {
                            expr: ast::Field::from("vendors").into()
                        }
                        .into()],
                    }
                }
                .into()
            ))
        )
    }

    //
    //   test("join type=left usetime=true earlier=false " +
    //     "overwrite=false product_id, host, name [search vendors]") {
    //     p(join(_),
    //       JoinCommand(
    //         joinType = "left",
    //         useTime = true,
    //         earlier = false,
    //         overwrite = false,
    //         max = 1,
    //         Seq(
    //           Field("product_id"),
    //           Field("host"),
    //           Field("name")
    //         ),
    //         Pipeline(Seq(
    //           SearchCommand(Field("vendors"))))
    //       )
    //     )
    //   }
    #[test]
    fn test_join_2() {
        assert_eq!(
            JoinParser::parse(
                r#"join type=left usetime=true earlier=false overwrite=false product_id, host, name [search vendors]"#
            ),
            Ok((
                "",
                JoinCommand {
                    join_type: "left".to_string(),
                    use_time: true,
                    earlier: false,
                    overwrite: false,
                    max: 1,
                    fields: vec![
                        ast::Field::from("product_id"),
                        ast::Field::from("host"),
                        ast::Field::from("name")
                    ],
                    sub_search: ast::Pipeline {
                        commands: vec![SearchCommand {
                            expr: ast::Field::from("vendors").into()
                        }
                        .into()],
                    }
                }
                .into()
            ))
        )
    }

    //
    //   test("join product_id [search vendors | rename pid AS product_id]") {
    //     p(join(_),
    //       JoinCommand(
    //         joinType = "inner",
    //         useTime = false,
    //         earlier = true,
    //         overwrite = false,
    //         max = 1,
    //         Seq(Field("product_id")),
    //         Pipeline(Seq(
    //           SearchCommand(Field("vendors")),
    //           RenameCommand(Seq(
    //             Alias(
    //               Field("pid"),
    //               "product_id"
    //             )))
    //         ))
    //       )
    //     )
    //   }
    #[test]
    fn test_join_3() {
        assert_eq!(
            JoinParser::parse(r#"join product_id [search vendors | rename pid AS product_id]"#),
            Ok((
                "",
                JoinCommand {
                    join_type: "inner".to_string(),
                    use_time: false,
                    earlier: true,
                    overwrite: false,
                    max: 1,
                    fields: vec![ast::Field::from("product_id")],
                    sub_search: ast::Pipeline {
                        commands: vec![
                            SearchCommand {
                                expr: ast::Field::from("vendors").into()
                            }
                            .into(),
                            RenameCommand {
                                alias: vec![_alias("product_id", ast::Field::from("pid"))],
                            }
                            .into()
                        ],
                    }
                    .into()
                }
                .into()
            ))
        )
    }

    //
    //   test("regex _raw=\"(?<!\\d)10\\.\\d{1,3}\\.\\d{1,3}\\.\\d{1,3}(?!\\d)\"") {
    //     p(_regex(_), RegexCommand(
    //       Some((Field("_raw"), "=")),
    //       "(?<!\\d)10\\.\\d{1,3}\\.\\d{1,3}\\.\\d{1,3}(?!\\d)"))
    //   }
    #[test]
    fn test_regex_1() {
        assert_eq!(
            RegexParser::parse(r#"regex _raw="(?<!\d)10\.\d{1,3}\.\d{1,3}\.\d{1,3}(?!\d)""#),
            Ok((
                "",
                RegexCommand {
                    item: Some((ast::Field::from("_raw"), "=".into())),
                    regex: r#"(?<!\d)10\.\d{1,3}\.\d{1,3}\.\d{1,3}(?!\d)"#.into()
                }
            ))
        )
    }

    //
    //   test("regex _raw!=\"(?<!\\d)10\\.\\d{1,3}\\.\\d{1,3}\\.\\d{1,3}(?!\\d)\"") {
    //     p(_regex(_), RegexCommand(
    //       Some((Field("_raw"), "!=")),
    //       "(?<!\\d)10\\.\\d{1,3}\\.\\d{1,3}\\.\\d{1,3}(?!\\d)"))
    //   }
    #[test]
    fn test_regex_2() {
        assert_eq!(
            RegexParser::parse(r#"regex _raw!="(?<!\d)10\.\d{1,3}\.\d{1,3}\.\d{1,3}(?!\d)""#),
            Ok((
                "",
                RegexCommand {
                    item: Some((ast::Field::from("_raw"), "!=".into())),
                    regex: r#"(?<!\d)10\.\d{1,3}\.\d{1,3}\.\d{1,3}(?!\d)"#.into()
                }
            ))
        )
    }

    //
    //   test("regex \"(?<!\\d)10\\.\\d{1,3}\\.\\d{1,3}\\.\\d{1,3}(?!\\d)\"") {
    //     p(_regex(_), RegexCommand(
    //       None,
    //       "(?<!\\d)10\\.\\d{1,3}\\.\\d{1,3}\\.\\d{1,3}(?!\\d)"))
    //   }
    #[test]
    fn test_regex_3() {
        assert_eq!(
            RegexParser::parse(r#"regex "(?<!\d)10\.\d{1,3}\.\d{1,3}\.\d{1,3}(?!\d)""#),
            Ok((
                "",
                RegexCommand {
                    item: None,
                    regex: r#"(?<!\d)10\.\d{1,3}\.\d{1,3}\.\d{1,3}(?!\d)"#.into()
                }
            ))
        )
    }

    //
    //   test("return 10 $test $env") {
    //     p(_return(_), ReturnCommand(
    //       IntValue(10),
    //       Seq(
    //         Field("test"),
    //         Field("env")
    //       )
    //     ))
    //   }
    #[test]
    fn test_return_1() {
        assert_eq!(
            ReturnParser::parse(r#"return 10 $test $env"#),
            Ok((
                "",
                ReturnCommand {
                    count: ast::IntValue(10),
                    fields: vec![
                        ast::Field::from("test").into(),
                        ast::Field::from("env").into(),
                    ],
                }
            ))
        )
    }

    //
    //   test("return 10 ip src host port") {
    //     p(_return(_), ReturnCommand(
    //       IntValue(10),
    //       Seq(
    //         Field("ip"),
    //         Field("src"),
    //         Field("host"),
    //         Field("port")
    //       )
    //     ))
    //   }
    #[test]
    fn test_return_2() {
        assert_eq!(
            ReturnParser::parse(r#"return 10 ip src host port"#),
            Ok((
                "",
                ReturnCommand {
                    count: ast::IntValue(10),
                    fields: vec![
                        ast::Field::from("ip").into(),
                        ast::Field::from("src").into(),
                        ast::Field::from("host").into(),
                        ast::Field::from("port").into(),
                    ],
                }
            ))
        )
    }

    //
    //   test("return 10 ip=src host=port") {
    //     p(_return(_), ReturnCommand(
    //       IntValue(10),
    //       Seq(
    //         Alias(Field("src"), "ip"),
    //         Alias(Field("port"), "host")
    //       )
    //     ))
    //   }
    #[test]
    fn test_return_3() {
        assert_eq!(
            ReturnParser::parse(r#"return 10 ip=src host=port"#),
            Ok((
                "",
                ReturnCommand {
                    count: ast::IntValue(10),
                    fields: vec![
                        _alias("ip", ast::Field::from("src")).into(),
                        _alias("host", ast::Field::from("port")).into(),
                    ],
                }
            ))
        )
    }

    //
    //   test("fillnull") {
    //     p(fillNull(_), FillNullCommand(None, None))
    //   }
    #[test]
    fn test_fill_null_1() {
        assert_eq!(
            FillNullParser::parse(r#"fillnull"#),
            Ok((
                "",
                FillNullCommand {
                    value: None,
                    fields: None,
                }
            ))
        )
    }

    //
    //   test("fillnull value=NA") {
    //     p(fillNull(_), FillNullCommand(Some("NA"), None))
    //   }
    #[test]
    fn test_fill_null_2() {
        assert_eq!(
            FillNullParser::parse(r#"fillnull value="NA""#),
            Ok((
                "",
                FillNullCommand {
                    value: Some("NA".into()),
                    fields: None,
                }
            ))
        )
    }

    //
    //   test("fillnull value=\"NULL\" host port") {
    //     p(fillNull(_), FillNullCommand(
    //       Some("NULL"),
    //       Some(Seq(
    //         Field("host"),
    //         Field("port")
    //       ))))
    //   }
    #[test]
    fn test_fill_null_3() {
        assert_eq!(
            FillNullParser::parse(r#"fillnull value="NULL" host port"#),
            Ok((
                "",
                FillNullCommand {
                    value: Some("NULL".into()),
                    fields: Some(vec![ast::Field::from("host"), ast::Field::from("port"),].into()),
                }
                .into()
            ))
        )
    }

    //
    //   test("dedup 10 keepevents=true keepempty=false consecutive=true host ip port") {
    //     p(dedup(_), DedupCommand(
    //       10,
    //       Seq(
    //         Field("host"),
    //         Field("ip"),
    //         Field("port")
    //       ),
    //       keepEvents = true,
    //       keepEmpty = false,
    //       consecutive = true,
    //       SortCommand(Seq((Some("+"), Field("_no"))))
    //     ))
    //   }
    #[test]
    fn test_dedup_1() {
        assert_eq!(
            DedupParser::parse(
                r#"dedup 10 keepevents=true keepempty=false consecutive=true host ip port"#
            ),
            Ok((
                "",
                DedupCommand {
                    num_results: 10,
                    fields: vec![
                        ast::Field::from("host"),
                        ast::Field::from("ip"),
                        ast::Field::from("port"),
                    ],
                    keep_events: true,
                    keep_empty: false,
                    consecutive: true,
                    sort_by: SortCommand::new_simple(vec![(
                        Some("+".into()),
                        ast::Field::from("_no").into()
                    )]),
                }
                .into()
            ))
        )
    }

    //
    //   test("dedup 10 keepevents=true host ip port sortby +host -ip") {
    //     p(dedup(_), DedupCommand(
    //       10,
    //       Seq(
    //         Field("host"),
    //         Field("ip"),
    //         Field("port")
    //       ),
    //       keepEvents = true,
    //       keepEmpty = false,
    //       consecutive = false,
    //       SortCommand(
    //         Seq(
    //           (Some("+"), Field("host")),
    //           (Some("-"), Field("ip"))
    //         )
    //       )
    //     ))
    //   }
    #[test]
    fn test_dedup_2() {
        assert_eq!(
            DedupParser::parse(r#"dedup 10 keepevents=true host ip port sortby +host -ip"#),
            Ok((
                "",
                DedupCommand {
                    num_results: 10,
                    fields: vec![
                        ast::Field::from("host"),
                        ast::Field::from("ip"),
                        ast::Field::from("port"),
                    ],
                    keep_events: true,
                    keep_empty: false,
                    consecutive: false,
                    sort_by: SortCommand::new_simple(vec![
                        (Some("+".into()), ast::Field::from("host").into()),
                        (Some("-".into()), ast::Field::from("ip").into()),
                    ]),
                }
                .into()
            ))
        )
    }

    //
    //   test("inputlookup append=t strict=f myTable where test_id=11") {
    //     p(inputLookup(_), InputLookup(
    //       append = true,
    //       strict = false,
    //       start = 0,
    //       max = 1000000000,
    //       "myTable",
    //       Some(
    //         Binary(
    //           Field("test_id"),
    //           Equals,
    //           IntValue(11)
    //         )
    //     )))
    //   }
    #[test]
    fn test_input_lookup_1() {
        assert_eq!(
            InputLookupParser::parse(r#"inputlookup append=t strict=f myTable where test_id=11"#),
            Ok((
                "",
                InputLookup {
                    append: true,
                    strict: false,
                    start: 0,
                    max: 1000000000,
                    table_name: "myTable".into(),
                    where_expr: Some(_eq(ast::Field::from("test_id"), ast::IntValue(11),)),
                }
            ))
        )
    }

    //
    //   test("inputlookup myTable") {
    //     p(inputLookup(_), InputLookup(
    //       append = false,
    //       strict = false,
    //       start = 0,
    //       max = 1000000000,
    //       "myTable",
    //       None
    //     ))
    //   }
    #[test]
    fn test_input_lookup_2() {
        assert_eq!(
            InputLookupParser::parse(r#"inputlookup myTable"#),
            Ok((
                "",
                InputLookup {
                    append: false,
                    strict: false,
                    start: 0,
                    max: 1000000000,
                    table_name: "myTable".into(),
                    where_expr: None,
                }
            ))
        )
    }

    //
    //   test("format maxresults=10") {
    //     p(format(_), FormatCommand(
    //       mvSep = "OR",
    //       maxResults = 10,
    //       rowPrefix = "(",
    //       colPrefix =  "(",
    //       colSep = "AND",
    //       colEnd = ")",
    //       rowSep = "OR",
    //       rowEnd = ")"
    //     ))
    //   }
    #[test]
    fn test_format_1() {
        assert_eq!(
            FormatParser::parse(r#"format maxresults=10"#),
            Ok((
                "",
                FormatCommand {
                    mv_sep: "OR".into(),
                    max_results: 10,
                    row_prefix: "(".into(),
                    col_prefix: "(".into(),
                    col_sep: "AND".into(),
                    col_end: ")".into(),
                    row_sep: "OR".into(),
                    row_end: ")".into(),
                }
            ))
        )
    }

    //
    //   test("format mvsep=\"||\" \"[\" \"[\" \"&&\" \"]\" \"||\" \"]\"") {
    //     p(format(_), FormatCommand(
    //       mvSep = "||",
    //       maxResults = 0,
    //       rowPrefix = "[",
    //       colPrefix = "[",
    //       colSep = "&&",
    //       colEnd = "]",
    //       rowSep = "||",
    //       rowEnd = "]"
    //     ))
    //   }
    #[test]
    fn test_format_2() {
        assert_eq!(
            FormatParser::parse(r#"format mvsep="||" "[" "[" "&&" "]" "||" "]""#),
            Ok((
                "",
                FormatCommand {
                    mv_sep: "||".into(),
                    max_results: 0,
                    row_prefix: "[".into(),
                    col_prefix: "[".into(),
                    col_sep: "&&".into(),
                    col_end: "]".into(),
                    row_sep: "||".into(),
                    row_end: "]".into(),
                }
            ))
        )
    }

    //
    //   test("mvcombine host") {
    //     p(mvcombine(_), MvCombineCommand(
    //       None,
    //       Field("host")
    //     ))
    //   }
    #[test]
    fn test_mvcombine_1() {
        assert_eq!(
            MvCombineParser::parse(r#"mvcombine host"#),
            Ok((
                "",
                MvCombineCommand {
                    delim: None,
                    field: ast::Field::from("host"),
                }
            ))
        )
    }

    //
    //   test("mvcombine delim=\",\" host") {
    //     p(mvcombine(_), MvCombineCommand(
    //       Some(","),
    //       Field("host")
    //     ))
    //   }
    #[test]
    fn test_mvcombine_2() {
        assert_eq!(
            MvCombineParser::parse(r#"mvcombine delim="," host"#),
            Ok((
                "",
                MvCombineCommand {
                    delim: Some(",".into()),
                    field: ast::Field::from("host"),
                }
            ))
        )
    }

    //
    //   test("bin span=30m minspan=5m bins=20 start=0 end=20 aligntime=latest foo AS bar") {
    //     p(command(_), BinCommand(
    //       Alias(Field("foo"), "bar"),
    //       Some(TimeSpan(30, "minutes")),
    //       Some(TimeSpan(5, "minutes")),
    //       Some(20),
    //       Some(0),
    //       Some(20),
    //       Some("latest")))
    //   }
    #[test]
    fn test_command_bin_1() {
        assert_eq!(
            command(
                r#"bin span=30m minspan=5m bins=20 start=0 end=20 aligntime=latest foo AS bar"#
            ),
            Ok((
                "",
                BinCommand {
                    field: _alias("bar", ast::Field::from("foo")).into(),
                    span: Some(ast::TimeSpan {
                        value: 30,
                        scale: "minutes".into()
                    }),
                    min_span: Some(ast::TimeSpan {
                        value: 5,
                        scale: "minutes".into()
                    }),
                    bins: Some(20),
                    start: Some(0),
                    end: Some(20),
                    align_time: Some("latest".into()),
                }
                .into()
            ))
        )
    }

    //
    //   test("makeresults") {
    //     p(command(_), MakeResults(
    //       count = 1,
    //       annotate = false,
    //       server = "local",
    //       serverGroup = null))
    //   }
    #[test]
    fn test_command_makeresults_1() {
        assert_eq!(
            command(r#"makeresults"#),
            Ok((
                "",
                MakeResults {
                    count: 1,
                    annotate: false,
                    server: "local".into(),
                    server_group: None,
                }
                .into()
            ))
        )
    }

    //
    //   test("makeresults count=10 annotate=t splunk_server_group=group0") {
    //     p(command(_), MakeResults(
    //       count = 10,
    //       annotate = true,
    //       server = "local",
    //       serverGroup = "group0"))
    //   }
    #[test]
    fn test_command_makeresults_2() {
        assert_eq!(
            command(r#"makeresults count=10 annotate=t splunk_server_group=group0"#),
            Ok((
                "",
                MakeResults {
                    count: 10,
                    annotate: true,
                    server: "local".into(),
                    server_group: Some("group0".into()),
                }
                .into()
            ))
        )
    }

    //
    //   test("addtotals row=t col=f fieldname=num_total num_1 num_2") {
    //     p(command(_), AddTotals(
    //       fields = Seq(Field("num_1"), Field("num_2")),
    //       row = true,
    //       col = false,
    //       fieldName = "num_total",
    //       labelField = null,
    //       label = "Total"
    //     ))
    //   }
    #[test]
    fn test_command_addtotals_1() {
        assert_eq!(
            command(r#"addtotals row=t col=f fieldname=num_total num_1 num_2"#),
            Ok((
                "",
                AddTotals {
                    fields: vec![ast::Field::from("num_1"), ast::Field::from("num_2")],
                    row: true,
                    col: false,
                    field_name: "num_total".into(),
                    label_field: None,
                    label: "Total".into(),
                }
                .into()
            ))
        )
    }

    //
    //   test("eventstats min(n) by gender") {
    //     p(command(_), EventStatsCommand(
    //       allNum = false,
    //       funcs = Seq(
    //         Call("min", Seq(Field("n")))
    //       ),
    //       by = Seq(Field("gender"))
    //     ))
    //   }
    #[test]
    fn test_command_eventstats_1() {
        assert_eq!(
            command(r#"eventstats min(n) by gender"#),
            Ok((
                "",
                EventStatsCommand {
                    all_num: false,
                    funcs: vec![_call!(min(ast::Field::from("n"))).into()],
                    by: vec![ast::Field::from("gender")],
                }
                .into()
            ))
        )
    }

    //
    //   test("map search=\"search index=dummy host=$host_var$\" maxsearches=20") {
    //     p(command(_), MapCommand(
    //       Pipeline(
    //         Seq(
    //           SearchCommand(
    //             Binary(
    //               Binary(
    //                 Field("index"),
    //                 Equals,
    //                 Field("dummy")
    //               ),
    //               And,
    //               Binary(
    //                 Field("host"),
    //                 Equals,
    //                 Variable("host_var")
    //               )
    //             )
    //           )
    //         )
    //       ),
    //       maxSearches = 20))
    //   }
    #[test]
    fn test_command_map_1() {
        assert_eq!(
            command(r#"map search="search index=dummy host=$host_var$" maxsearches=20"#),
            Ok((
                "",
                MapCommand {
                    search: ast::Pipeline {
                        commands: vec![SearchCommand {
                            expr: _and(
                                _eq(ast::Field::from("index"), ast::Field::from("dummy"),),
                                _eq(ast::Field::from("host"), ast::Variable::from("host_var"),)
                            )
                        }
                        .into()]
                    },
                    max_searches: 20,
                }
                .into()
            ))
        )
    }

    //
    //   test(
    //     """map search="search index=dummy host=$host_var$ | eval this=\"that\" |
    //       |dedup 10 keepevents=true keepempty=false consecutive=true host ip port"""".stripMargin) {
    //     p(_map(_), MapCommand(
    //       Pipeline(
    //         Seq(
    //           SearchCommand(
    //             Binary(
    //               Binary(
    //                 Field("index"),
    //                 Equals,
    //                 Field("dummy")
    //               ),
    //               And,
    //               Binary(
    //                 Field("host"),
    //                 Equals,
    //                 Variable("host_var")
    //               )
    //             )
    //           ),
    //           EvalCommand(Seq(
    //             (Field("this"), StrValue("that"))
    //           )),
    //           DedupCommand(10,
    //             Seq(Field("host"), Field("ip"), Field("port")),
    //             keepEvents = true,
    //             keepEmpty = false,
    //             consecutive = true,
    //             SortCommand(Seq(
    //               (Some("+"), Field("_no"))
    //             ))
    //           )
    //         )
    //       ),
    //       maxSearches = 10))
    //   }
    #[test]
    fn test_quoted_search() {
        assert_eq!(
            double_quoted_alt(r#""search index=\"dummy\"""#),
            Ok(("", r#"search index=\"dummy\""#))
        );
        assert_eq!(
            quoted_search(r#"search="search index=\"dummy\"""#),
            Ok((
                "",
                ast::Pipeline {
                    commands: vec![SearchCommand {
                        expr: _eq(ast::Field::from("index"), ast::StrValue::from("dummy"))
                    }
                    .into()],
                }
            ))
        );
        assert_eq!(
            MapParser::parse(r#"map search="search index=\"dummy\"""#),
            Ok((
                "",
                MapCommand {
                    search: ast::Pipeline {
                        commands: vec![SearchCommand {
                            expr: _eq(ast::Field::from("index"), ast::StrValue::from("dummy"))
                        }
                        .into()],
                    },
                    max_searches: 10,
                }
            ))
        );
        assert_eq!(
            MapParser::parse(r#"map search="search index=dummy""#),
            Ok((
                "",
                MapCommand {
                    search: ast::Pipeline {
                        commands: vec![SearchCommand {
                            expr: _eq(ast::Field::from("index"), ast::Field::from("dummy"))
                        }
                        .into()],
                    },
                    max_searches: 10,
                }
            ))
        );
    }

    #[test]
    fn test_map_1() {
        let s = r#"map search="search index=dummy host=$host_var$ | eval this=\"that\" | dedup 10 keepevents=true keepempty=false consecutive=true host ip port""#;
        let _pipeline = ast::Pipeline {
            commands: vec![
                SearchCommand {
                    expr: _and(
                        _eq(ast::Field::from("index"), ast::Field::from("dummy")),
                        _eq(ast::Field::from("host"), ast::Variable::from("host_var")),
                    ),
                }
                .into(),
                EvalCommand {
                    fields: vec![(ast::Field::from("this"), ast::StrValue::from("that").into())],
                }
                .into(),
                DedupCommand {
                    num_results: 10,
                    fields: vec![
                        ast::Field::from("host"),
                        ast::Field::from("ip"),
                        ast::Field::from("port"),
                    ],
                    keep_events: true,
                    keep_empty: false,
                    consecutive: true,
                    sort_by: SortCommand::new_simple(vec![(
                        Some("+".into()),
                        ast::Field::from("_no").into(),
                    )]),
                }
                .into(),
            ],
        };
        assert_eq!(
            pipeline(
                r#"search index=dummy host=$host_var$ | eval this="that" | dedup 10 keepevents=true keepempty=false consecutive=true host ip port"#
            ),
            Ok(("", _pipeline.clone()))
        );
        assert!(quoted_search(r#"search="search index=dummy host=$host_var$ | eval this=\"that\" | dedup 10 keepevents=true keepempty=false consecutive=true host ip port""#).is_ok());
        assert_eq!(
            command(s),
            MapParser::parse(s).map(|(remaining, result)| (remaining, result.into()))
        );
        assert_eq!(
            command(s),
            Ok((
                "",
                MapCommand {
                    search: _pipeline,
                    max_searches: 10,
                }
                .into()
            ))
        )
    }
}
