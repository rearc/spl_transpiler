use crate::pyspark::base::TemplateNode;
use crate::pyspark::dealias::Dealias;
use anyhow::{ensure, Result};

#[derive(Debug, PartialEq, Clone, Hash)]
// #[pyclass(frozen,eq,hash)]
pub struct Base();

#[derive(Debug, PartialEq, Clone, Hash)]
// #[pyclass(frozen,eq,hash)]
pub struct Raw(String);

impl From<String> for Raw {
    fn from(value: String) -> Raw {
        Raw(format!("\"{}\"", value))
    }
}
impl From<&str> for Raw {
    fn from(value: &str) -> Raw {
        Raw(format!("\"{}\"", value))
    }
}

impl From<i64> for Raw {
    fn from(value: i64) -> Raw {
        Raw(value.to_string())
    }
}
impl From<f64> for Raw {
    fn from(value: f64) -> Raw {
        Raw(value.to_string())
    }
}
impl From<bool> for Raw {
    fn from(value: bool) -> Raw {
        Raw(if value { "True".into() } else { "False".into() })
    }
}

impl TemplateNode for Raw {
    fn to_spark_query(&self) -> Result<String> {
        Ok(self.0.to_string())
    }
}

#[derive(Debug, PartialEq, Clone, Hash)]
// #[pyclass(frozen,eq,hash)]
pub enum ColumnLike {
    Named {
        name: String,
    },
    Literal {
        code: String,
    },
    MethodCall {
        col: Box<Expr>,
        func: String,
        args: Vec<Expr>,
    },
    FunctionCall {
        func: String,
        args: Vec<Expr>,
    },
    Aliased {
        col: Box<Expr>,
        name: String,
    },
    BinaryOp {
        op: String,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    UnaryNot {
        right: Box<Expr>,
    },
}

macro_rules! column_like {
    ($name: ident) => { $name };
    (col($name: expr)) => { ColumnLike::Named { name: $name.to_string() } };
    (lit(true)) => { ColumnLike::Literal { code: "True".to_string() } };
    (lit(false)) => { ColumnLike::Literal { code: "False".to_string() } };
    (lit($code: literal)) => { ColumnLike::Literal { code: stringify!($code).to_string() } };
    (lit($code: expr)) => { ColumnLike::Literal { code: $code.to_string() } };
    // (py_lit($code: literal)) => { Raw::from($code) };
    (py_lit($code: expr)) => { Raw::from($code) };
    ([$($base: tt)*] . alias ( $name: expr ) ) => { ColumnLike::Aliased {
        col: Box::new(column_like!($($base)*).into()),
        name: $name.into()
    } };
    ([$($base: tt)*] . $method: ident ( $([$($args: tt)*]),* )) => { ColumnLike::MethodCall {
        col: Box::new(column_like!($($base)*).into()),
        func: stringify!($method).to_string(),
        args: vec![$(column_like!($($args)*).into()),*],
    } };
    ($function: ident ( $([$($args: tt)*]),* )) => { ColumnLike::FunctionCall {
        func: stringify!($function).to_string(),
        args: vec![$(column_like!($($args)*).into()),*],
    } };
    ([$($left: tt)*] $op: tt [$($right: tt)*]) => { ColumnLike::BinaryOp {
        left: Box::new(column_like!($($left)*).into()),
        op: stringify!($op).to_string(),
        right: Box::new(column_like!($($right)*).into()),
    } };
    ($e: expr) => { $e };
}

pub(crate) use column_like; // <-- the trick

impl TemplateNode for ColumnLike {
    fn to_spark_query(&self) -> anyhow::Result<String> {
        match self {
            ColumnLike::Named { name } => Ok(format!("F.col('{}')", name)),
            ColumnLike::Literal { code } => Ok(format!("F.lit({})", code)),
            ColumnLike::MethodCall { col, func, args } => {
                let args: Result<Vec<String>> = args.iter().map(|e| e.to_spark_query()).collect();
                Ok(format!(
                    "{}.{}({})",
                    col.to_spark_query()?,
                    func,
                    args?.join(", ")
                ))
            }
            ColumnLike::FunctionCall { func, args } => {
                let args: Result<Vec<String>> = args.iter().map(|e| e.to_spark_query()).collect();
                Ok(format!("F.{}({})", func, args?.join(", ")))
            }
            ColumnLike::Aliased { col, name } => {
                Ok(format!("{}.alias('{}')", col.to_spark_query()?, name))
            }
            ColumnLike::BinaryOp { op, left, right } => Ok(format!(
                "{} {} {}",
                left.to_spark_query()?,
                op,
                right.to_spark_query()?,
            )),
            ColumnLike::UnaryNot { right } => Ok(format!("~{}", right.to_spark_query()?,)),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Hash)]
// #[pyclass(frozen,eq,hash)]
pub enum Expr {
    Column(ColumnLike),
    Raw(Raw),
}

impl TemplateNode for Expr {
    fn to_spark_query(&self) -> anyhow::Result<String> {
        match self {
            Expr::Column(col @ ColumnLike::BinaryOp { .. }) => {
                Ok(format!("({})", col.to_spark_query()?,))
            }
            Expr::Column(col) => col.to_spark_query(),
            Expr::Raw(raw) => raw.to_spark_query(),
        }
    }
}

impl Into<Expr> for ColumnLike {
    fn into(self) -> Expr {
        Expr::Column(self)
    }
}
impl Into<Expr> for Raw {
    fn into(self) -> Expr {
        Expr::Raw(self)
    }
}
impl TryInto<ColumnLike> for Expr {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<ColumnLike, Self::Error> {
        match self {
            Expr::Column(col) => Ok(col),
            _ => Err(anyhow::anyhow!("Expected ColumnLike, got {:?}", self)),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Hash)]
pub enum DataFrame {
    Source {
        name: String,
    },
    Select {
        source: Box<DataFrame>,
        columns: Vec<ColumnLike>,
    },
    Where {
        source: Box<DataFrame>,
        condition: Expr,
    },
    GroupBy {
        source: Box<DataFrame>,
        columns: Vec<String>,
    },
    Aggregation {
        source: Box<DataFrame>,
        columns: Vec<ColumnLike>,
    },
    WithColumn {
        source: Box<DataFrame>,
        column: ColumnLike,
        name: String,
    },
    OrderBy {
        source: Box<DataFrame>,
        columns: Vec<ColumnLike>,
    },
    UnionByName {
        source: Box<DataFrame>,
        other: Box<DataFrame>,
    },
    Limit {
        source: Box<DataFrame>,
        limit: u64,
    },
    Join {
        source: Box<DataFrame>,
        other: Box<DataFrame>,
        condition: Expr,
        join_type: String,
    },
    Alias {
        source: Box<DataFrame>,
        name: String,
    },
}

impl DataFrame {
    pub fn source(name: impl ToString) -> DataFrame {
        DataFrame::Source {
            name: name.to_string(),
        }
    }
    pub fn select(&self, columns: Vec<ColumnLike>) -> DataFrame {
        Self::Select {
            source: Box::new(self.clone()),
            columns,
        }
    }
    pub fn where_(&self, condition: impl Into<Expr>) -> DataFrame {
        Self::Where {
            source: Box::new(self.clone()),
            condition: condition.into().unaliased(),
        }
    }
    pub fn group_by(&self, columns: Vec<String>) -> DataFrame {
        Self::GroupBy {
            source: Box::new(self.clone()),
            columns,
        }
    }
    pub fn agg(&self, columns: Vec<ColumnLike>) -> DataFrame {
        Self::Aggregation {
            source: Box::new(self.clone()),
            columns,
        }
    }
    pub fn with_column(&self, name: impl ToString, column: ColumnLike) -> DataFrame {
        Self::WithColumn {
            source: Box::new(self.clone()),
            column: column.unaliased(),
            name: name.to_string(),
        }
    }
    pub fn order_by(&self, columns: Vec<ColumnLike>) -> DataFrame {
        Self::OrderBy {
            source: Box::new(self.clone()),
            columns,
        }
    }
    pub fn union_by_name(&self, other: DataFrame) -> DataFrame {
        Self::UnionByName {
            source: Box::new(self.clone()),
            other: Box::new(other),
        }
    }
    pub fn limit(&self, limit: impl Into<u64>) -> DataFrame {
        Self::Limit {
            source: Box::new(self.clone()),
            limit: limit.into(),
        }
    }
    pub fn join(
        &self,
        other: DataFrame,
        condition: impl Into<Expr>,
        join_type: impl ToString,
    ) -> DataFrame {
        Self::Join {
            source: Box::new(self.clone()),
            other: Box::new(other),
            condition: condition.into().unaliased(),
            join_type: join_type.to_string(),
        }
    }
    pub fn alias(&self, name: impl ToString) -> DataFrame {
        Self::Alias {
            source: Box::new(self.clone()),
            name: name.to_string(),
        }
    }
}

impl TemplateNode for DataFrame {
    fn to_spark_query(&self) -> anyhow::Result<String> {
        match self {
            DataFrame::Source { name } => Ok(format!("spark.table('{}')", name)),
            DataFrame::Select { source, columns } => {
                let columns: Result<Vec<String>> =
                    columns.iter().map(|col| col.to_spark_query()).collect();
                Ok(format!(
                    "{}.select({})",
                    source.to_spark_query()?,
                    columns?.join(", ")
                ))
            }
            DataFrame::Where { source, condition } => Ok(format!(
                "{}.where({})",
                source.to_spark_query()?,
                condition.to_spark_query()?,
            )),
            DataFrame::GroupBy { source, columns } => {
                let columns: Vec<String> = columns.iter().map(|s| format!(r#"'{}'"#, s)).collect();
                Ok(format!(
                    "{}.groupBy({})",
                    source.to_spark_query()?,
                    columns.join(", ")
                ))
            }
            DataFrame::Aggregation { source, columns } => {
                let columns: Result<Vec<String>> =
                    columns.iter().map(|col| col.to_spark_query()).collect();
                Ok(format!(
                    "{}.agg({})",
                    source.to_spark_query()?,
                    columns?.join(", ")
                ))
            }
            DataFrame::WithColumn {
                source,
                column,
                name,
            } => Ok(format!(
                "{}.withColumn('{}', {})",
                source.to_spark_query()?,
                name,
                column.to_spark_query()?,
            )),
            DataFrame::OrderBy { source, columns } => {
                let columns: Result<Vec<String>> =
                    columns.iter().map(|col| col.to_spark_query()).collect();
                Ok(format!(
                    "{}.orderBy({})",
                    source.to_spark_query()?,
                    columns?.join(", ")
                ))
            }
            DataFrame::UnionByName { source, other } => Ok(format!(
                "{}.unionByName({}, allowMissingColumns=True)",
                source.to_spark_query()?,
                other.to_spark_query()?,
            )),
            DataFrame::Limit { source, limit } => {
                Ok(format!("{}.limit({})", source.to_spark_query()?, limit))
            }
            DataFrame::Join {
                source,
                other,
                condition,
                join_type,
            } => Ok(format!(
                "{}.join({}, {}, '{}')",
                source.to_spark_query()?,
                other.to_spark_query()?,
                condition.to_spark_query()?,
                join_type
            )),
            DataFrame::Alias { source, name } => {
                Ok(format!("{}.alias('{}')", source.to_spark_query()?, name,))
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone, Hash)]
pub struct TransformedPipeline {
    pub(crate) dataframes: Vec<DataFrame>,
}

impl TemplateNode for TransformedPipeline {
    fn to_spark_query(&self) -> anyhow::Result<String> {
        let dfs: Result<Vec<String>> = self
            .dataframes
            .iter()
            .map(|df| df.to_spark_query())
            .collect();
        Ok(dfs?.join("\n\n"))
    }
}

impl TryInto<DataFrame> for TransformedPipeline {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<DataFrame, Self::Error> {
        ensure!(
            self.dataframes.len() == 1,
            "Unable to map over multi-dataframe pipelines"
        );
        Ok(self.dataframes.get(0).unwrap().clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generates(ast: impl TemplateNode, code: impl ToString) {
        let generated = ast
            .to_formatted_spark_query()
            .expect("Failed to generate code");
        assert_eq!(generated, code.to_string());
    }

    #[test]
    fn test_column_like() {
        generates(column_like!(col("x")), r#"F.col("x")"#);
        generates(column_like!(lit(42)), r#"F.lit(42)"#);
        generates(column_like!([lit(42)].sqrt()), r#"F.lit(42).sqrt()"#);
        generates(column_like!(lit("xyz")), r#"F.lit("xyz")"#);
        generates(column_like!(sqrt([lit(42)])), r#"F.sqrt(F.lit(42))"#);
        generates(
            column_like!([col("x")].alias("y")),
            r#"F.col("x").alias("y")"#,
        );
        generates(
            column_like!([col("x")] + [lit(42)]),
            r#"F.col("x") + F.lit(42)"#,
        );
        generates(
            ColumnLike::UnaryNot {
                right: Box::new(
                    ColumnLike::Named {
                        name: "x".to_string(),
                    }
                    .into(),
                ),
            },
            r#"~F.col("x")"#,
        )
    }

    #[test]
    fn test_dataframe() {
        generates(
            DataFrame::source("main").where_(column_like!(col("x"))),
            r#"spark.table("main").where(F.col("x"))"#,
        )
    }

    #[test]
    fn test_with_aliased_column() {
        generates(
            DataFrame::source("main").with_column(
                "final_name",
                column_like!([col("orig_name")].alias("alias_name")),
            ),
            r#"spark.table("main").withColumn("final_name", F.col("orig_name"))"#,
        )
    }
}
