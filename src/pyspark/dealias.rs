use crate::pyspark::ast::{ColumnLike, Expr};

pub trait Dealias {
    fn unaliased(&self) -> Self;
}

impl Dealias for ColumnLike {
    fn unaliased(&self) -> Self {
        match self {
            ColumnLike::Aliased { col, .. } => match *col.clone() {
                Expr::Column(col) => col.unaliased(),
                _ => self.clone(),
            },
            _ => self.clone(),
        }
    }
}

impl Dealias for Expr {
    fn unaliased(&self) -> Self {
        match self {
            Expr::Column(col) => Self::Column(col.unaliased()),
            _ => self.clone(),
        }
    }
}

impl Dealias for String {
    fn unaliased(&self) -> Self {
        self.clone()
    }
}

impl Dealias for i64 {
    fn unaliased(&self) -> Self {
        *self
    }
}

impl Dealias for f64 {
    fn unaliased(&self) -> Self {
        *self
    }
}

impl Dealias for bool {
    fn unaliased(&self) -> Self {
        *self
    }
}
