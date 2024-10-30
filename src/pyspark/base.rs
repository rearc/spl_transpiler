use crate::pyspark::ast::Expr;
use anyhow::Result;
use std::fmt::{Debug, Display};
use std::ops::Deref;
use std::rc::Rc;
use std::sync::atomic::AtomicUsize;

#[derive(Debug, Clone)]
pub enum RuntimeSelection {
    Disallow,
    Allow,
    #[warn(dead_code)]
    Require,
}

#[derive(Debug, Clone)]
pub struct PysparkTranspileContext {
    pub df_num: Rc<AtomicUsize>,
    pub runtime: RuntimeSelection,
}

impl PysparkTranspileContext {
    #[warn(dead_code)]
    pub fn new(runtime: RuntimeSelection) -> Self {
        Self {
            runtime,
            ..Default::default()
        }
    }
}

impl Default for PysparkTranspileContext {
    fn default() -> Self {
        PysparkTranspileContext {
            df_num: Rc::new(AtomicUsize::new(1)),
            runtime: RuntimeSelection::Disallow,
        }
    }
}

pub struct PythonCode {
    pub preface: Vec<String>,
    pub primary_df_code: String,
}

impl PythonCode {
    pub fn new(primary_df_code: String, preface: Vec<String>, root: Option<PythonCode>) -> Self {
        let mut new_preface = vec![];
        if let Some(root) = root {
            new_preface.extend(root.preface);
        }
        new_preface.extend(preface);
        Self {
            primary_df_code,
            preface: new_preface,
        }
    }
}

impl From<String> for PythonCode {
    fn from(primary_df_code: String) -> Self {
        Self {
            primary_df_code: primary_df_code.to_string(),
            preface: Vec::new(),
        }
    }
}

impl Display for PythonCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for line in self.preface.iter() {
            writeln!(f, "{}", line)?;
        }
        write!(f, "{}", self.primary_df_code)
    }
}

pub trait ToSparkQuery {
    fn to_spark_query(&self, ctx: &PysparkTranspileContext) -> Result<PythonCode>;
}

pub trait ToSparkExpr: Sized + Debug + Clone {
    fn to_spark_expr(&self, ctx: &PysparkTranspileContext) -> Result<Expr>;

    fn with_context(self, ctx: &PysparkTranspileContext) -> ContextualizedExpr<Self> {
        ContextualizedExpr {
            ctx: ctx.clone(),
            val: self,
        }
    }

    // fn with_default_context(self) -> ContextualizedExpr<Self> {
    //
    // }
}

#[derive(Debug, Clone)]
pub struct ContextualizedExpr<T: ToSparkExpr> {
    ctx: PysparkTranspileContext,
    val: T,
}

impl<T: ToSparkExpr> Deref for ContextualizedExpr<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.val
    }
}

impl<T: ToSparkExpr> TryFrom<ContextualizedExpr<T>> for Expr {
    type Error = anyhow::Error;

    fn try_from(value: ContextualizedExpr<T>) -> std::result::Result<Self, Self::Error> {
        value.val.to_spark_expr(&value.ctx)
    }
}

#[cfg(test)]
pub mod test {
    use super::*;
    use rstest::fixture;

    pub fn test_pyspark_transpile_context(runtime: RuntimeSelection) -> PysparkTranspileContext {
        PysparkTranspileContext {
            runtime,
            ..Default::default()
        }
    }

    #[fixture]
    pub fn ctx_bare() -> PysparkTranspileContext {
        test_pyspark_transpile_context(RuntimeSelection::Disallow)
    }

    #[fixture]
    pub fn ctx_runtime() -> PysparkTranspileContext {
        test_pyspark_transpile_context(RuntimeSelection::Require)
    }
}
