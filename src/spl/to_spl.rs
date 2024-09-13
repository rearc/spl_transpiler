// use crate::spl::ast;
//
// pub trait ToSpl {
//     fn to_spl(&self) -> String;
// }
//
// impl ToSpl for ast::Expr {
//     fn to_spl(&self) -> String {
//         match self {
//             ast::Expr::Leaf(obj) => (*obj).to_spl(),
//             ast::Expr::AliasedField(obj) => (*obj).to_spl(),
//             ast::Expr::Binary(obj) => (*obj).to_spl(),
//             ast::Expr::Unary(obj) => (*obj).to_spl(),
//             ast::Expr::Call(obj) => (*obj).to_spl(),
//             ast::Expr::FieldIn(obj) => (*obj).to_spl(),
//             ast::Expr::Alias(obj) => (*obj).to_spl(),
//             ast::Expr::FieldConversion(obj) => (*obj).to_spl(),
//         }
//     }
// }
//
// impl ToSpl for ast::LeafExpr {
//     fn to_spl(&self) -> String {
//         match self {
//             ast::LeafExpr::Constant(obj) => (*obj).to_spl(),
//             ast::LeafExpr::FV(obj) => (*obj).to_spl(),
//             ast::LeafExpr::FB(obj) => (*obj).to_spl(),
//             ast::LeafExpr::FC(obj) => (*obj).to_spl(),
//         }
//     }
// }
//
// impl ToSpl for ast::Constant {
//     fn to_spl(&self) -> String {
//         match self {
//             ast::Constant::Null(obj) => (*obj).to_spl(),
//             ast::Constant::Bool(obj) => (*obj).to_spl(),
//             ast::Constant::Int(obj) => (*obj).to_spl(),
//             ast::Constant::Double(obj) => (*obj).to_spl(),
//             ast::Constant::Str(obj) => (*obj).to_spl(),
//             ast::Constant::SnapTime(obj) => (*obj).to_spl(),
//             ast::Constant::SplSpan(obj) => (*obj).to_spl(),
//             ast::Constant::Field(obj) => (*obj).to_spl(),
//             ast::Constant::Wildcard(obj) => (*obj).to_spl(),
//             ast::Constant::Variable(obj) => (*obj).to_spl(),
//             ast::Constant::IPv4CIDR(obj) => (*obj).to_spl(),
//         }
//     }
// }
//
// impl ToSpl for ast::SplSpan {
//     fn to_spl(&self) -> String {
//         match self {
//             ast::SplSpan::TimeSpan(obj) => (*obj).to_spl(),
//         }
//     }
// }
//
// impl ToSpl for ast::NullValue {
//     fn to_spl(&self) -> String {
//         "null".to_string()
//     }
// }
//
// impl ToSpl for ast::BoolValue {
//     fn to_spl(&self) -> String {
//         if self.0 { "true".to_string() } else { "false".to_string() }
//     }
// }
//
// impl ToSpl for ast::IntValue {
//     fn to_spl(&self) -> String {
//         format!("{}", self.0).to_string()
//     }
// }
//
// impl ToSpl for ast::DoubleValue {
//     fn to_spl(&self) -> String {
//         format!("{}", self.0).to_string()
//     }
// }
//
// impl ToSpl for ast::StrValue {
//     fn to_spl(&self) -> String {
//         format!("\"{}\"", self.0).to_string()
//     }
// }
//
// impl ToSpl for ast::SnapTime {
//     fn to_spl(&self) -> String {
//         let span = match self.span.clone() {
//             Some(span) => span.to_spl(),
//             None => "".to_string(),
//         };
//         let snap_offset = match self.snap_offset.clone() {
//             Some(offset) => offset.to_spl(),
//             None => "".to_string(),
//         };
//         format!("{}@{}{}", span, self.snap, snap_offset)
//     }
// }
//
// impl ToSpl for ast::TimeSpan {
//     fn to_spl(&self) -> String {
//         let sign = if self.value >= 0 { "+" } else { "-" };
//         format!("{}{}{}", sign, self.value, self.scale)
//     }
// }
//
// impl ToSpl for ast::Field {
//     fn to_spl(&self) -> String {
//         format!("\"{}\"", self.0).to_string()
//     }
// }
//
// impl ToSpl for ast::Wildcard {
//     fn to_spl(&self) -> String {
//         format!("\"{}\"", self.0).to_string()
//     }
// }
//
// impl ToSpl for ast::Variable {
//     fn to_spl(&self) -> String {
//         format!("\"{}\"", self.0).to_string()
//     }
// }
//
// impl ToSpl for ast::IPv4CIDR {
//     fn to_spl(&self) -> String {
//         format!("\"{}\"", self.0).to_string()
//     }
// }
//
// impl ToSpl for ast::FV {
//     fn to_spl(&self) -> String {
//         format!("{}={}", self.field, self.value).to_string()
//     }
// }
//
// impl ToSpl for ast::FB {
//     fn to_spl(&self) -> String {
//         format!("{}={}", self.field, if self.value { "true" } else { "false" }).to_string()
//     }
// }
//
// impl ToSpl for ast::FC {
//     fn to_spl(&self) -> String {
//         format!("{}={}", self.field, self.value.to_spl()).to_string()
//     }
// }
//
// impl ToSpl for ast::AliasedField {
//     fn to_spl(&self) -> String {
//         format!("{} AS {}", self.field.to_spl(), self.alias)
//     }
// }
//
// impl ToSpl for ast::Binary {
//     fn to_spl(&self) -> String {
//         format!("({}) {} ({})", self.left.to_spl(), self.symbol, self.right.to_spl())
//     }
// }
//
// impl ToSpl for ast::Unary {
//     fn to_spl(&self) -> String {
//         format!("{}({})", self.symbol, self.right.to_spl()).to_string()
//     }
// }
//
// impl ToSpl for ast::Call {
//     fn to_spl(&self) -> String {
//         let args: Vec<_> = self.args.iter().map(|e| e.to_spl()).collect();
//         format!("{}({})", self.name, args.join(", ")).to_string()
//     }
// }
//
// impl ToSpl for ast::FieldIn {
//     fn to_spl(&self) -> String {
//         let args: Vec<_> = self.exprs.iter().map(|e| e.to_spl()).collect();
//         format!("{} IN ({})", self.field, args.join(", ")).to_string()
//     }
// }
//
// impl ToSpl for ast::Alias {
//     fn to_spl(&self) -> String {
//         format!("({}) AS {}", self.expr.to_spl(), self.name).to_string()
//     }
// }
//
// impl ToSpl for ast::FieldConversion {
//     fn to_spl(&self) -> String {
//         let alias = match self.alias {
//             Some(ref alias) => format!(" AS {}", alias),
//             None => String::new(),
//         };
//         format!("{}({}){}", self.func, self.field.to_spl(), alias)
//     }
// }
