#[cfg(test)]
pub mod test {
    use crate::spl::operators::OperatorSymbolTrait;
    use crate::spl::{ast, operators};

    pub fn _field_equals(field: &str, value: ast::Constant) -> ast::Expr {
        ast::Expr::Binary(ast::Binary::new(
            ast::Expr::Leaf(ast::LeafExpr::Constant(ast::Constant::Field(ast::Field(
                field.into(),
            )))),
            operators::Equals::SYMBOL,
            ast::Expr::Leaf(ast::LeafExpr::Constant(value)),
        ))
    }

    pub fn _binop<Op: OperatorSymbolTrait>(
        left: impl Into<ast::Expr>,
        right: impl Into<ast::Expr>,
    ) -> ast::Expr {
        ast::Expr::Binary(ast::Binary::new(left, Op::SYMBOL, right))
    }

    pub fn _or(left: impl Into<ast::Expr>, right: impl Into<ast::Expr>) -> ast::Expr {
        _binop::<operators::Or>(left, right)
    }

    pub fn _and(left: impl Into<ast::Expr>, right: impl Into<ast::Expr>) -> ast::Expr {
        _binop::<operators::And>(left, right)
    }

    pub fn _eq(left: impl Into<ast::Expr>, right: impl Into<ast::Expr>) -> ast::Expr {
        _binop::<operators::Equals>(left, right)
    }

    pub fn _neq(left: impl Into<ast::Expr>, right: impl Into<ast::Expr>) -> ast::Expr {
        _binop::<operators::NotEquals>(left, right)
    }

    pub fn _lt(left: impl Into<ast::Expr>, right: impl Into<ast::Expr>) -> ast::Expr {
        _binop::<operators::LessThan>(left, right)
    }

    pub fn _gt(left: impl Into<ast::Expr>, right: impl Into<ast::Expr>) -> ast::Expr {
        _binop::<operators::GreaterThan>(left, right)
    }

    pub fn _gte(left: impl Into<ast::Expr>, right: impl Into<ast::Expr>) -> ast::Expr {
        _binop::<operators::GreaterEquals>(left, right)
    }

    pub fn _lte(left: impl Into<ast::Expr>, right: impl Into<ast::Expr>) -> ast::Expr {
        _binop::<operators::LessEquals>(left, right)
    }

    pub fn _not(right: impl Into<ast::Expr>) -> ast::Expr {
        ast::Unary::new(operators::UnaryNot::SYMBOL, right).into()
    }

    pub fn _isin(left: impl ToString, right: Vec<ast::Expr>) -> ast::Expr {
        ast::FieldIn {
            field: left.to_string(),
            exprs: right,
        }
        .into()
    }

    pub fn _alias(name: impl ToString, expr: impl Into<ast::Expr>) -> ast::Alias {
        ast::Alias {
            name: name.to_string(),
            expr: Box::new(expr.into()),
        }
    }

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

    #[allow(unused_imports)]
    pub(crate) use _call;
    #[allow(unused_imports)]
    pub(crate) use _call_args;
}
