use crate::spl::ast::{
    Alias, AliasedField, Binary, BoolValue, Call, Constant, DoubleValue, Expr, Field, FieldIn,
    FieldLike, FieldOrAlias, FormattedTimeModifier, IPv4CIDR, IntValue, LeafExpr, SearchModifier,
    SnapTime, StrValue, TimeSpan, Unary, Variable, Wildcard, FB, FC, FV,
};

impl<T: Into<bool>> From<T> for BoolValue {
    fn from(value: T) -> Self {
        BoolValue(value.into())
    }
}

impl<T: Into<i64>> From<T> for IntValue {
    fn from(value: T) -> Self {
        IntValue(value.into())
    }
}

impl<T: Into<f64>> From<T> for DoubleValue {
    fn from(value: T) -> Self {
        DoubleValue(value.into())
    }
}

impl<T: ToString> From<T> for StrValue {
    fn from(value: T) -> Self {
        StrValue(value.to_string())
    }
}

impl<S: ToString> From<S> for Field {
    fn from(value: S) -> Field {
        Field(value.to_string())
    }
}

impl<S: ToString> From<S> for Wildcard {
    fn from(value: S) -> Wildcard {
        Wildcard(value.to_string())
    }
}

impl<S: ToString> From<S> for Variable {
    fn from(value: S) -> Variable {
        Variable(value.to_string())
    }
}

impl<S: ToString> From<S> for IPv4CIDR {
    fn from(value: S) -> IPv4CIDR {
        IPv4CIDR(value.to_string())
    }
}

impl From<AliasedField> for Alias {
    fn from(value: AliasedField) -> Self {
        Alias {
            expr: Box::new(value.field.into()),
            name: value.alias,
        }
    }
}

impl From<TimeSpan> for Constant {
    fn from(val: TimeSpan) -> Self {
        Constant::TimeSpan(val)
    }
}

impl From<BoolValue> for Constant {
    fn from(val: BoolValue) -> Self {
        Constant::Bool(val)
    }
}

impl From<IntValue> for Constant {
    fn from(val: IntValue) -> Self {
        Constant::Int(val)
    }
}

impl From<DoubleValue> for Constant {
    fn from(val: DoubleValue) -> Self {
        Constant::Double(val)
    }
}

impl From<StrValue> for Constant {
    fn from(val: StrValue) -> Self {
        Constant::Str(val)
    }
}

impl From<SnapTime> for Constant {
    fn from(val: SnapTime) -> Self {
        Constant::SnapTime(val)
    }
}

impl From<Field> for Constant {
    fn from(val: Field) -> Self {
        Constant::Field(val)
    }
}

impl From<Wildcard> for Constant {
    fn from(val: Wildcard) -> Self {
        Constant::Wildcard(val)
    }
}

impl From<Variable> for Constant {
    fn from(val: Variable) -> Self {
        Constant::Variable(val)
    }
}

impl From<IPv4CIDR> for Constant {
    fn from(val: IPv4CIDR) -> Self {
        Constant::IPv4CIDR(val)
    }
}

impl From<Constant> for Expr {
    fn from(val: Constant) -> Self {
        Expr::Leaf(LeafExpr::Constant(val))
    }
}

impl From<TimeSpan> for Expr {
    fn from(val: TimeSpan) -> Self {
        <TimeSpan as Into<Constant>>::into(val).into()
    }
}

impl From<BoolValue> for Expr {
    fn from(val: BoolValue) -> Self {
        <BoolValue as Into<Constant>>::into(val).into()
    }
}

impl From<IntValue> for Expr {
    fn from(val: IntValue) -> Self {
        Expr::Leaf(LeafExpr::Constant(val.into()))
    }
}

impl From<DoubleValue> for Expr {
    fn from(val: DoubleValue) -> Self {
        Expr::Leaf(LeafExpr::Constant(val.into()))
    }
}

impl From<StrValue> for Expr {
    fn from(val: StrValue) -> Self {
        Expr::Leaf(LeafExpr::Constant(val.into()))
    }
}

impl From<SnapTime> for Expr {
    fn from(val: SnapTime) -> Self {
        Expr::Leaf(LeafExpr::Constant(val.into()))
    }
}

impl From<Field> for Expr {
    fn from(val: Field) -> Self {
        Expr::Leaf(LeafExpr::Constant(val.into()))
    }
}

impl From<Wildcard> for Expr {
    fn from(val: Wildcard) -> Self {
        Expr::Leaf(LeafExpr::Constant(val.into()))
    }
}

impl From<Variable> for Expr {
    fn from(val: Variable) -> Self {
        Expr::Leaf(LeafExpr::Constant(val.into()))
    }
}

impl From<IPv4CIDR> for Expr {
    fn from(val: IPv4CIDR) -> Self {
        Expr::Leaf(LeafExpr::Constant(val.into()))
    }
}

impl From<FV> for Expr {
    fn from(val: FV) -> Self {
        Expr::Leaf(LeafExpr::FV(val))
    }
}

impl From<FB> for Expr {
    fn from(val: FB) -> Self {
        Expr::Leaf(LeafExpr::FB(val))
    }
}

impl From<FC> for Expr {
    fn from(val: FC) -> Self {
        Expr::Leaf(LeafExpr::FC(val))
    }
}

impl From<AliasedField> for Expr {
    fn from(val: AliasedField) -> Self {
        Expr::AliasedField(val)
    }
}

impl From<Binary> for Expr {
    fn from(val: Binary) -> Self {
        Expr::Binary(val)
    }
}

impl From<Unary> for Expr {
    fn from(val: Unary) -> Self {
        Expr::Unary(val)
    }
}

impl From<Call> for Expr {
    fn from(val: Call) -> Self {
        Expr::Call(val)
    }
}

impl From<FieldIn> for Expr {
    fn from(val: FieldIn) -> Self {
        Expr::FieldIn(val)
    }
}

impl From<Alias> for Expr {
    fn from(val: Alias) -> Self {
        Expr::Alias(val)
    }
}

impl From<FormattedTimeModifier> for Expr {
    fn from(val: FormattedTimeModifier) -> Self {
        Expr::TimeModifier(val)
    }
}

impl From<SearchModifier> for Expr {
    fn from(val: SearchModifier) -> Self {
        Expr::SearchModifier(val)
    }
}

impl From<Field> for FieldLike {
    fn from(val: Field) -> Self {
        FieldLike::Field(val)
    }
}

impl From<Wildcard> for FieldLike {
    fn from(val: Wildcard) -> Self {
        FieldLike::Wildcard(val)
    }
}

impl From<AliasedField> for FieldLike {
    fn from(val: AliasedField) -> Self {
        FieldLike::AliasedField(val)
    }
}

impl From<Alias> for FieldLike {
    fn from(val: Alias) -> Self {
        FieldLike::Alias(val)
    }
}

impl From<Field> for FieldOrAlias {
    fn from(val: Field) -> Self {
        FieldOrAlias::Field(val)
    }
}

impl From<Alias> for FieldOrAlias {
    fn from(val: Alias) -> Self {
        FieldOrAlias::Alias(val)
    }
}

impl From<AliasedField> for FieldOrAlias {
    fn from(val: AliasedField) -> Self {
        FieldOrAlias::Alias(val.into())
    }
}
