use crate::pyspark::ast::{ColumnLike, Expr, RuntimeExpr};
use crate::spl::ast;

pub trait Aliasable: Sized {
    fn unaliased_with_name(&self) -> (Self, Option<String>);

    fn unaliased(&self) -> Self {
        self.unaliased_with_name().0
    }
}

impl Aliasable for ast::Expr {
    fn unaliased_with_name(&self) -> (Self, Option<String>) {
        match self {
            ast::Expr::Alias(ast::Alias { expr, name }) => {
                let (expr, _) = (*expr).unaliased_with_name();
                (expr, Some(name.clone()))
            }
            ast::Expr::AliasedField(ast::AliasedField { field, alias }) => {
                (field.clone().into(), Some(alias.clone()))
            }
            _ => (self.clone(), None),
        }
    }
}

impl Aliasable for ColumnLike {
    fn unaliased_with_name(&self) -> (Self, Option<String>) {
        match self {
            ColumnLike::Aliased { col, name } => match *col.clone() {
                Expr::Column(col) => (col.unaliased(), Some(name.clone())),
                _ => (self.clone(), Some(name.clone())),
            },
            _ => (self.clone(), None),
        }
    }
}

impl Aliasable for Expr {
    fn unaliased_with_name(&self) -> (Self, Option<String>) {
        match self {
            Expr::Column(col) => {
                let (c, name) = col.unaliased_with_name();
                (c.into(), name)
            }
            _ => (self.clone(), None),
        }
    }
}

impl Aliasable for RuntimeExpr {
    fn unaliased_with_name(&self) -> (Self, Option<String>) {
        match self {
            RuntimeExpr::Expr(expr) => {
                let (expr, name) = expr.unaliased_with_name();
                (expr.into(), name)
            }
            _ => (self.clone(), None),
        }
    }
}

impl ColumnLike {
    pub fn with_alias(&self, name: String) -> Self {
        ColumnLike::aliased(self.unaliased(), name)
    }

    pub fn maybe_with_alias(&self, name: Option<String>) -> Self {
        match name {
            Some(n) => self.with_alias(n),
            None => self.clone(),
        }
    }
}

impl Expr {
    pub fn with_alias(&self, name: String) -> Self {
        ColumnLike::aliased(self.clone(), name).into()
    }

    pub fn maybe_with_alias(&self, name: Option<String>) -> Self {
        match name {
            Some(n) => self.with_alias(n),
            None => self.clone(),
        }
    }
}

impl ast::Expr {
    pub fn with_alias(&self, name: String) -> Self {
        ast::Expr::Alias(ast::Alias {
            expr: Box::new(self.clone()),
            name,
        })
    }

    pub fn maybe_with_alias(&self, name: Option<String>) -> Self {
        match name {
            Some(n) => self.with_alias(n),
            None => self.clone(),
        }
    }
}

impl Aliasable for String {
    fn unaliased_with_name(&self) -> (Self, Option<String>) {
        (self.clone(), None)
    }
}

impl Aliasable for i64 {
    fn unaliased_with_name(&self) -> (Self, Option<String>) {
        (*self, None)
    }
}

impl Aliasable for f64 {
    fn unaliased_with_name(&self) -> (Self, Option<String>) {
        (*self, None)
    }
}

impl Aliasable for bool {
    fn unaliased_with_name(&self) -> (Self, Option<String>) {
        (*self, None)
    }
}
