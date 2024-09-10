use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case};
use nom::IResult;

pub trait OperatorSymbolTrait {
    const SYMBOL: &'static str;
    const PRECEDENCE: i32;
    fn pattern(input: &str) -> IResult<&str, &str> {
        tag_no_case(Self::SYMBOL)(input)
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct UnaryNot;
impl OperatorSymbolTrait for UnaryNot {
    const SYMBOL: &'static str = "NOT";
    const PRECEDENCE: i32 = 2;
}
#[derive(Debug, PartialEq, Clone)]
pub struct Or;
impl OperatorSymbolTrait for Or {
    const SYMBOL: &'static str = "OR";
    const PRECEDENCE: i32 = 9;
}
#[derive(Debug, PartialEq, Clone)]
pub struct And;
impl OperatorSymbolTrait for And {
    const SYMBOL: &'static str = "AND";
    const PRECEDENCE: i32 = 8;
    fn pattern(input: &str) -> IResult<&str, &str> {
        alt((tag_no_case("AND"), tag(" ")))(input)
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct InList;
impl OperatorSymbolTrait for InList {
    const SYMBOL: &'static str = "IN";
    const PRECEDENCE: i32 = 7;

    fn pattern(input: &str) -> IResult<&str, &str> {
        tag_no_case(" IN")(input)
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct Concatenate;
impl OperatorSymbolTrait for Concatenate {
    const SYMBOL: &'static str = ".";
    const PRECEDENCE: i32 = 5;
}
#[derive(Debug, PartialEq, Clone)]
pub struct Add;
impl OperatorSymbolTrait for Add {
    const SYMBOL: &'static str = "+";
    const PRECEDENCE: i32 = 4;
}
#[derive(Debug, PartialEq, Clone)]
pub struct Subtract;
impl OperatorSymbolTrait for Subtract {
    const SYMBOL: &'static str = "-";
    const PRECEDENCE: i32 = 4;
}
#[derive(Debug, PartialEq, Clone)]
pub struct Multiply;
impl OperatorSymbolTrait for Multiply {
    const SYMBOL: &'static str = "*";
    const PRECEDENCE: i32 = 3;
}
#[derive(Debug, PartialEq, Clone)]
pub struct Divide;
impl OperatorSymbolTrait for Divide {
    const SYMBOL: &'static str = "/";
    const PRECEDENCE: i32 = 3;
}
#[derive(Debug, PartialEq, Clone)]
pub struct LessThan;
impl OperatorSymbolTrait for LessThan {
    const SYMBOL: &'static str = "<";
    const PRECEDENCE: i32 = 7;
}
#[derive(Debug, PartialEq, Clone)]
pub struct GreaterThan;
impl OperatorSymbolTrait for GreaterThan {
    const SYMBOL: &'static str = ">";
    const PRECEDENCE: i32 = 7;
}
#[derive(Debug, PartialEq, Clone)]
pub struct GreaterEquals;
impl OperatorSymbolTrait for GreaterEquals {
    const SYMBOL: &'static str = ">=";
    const PRECEDENCE: i32 = 7;
}
#[derive(Debug, PartialEq, Clone)]
pub struct LessEquals;
impl OperatorSymbolTrait for LessEquals {
    const SYMBOL: &'static str = "<=";
    const PRECEDENCE: i32 = 7;
}
#[derive(Debug, PartialEq, Clone)]
pub struct Equals;
impl OperatorSymbolTrait for Equals {
    const SYMBOL: &'static str = "=";
    const PRECEDENCE: i32 = 7;
    fn pattern(input: &str) -> IResult<&str, &str> {
        alt((tag("="), tag("::")))(input)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct NotEquals;
impl OperatorSymbolTrait for NotEquals {
    const SYMBOL: &'static str = "!=";
    const PRECEDENCE: i32 = 7;
}

#[derive(Debug, PartialEq, Clone)]
pub enum OperatorSymbol {
    #[allow(dead_code)]
    UnaryNot(UnaryNot),
    Or(Or),
    And(And),
    InList(InList),
    Concatenate(Concatenate),
    Add(Add),
    Subtract(Subtract),
    Multiply(Multiply),
    Divide(Divide),
    LessThan(LessThan),
    GreaterThan(GreaterThan),
    GreaterEquals(GreaterEquals),
    LessEquals(LessEquals),
    Equals(Equals),
    NotEquals(NotEquals),
}

impl OperatorSymbol {
    // pub fn pattern<F>(self) -> F where F: Fn(&str) -> IResult<&str, &str> {
    //     match self {
    //         OperatorSymbol::UnaryNot(op) => op.pattern,
    //         OperatorSymbol::Or => tag_no_case("OR"),
    //         OperatorSymbol::And => alt((tag_no_case("AND"), tag(" "))),
    //         OperatorSymbol::InList => tag_no_case(" IN"),
    //         OperatorSymbol::Concatenate => tag("."),
    //         OperatorSymbol::Add => tag("+"),
    //         OperatorSymbol::Subtract => tag("-"),
    //         OperatorSymbol::Multiply => tag("*"),
    //         OperatorSymbol::Divide => tag("/"),
    //         OperatorSymbol::LessThan => tag("<"),
    //         OperatorSymbol::GreaterThan => tag(">"),
    //         OperatorSymbol::GreaterEquals => tag(">="),
    //         OperatorSymbol::LessEquals => tag("<="),
    //         OperatorSymbol::Equals => alt((tag("="), tag("::"))),
    //         OperatorSymbol::NotEquals => tag("!="),
    //     }
    // }
    pub fn symbol_string(&self) -> &'static str {
        match self {
            OperatorSymbol::UnaryNot(_) => UnaryNot::SYMBOL,
            OperatorSymbol::Or(_) => Or::SYMBOL,
            OperatorSymbol::And(_) => And::SYMBOL,
            OperatorSymbol::InList(_) => InList::SYMBOL,
            OperatorSymbol::Concatenate(_) => Concatenate::SYMBOL,
            OperatorSymbol::Add(_) => Add::SYMBOL,
            OperatorSymbol::Subtract(_) => Subtract::SYMBOL,
            OperatorSymbol::Multiply(_) => Multiply::SYMBOL,
            OperatorSymbol::Divide(_) => Divide::SYMBOL,
            OperatorSymbol::LessThan(_) => LessThan::SYMBOL,
            OperatorSymbol::GreaterThan(_) => GreaterThan::SYMBOL,
            OperatorSymbol::GreaterEquals(_) => GreaterEquals::SYMBOL,
            OperatorSymbol::LessEquals(_) => LessEquals::SYMBOL,
            OperatorSymbol::Equals(_) => Equals::SYMBOL,
            OperatorSymbol::NotEquals(_) => NotEquals::SYMBOL,
        }
    }
    pub fn precedence(&self) -> i32 {
        match self {
            OperatorSymbol::UnaryNot(_) => UnaryNot::PRECEDENCE,
            OperatorSymbol::Or(_) => Or::PRECEDENCE,
            OperatorSymbol::And(_) => And::PRECEDENCE,
            OperatorSymbol::InList(_) => InList::PRECEDENCE,
            OperatorSymbol::Concatenate(_) => Concatenate::PRECEDENCE,
            OperatorSymbol::Add(_) => Add::PRECEDENCE,
            OperatorSymbol::Subtract(_) => Subtract::PRECEDENCE,
            OperatorSymbol::Multiply(_) => Multiply::PRECEDENCE,
            OperatorSymbol::Divide(_) => Divide::PRECEDENCE,
            OperatorSymbol::LessThan(_) => LessThan::PRECEDENCE,
            OperatorSymbol::GreaterThan(_) => GreaterThan::PRECEDENCE,
            OperatorSymbol::GreaterEquals(_) => GreaterEquals::PRECEDENCE,
            OperatorSymbol::LessEquals(_) => LessEquals::PRECEDENCE,
            OperatorSymbol::Equals(_) => Equals::PRECEDENCE,
            OperatorSymbol::NotEquals(_) => NotEquals::PRECEDENCE,
        }
    }
}
