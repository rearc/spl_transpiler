use std::sync::atomic::{AtomicUsize, Ordering};
// use std::sync::atomic::{AtomicUsize, Ordering};
use crate::pyspark::alias::Aliasable;
use crate::pyspark::base::{PythonCode, ToSparkQuery};
use anyhow::{ensure, Result};

#[derive(Debug, PartialEq, Clone, Hash)]
// #[pyclass(frozen,eq,hash)]
pub struct Base();

#[derive(Debug, PartialEq, Clone, Hash)]
// #[pyclass(frozen,eq,hash)]
pub struct PyLiteral(pub String);

impl<T: Into<PyLiteral>> From<Option<T>> for PyLiteral {
    fn from(value: Option<T>) -> PyLiteral {
        match value {
            Some(v) => v.into(),
            None => PyLiteral("None".into()),
        }
    }
}

impl From<String> for PyLiteral {
    fn from(value: String) -> PyLiteral {
        PyLiteral(format!("\"{}\"", value))
    }
}
impl From<&str> for PyLiteral {
    fn from(value: &str) -> PyLiteral {
        PyLiteral(format!("\"{}\"", value))
    }
}

impl From<i64> for PyLiteral {
    fn from(value: i64) -> PyLiteral {
        PyLiteral(value.to_string())
    }
}
impl From<f64> for PyLiteral {
    fn from(value: f64) -> PyLiteral {
        PyLiteral(value.to_string())
    }
}
impl From<bool> for PyLiteral {
    fn from(value: bool) -> PyLiteral {
        PyLiteral(if value { "True".into() } else { "False".into() })
    }
}

impl ToSparkQuery for PyLiteral {
    fn to_spark_query(&self) -> Result<PythonCode> {
        Ok(self.0.to_string().into())
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

impl ColumnLike {
    pub fn named(name: impl ToString) -> Self {
        ColumnLike::Named {
            name: name.to_string(),
        }
    }

    pub fn literal(code: impl ToString) -> Self {
        ColumnLike::Literal {
            code: code.to_string(),
        }
    }

    pub fn method_call(col: impl Into<Expr>, func: impl ToString, args: Vec<Expr>) -> Self {
        Self::try_method_call(col, func, args).unwrap()
    }

    pub fn try_method_call(
        col: impl TryInto<Expr, Error = impl Into<anyhow::Error> + Send>,
        func: impl ToString,
        args: Vec<Expr>,
    ) -> Result<Self> {
        let col: Expr = col.try_into().map_err(|e| e.into())?;
        Ok(ColumnLike::MethodCall {
            col: Box::new(col.unaliased()),
            func: func.to_string(),
            args,
        })
    }

    pub fn function_call(func: impl ToString, args: Vec<Expr>) -> Self {
        ColumnLike::FunctionCall {
            func: func.to_string(),
            args,
        }
    }

    pub fn aliased(col: impl Into<Expr>, name: impl ToString) -> Self {
        Self::try_aliased(col, name).unwrap()
    }

    pub fn try_aliased(
        col: impl TryInto<Expr, Error = impl Into<anyhow::Error> + Send>,
        name: impl ToString,
    ) -> Result<Self> {
        let col: Expr = col.try_into().map_err(|e| e.into())?;
        Ok(ColumnLike::Aliased {
            col: Box::new(col.unaliased()),
            name: name.to_string(),
        })
    }

    pub fn binary_op(left: impl Into<Expr>, op: impl ToString, right: impl Into<Expr>) -> Self {
        Self::try_binary_op(left, op, right).unwrap()
    }

    pub fn try_binary_op(
        left: impl TryInto<Expr, Error = impl Into<anyhow::Error> + Send>,
        op: impl ToString,
        right: impl TryInto<Expr, Error = impl Into<anyhow::Error> + Send>,
    ) -> Result<Self> {
        let left: Expr = left.try_into().map_err(|e| e.into())?;
        let right: Expr = right.try_into().map_err(|e| e.into())?;
        Ok(ColumnLike::BinaryOp {
            left: Box::new(left.unaliased()),
            op: op.to_string(),
            right: Box::new(right.unaliased()),
        })
    }

    pub fn unary_not(right: impl Into<Expr>) -> Self {
        Self::try_unary_not(right).unwrap()
    }

    pub fn try_unary_not(
        right: impl TryInto<Expr, Error = impl Into<anyhow::Error> + Send>,
    ) -> Result<Self> {
        let right: Expr = right.try_into().map_err(|e| e.into())?;
        Ok(ColumnLike::UnaryNot {
            right: Box::new(right.unaliased()),
        })
    }
}

macro_rules! column_like {
    ($name: ident) => { $name };
    (col($name: expr)) => { ColumnLike::named($name) };
    (lit(true)) => { ColumnLike::literal("True") };
    (lit(false)) => { ColumnLike::literal("False") };
    (lit(None)) => { ColumnLike::literal("None") };
    (lit($code: literal)) => { ColumnLike::literal(stringify!($code)) };
    (lit($code: expr)) => { ColumnLike::literal($code) };
    // (py_lit($code: literal)) => { Raw::from($code) };
    (py_lit($code: expr)) => { PyLiteral::from($code) };
    (expr($fmt: literal $($args:tt)*)) => { ColumnLike::function_call(
        "expr",
        vec![PyLiteral::from( format!($fmt $($args)*) ).into()],
    ) };
    ([$($base: tt)*] . alias ( $name: expr ) ) => { ColumnLike::aliased(
        column_like!($($base)*),
        $name
    ) };
    ([$($base: tt)*] . $method: ident ( $([$($args: tt)*]),* )) => { ColumnLike::method_call(
        column_like!($($base)*),
        stringify!($method),
        vec![$(column_like!($($args)*).into()),*],
    ) };
    ($function: ident ( $([$($args: tt)*]),* )) => { ColumnLike::function_call(
        stringify!($function),
        vec![$(column_like!($($args)*).into()),*],
    ) };
    ($function: ident ( $args: expr )) => { ColumnLike::function_call(
        stringify!($function),
        $args,
    ) };
    ([$($left: tt)*] $op: tt [$($right: tt)*]) => { ColumnLike::binary_op(
        column_like!($($left)*),
        stringify!($op),
        column_like!($($right)*),
    ) };
    (~ [$($right: tt)*]) => { ColumnLike::unary_not(
        column_like!($($right)*),
    ) };
    ($e: expr) => { $e };
}

pub(crate) use column_like;
// <-- the trick

impl ToSparkQuery for ColumnLike {
    fn to_spark_query(&self) -> Result<PythonCode> {
        match self {
            ColumnLike::Named { name } => Ok(format!("F.col('{}')", name)),
            ColumnLike::Literal { code } => Ok(format!("F.lit({})", code)),
            ColumnLike::MethodCall { col, func, args } => {
                let args: Result<Vec<String>> = args
                    .iter()
                    .map(|e| e.to_spark_query().map(|code| code.to_string()))
                    .collect();
                Ok(format!(
                    "{}.{}({})",
                    col.to_spark_query()?,
                    func,
                    args?.join(", ")
                ))
            }
            ColumnLike::FunctionCall { func, args } => {
                let args: Result<Vec<String>> = args
                    .iter()
                    .map(|e| e.to_spark_query().map(|code| code.to_string()))
                    .collect();
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
        .map(Into::into)
    }
}

#[derive(Debug, PartialEq, Clone, Hash)]
// #[pyclass(frozen,eq,hash)]
pub enum Expr {
    Column(ColumnLike),
    PyLiteral(PyLiteral),
}

impl ToSparkQuery for Expr {
    fn to_spark_query(&self) -> Result<PythonCode> {
        match self {
            Expr::Column(col @ ColumnLike::BinaryOp { .. }) => {
                Ok(format!("({})", col.to_spark_query()?,).into())
            }
            Expr::Column(col) => col.to_spark_query(),
            Expr::PyLiteral(literal) => literal.to_spark_query(),
        }
    }
}

impl From<ColumnLike> for Expr {
    fn from(val: ColumnLike) -> Self {
        Expr::Column(val)
    }
}
impl From<PyLiteral> for Expr {
    fn from(val: PyLiteral) -> Self {
        Expr::PyLiteral(val)
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
pub enum ColumnOrName {
    Column(ColumnLike),
    Name(String),
}

impl From<String> for ColumnOrName {
    fn from(val: String) -> Self {
        ColumnOrName::Name(val)
    }
}

impl From<ColumnLike> for ColumnOrName {
    fn from(val: ColumnLike) -> Self {
        ColumnOrName::Column(val)
    }
}

impl ToSparkQuery for ColumnOrName {
    fn to_spark_query(&self) -> Result<PythonCode> {
        match self {
            ColumnOrName::Column(col) => col.to_spark_query(),
            ColumnOrName::Name(name) => Ok(format!("\"{}\"", name).into()),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Hash)]
pub struct PyDict(pub Vec<(String, RuntimeExpr)>);

macro_rules! py_dict {
    () => { PyDict(vec![]) };
    ($($key: ident = $value: expr),+ $(,)?) => {
        PyDict(vec![$((stringify!($key).into(), $value.into())),+])
    };
}

pub(crate) use py_dict;

impl PyDict {
    pub fn push(&mut self, key: impl ToString, value: impl Into<RuntimeExpr>) {
        self.0.push((key.to_string(), value.into()));
    }
}

impl ToSparkQuery for PyDict {
    fn to_spark_query(&self) -> Result<PythonCode> {
        let mut out_preface = vec![];
        let mut out_vals = vec![];

        for (key, value) in self.0.iter() {
            let PythonCode {
                preface,
                primary_df_code,
            } = value.to_spark_query()?;
            out_preface.extend(preface);
            out_vals.push(format!("\"{}\": {}", key, primary_df_code).to_string());
        }
        Ok(format!(r#"{{ {} }}"#, out_vals.join(", ")).into())
    }
}

impl Extend<(String, RuntimeExpr)> for PyDict {
    fn extend<T: IntoIterator<Item = (String, RuntimeExpr)>>(&mut self, iter: T) {
        self.0.extend(iter)
    }
}

impl IntoIterator for PyDict {
    type Item = (String, RuntimeExpr);
    type IntoIter = <Vec<(String, RuntimeExpr)> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Debug, PartialEq, Clone, Hash)]
pub struct PyList(pub Vec<RuntimeExpr>);

macro_rules! py_list {
    ($($value: expr),+ $(,)?) => {
        PyList(vec![$($value.into()),+])
    }
}

impl ToSparkQuery for PyList {
    fn to_spark_query(&self) -> Result<PythonCode> {
        let mut out_preface = vec![];
        let mut out_vals = vec![];

        for value in self.0.iter() {
            let PythonCode {
                preface,
                primary_df_code,
            } = value.to_spark_query()?;
            out_preface.extend(preface);
            out_vals.push(primary_df_code);
        }
        Ok(format!(r#"[ {} ]"#, out_vals.join(", ")).into())
    }
}

impl Extend<RuntimeExpr> for PyList {
    fn extend<T: IntoIterator<Item = RuntimeExpr>>(&mut self, iter: T) {
        self.0.extend(iter)
    }
}

impl IntoIterator for PyList {
    type Item = RuntimeExpr;
    type IntoIter = <Vec<RuntimeExpr> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Debug, PartialEq, Clone, Hash)]
pub struct PyRuntimeFunc {
    pub name: String,
    pub args: PyList,
    pub kwargs: PyDict,
}

impl PyRuntimeFunc {
    pub fn new(name: impl ToString, args: impl Into<PyList>, kwargs: impl Into<PyDict>) -> Self {
        PyRuntimeFunc {
            name: name.to_string(),
            args: args.into(),
            kwargs: kwargs.into(),
        }
    }
}

macro_rules! py_runtime_func {
    ($func: ident ( $($arg: expr),* ; $($kwkey: ident = $kwval: expr),* )) => {
        PyRuntimeFunc::new(
            stringify!($func),
            py_list!($($arg),*),
            py_dict!($($kwkey = $kwval),*),
        )
    };
}

impl ToSparkQuery for PyRuntimeFunc {
    fn to_spark_query(&self) -> Result<PythonCode> {
        let mut out_preface = vec![];
        let mut out_args = vec![];

        for arg in self.args.0.iter() {
            let PythonCode {
                preface,
                primary_df_code,
            } = arg.to_spark_query()?;
            out_preface.extend(preface);
            out_args.push(primary_df_code);
        }

        for (key, value) in self.kwargs.0.iter() {
            let PythonCode {
                preface,
                primary_df_code,
            } = value.to_spark_query()?;
            out_preface.extend(preface);
            out_args.push(format!("{}={}", key, primary_df_code).to_string());
        }

        Ok(format!("functions.{}({})", self.name, out_args.join(", ")).into())
    }
}

#[derive(Debug, PartialEq, Clone, Hash)]
pub enum RuntimeExpr {
    DataFrame(Box<DataFrame>),
    Expr(Expr),
    PyDict(PyDict),
    PyList(PyList),
    PyRuntimeFunc(PyRuntimeFunc),
}

impl From<DataFrame> for RuntimeExpr {
    fn from(val: DataFrame) -> Self {
        RuntimeExpr::DataFrame(Box::new(val))
    }
}

impl<E: Into<Expr>> From<E> for RuntimeExpr {
    fn from(val: E) -> Self {
        RuntimeExpr::Expr(val.into())
    }
}

impl From<ColumnOrName> for RuntimeExpr {
    fn from(val: ColumnOrName) -> Self {
        match val {
            ColumnOrName::Column(col) => RuntimeExpr::from(col),
            ColumnOrName::Name(name) => RuntimeExpr::Expr(Expr::Column(ColumnLike::Named { name })),
        }
    }
}

impl From<PyDict> for RuntimeExpr {
    fn from(val: PyDict) -> Self {
        RuntimeExpr::PyDict(val)
    }
}

impl From<PyList> for RuntimeExpr {
    fn from(val: PyList) -> Self {
        RuntimeExpr::PyList(val)
    }
}

impl From<PyRuntimeFunc> for RuntimeExpr {
    fn from(val: PyRuntimeFunc) -> Self {
        RuntimeExpr::PyRuntimeFunc(val)
    }
}

impl ToSparkQuery for RuntimeExpr {
    fn to_spark_query(&self) -> Result<PythonCode> {
        match self {
            RuntimeExpr::DataFrame(val) => val.to_spark_query(),
            RuntimeExpr::Expr(val) => val.to_spark_query(),
            RuntimeExpr::PyDict(val) => val.to_spark_query(),
            RuntimeExpr::PyList(val) => val.to_spark_query(),
            RuntimeExpr::PyRuntimeFunc(val) => val.to_spark_query(),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Clone, Hash)]
pub enum DataFrame {
    Source {
        name: String,
    },
    Runtime {
        name: String,
        source: Option<Box<DataFrame>>,
        runtime_func: String,
        args: Vec<RuntimeExpr>,
        kwargs: Vec<(String, RuntimeExpr)>,
    },
    RawSource {
        code: String,
    },
    Named {
        source: Box<DataFrame>,
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
        columns: Vec<ColumnOrName>,
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
    WithColumnRenamed {
        source: Box<DataFrame>,
        old_name: String,
        new_name: String,
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
    Tail {
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
    ArbitraryMethod {
        source: Box<DataFrame>,
        method: String,
        params: Vec<Expr>,
    },
}

static DF_COUNTER: AtomicUsize = AtomicUsize::new(1);

#[allow(dead_code)]
impl DataFrame {
    fn generate_unique_name() -> String {
        let uid = DF_COUNTER.fetch_add(1, Ordering::Relaxed);
        format!("df_{uid}")
    }

    pub fn reset_df_count() {
        DF_COUNTER.store(1, Ordering::Relaxed);
    }

    pub fn source(name: impl ToString) -> DataFrame {
        DataFrame::Source {
            name: name.to_string(),
        }
    }
    pub fn runtime(
        source: Option<DataFrame>,
        runtime_func: impl ToString,
        args: Vec<RuntimeExpr>,
        kwargs: Vec<(String, RuntimeExpr)>,
    ) -> Self {
        Self::Runtime {
            name: Self::generate_unique_name(),
            source: source.map(Box::new),
            runtime_func: runtime_func.to_string(),
            args,
            kwargs,
        }
    }
    pub fn raw_source(code: impl ToString) -> DataFrame {
        DataFrame::RawSource {
            code: code.to_string(),
        }
    }
    pub fn named(&self, name: Option<String>) -> DataFrame {
        DataFrame::Named {
            source: Box::new(self.clone()),
            name: name.unwrap_or_else(Self::generate_unique_name),
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
    pub fn group_by(&self, columns: Vec<impl Into<ColumnOrName>>) -> DataFrame {
        Self::GroupBy {
            source: Box::new(self.clone()),
            columns: columns.into_iter().map(Into::into).collect(),
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
    pub fn with_column_renamed(
        &self,
        old_name: impl ToString,
        new_name: impl ToString,
    ) -> DataFrame {
        Self::WithColumnRenamed {
            source: Box::new(self.clone()),
            old_name: old_name.to_string(),
            new_name: new_name.to_string(),
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
    pub fn tail(&self, limit: impl Into<u64>) -> DataFrame {
        Self::Tail {
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
    pub fn arbitrary_method(&self, method: impl ToString, params: Vec<Expr>) -> DataFrame {
        Self::ArbitraryMethod {
            source: Box::new(self.clone()),
            method: method.to_string(),
            params,
        }
    }
}

impl ToSparkQuery for DataFrame {
    fn to_spark_query(&self) -> Result<PythonCode> {
        match self {
            DataFrame::Source { name } => Ok(format!("spark.table('{}')", name).into()),
            DataFrame::RawSource { code } => Ok(code.clone().into()),
            DataFrame::Runtime {
                name,
                source,
                runtime_func,
                args,
                kwargs,
            } => {
                let primary_source_code = match source {
                    Some(ref source) => source.to_spark_query()?,
                    None => PythonCode::from("None".to_string()),
                };
                let mut all_preface = vec![];
                let mut all_args = vec![primary_source_code.primary_df_code.clone()];
                for arg in args.iter() {
                    let PythonCode {
                        preface,
                        primary_df_code,
                    } = arg.to_spark_query()?;
                    all_preface.extend(preface);
                    all_args.push(primary_df_code);
                }
                for (name, kwarg) in kwargs.iter() {
                    let PythonCode {
                        preface,
                        primary_df_code,
                    } = kwarg.to_spark_query()?;
                    all_preface.extend(preface);
                    all_args.push(format!("{name}={primary_df_code}"));
                }
                let all_args_str = all_args.join(", ");
                let full_command =
                    format!("{} = commands.{}({})", name, runtime_func, all_args_str);
                all_preface.push(full_command);

                Ok(PythonCode::new(
                    name.clone(),
                    all_preface,
                    Some(primary_source_code),
                ))
            }
            DataFrame::Named { source, name } => {
                let source_code = source.to_spark_query()?;
                Ok(PythonCode::new(
                    name.clone(),
                    vec![format!("{} = {}", name, source_code.primary_df_code)],
                    Some(source_code),
                ))
            }
            DataFrame::Select { source, columns } => {
                let columns: Result<Vec<String>> = columns
                    .iter()
                    .map(|col| col.to_spark_query().map(|code| code.to_string()))
                    .collect();
                Ok(format!(
                    "{}.select({},)",
                    source.to_spark_query()?,
                    columns?.join(", ")
                )
                .into())
            }
            DataFrame::Where { source, condition } => Ok(format!(
                "{}.where({},)",
                source.to_spark_query()?,
                condition.to_spark_query()?,
            )
            .into()),
            DataFrame::GroupBy { source, columns } => {
                // Extra trailing comma is intentional
                let columns: Result<Vec<String>> = columns
                    .iter()
                    .map(|s| s.to_spark_query().map(|s| format!("{}, ", s)))
                    .collect();
                Ok(format!(
                    "{}.groupBy([{}])",
                    source.to_spark_query()?,
                    columns?.join("")
                )
                .into())
            }
            DataFrame::Aggregation { source, columns } => {
                let columns: Result<Vec<String>> = columns
                    .iter()
                    .map(|col| col.to_spark_query().map(|code| code.to_string()))
                    .collect();
                Ok(format!("{}.agg({},)", source.to_spark_query()?, columns?.join(", ")).into())
            }
            DataFrame::WithColumn {
                source,
                column,
                name,
            } => Ok(format!(
                "{}.withColumn('{}', {},)",
                source.to_spark_query()?,
                name,
                column.to_spark_query()?,
            )
            .into()),
            DataFrame::WithColumnRenamed {
                source,
                old_name,
                new_name,
            } => Ok(format!(
                "{}.withColumnRenamed('{}', '{}',)",
                source.to_spark_query()?,
                old_name,
                new_name,
            )
            .into()),
            DataFrame::OrderBy { source, columns } => {
                let columns: Result<Vec<String>> = columns
                    .iter()
                    .map(|col| col.to_spark_query().map(|code| code.to_string()))
                    .collect();
                Ok(format!(
                    "{}.orderBy({},)",
                    source.to_spark_query()?,
                    columns?.join(", ")
                )
                .into())
            }
            DataFrame::UnionByName { source, other } => Ok(format!(
                "{}.unionByName({}, allowMissingColumns=True,)",
                source.to_spark_query()?,
                other.to_spark_query()?,
            )
            .into()),
            DataFrame::Limit { source, limit } => {
                Ok(format!("{}.limit({},)", source.to_spark_query()?, limit).into())
            }
            DataFrame::Tail { source, limit } => {
                Ok(format!("{}.tail({})", source.to_spark_query()?, limit).into())
            }
            DataFrame::Join {
                source,
                other,
                condition,
                join_type,
            } => Ok(format!(
                "{}.join({}, {}, '{}',)",
                source.to_spark_query()?,
                other.to_spark_query()?,
                condition.to_spark_query()?,
                join_type
            )
            .into()),
            DataFrame::Alias { source, name } => {
                Ok(format!("{}.alias('{}',)", source.to_spark_query()?, name,).into())
            }
            DataFrame::ArbitraryMethod {
                source,
                method,
                params,
            } => {
                let params: Result<Vec<String>> = params
                    .iter()
                    .map(|col| col.to_spark_query().map(|code| code.to_string()))
                    .collect();
                Ok(format!(
                    "{}.{}({},)",
                    source.to_spark_query()?,
                    method,
                    params?.join(",")
                )
                .into())
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone, Hash)]
pub struct TransformedPipeline {
    pub dataframes: Vec<DataFrame>,
}

impl ToSparkQuery for TransformedPipeline {
    fn to_spark_query(&self) -> Result<PythonCode> {
        let dfs: Result<Vec<String>> = self
            .dataframes
            .iter()
            .map(|df| df.to_spark_query().map(|code| code.to_string()))
            .collect();
        Ok(dfs?.join("\n\n").into())
    }
}

impl TryInto<DataFrame> for TransformedPipeline {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<DataFrame, Self::Error> {
        ensure!(
            self.dataframes.len() == 1,
            "Unable to map over multi-dataframe pipelines"
        );
        Ok(self.dataframes.first().unwrap().clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format_python::format_python_code;

    use crate::pyspark::utils::test::assert_python_code_eq;
    // fn generates(spl_query: &str, spark_query: &str) {
    //     let (_, pipeline_ast) = crate::spl::pipeline(spl_query).expect("Failed to parse SPL query");
    //     let converted = convert(pipeline_ast).expect("Failed to convert SPL query to Spark query");
    //     let rendered = converted
    //         .to_spark_query()
    //         .expect("Failed to render Spark query");
    //     let formatted_rendered = format_python_code(rendered.replace(",)", ")")).expect("Failed to format rendered Spark query");
    //     let formatted_spark_query =
    //         format_python_code(spark_query.replace(",)", ")")).expect("Failed to format target Spark query");
    //     assert_eq!(formatted_rendered, formatted_spark_query);
    // }

    fn generates(ast: impl ToSparkQuery, code: impl ToString) {
        let generated = ast.to_spark_query().expect("Failed to generate code");
        let formatted_generated = format_python_code(generated.to_string().replace(",)", ")"))
            .expect("Failed to format rendered Spark query");
        let formatted_code = format_python_code(code.to_string().replace(",)", ")"))
            .expect("Failed to format target code");
        assert_eq!(formatted_generated, formatted_code);
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
            ColumnLike::unary_not(ColumnLike::named("x")),
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

    #[test]
    fn test_named() {
        generates(
            DataFrame::source("main")
                .with_column(
                    "final_name",
                    column_like!([col("orig_name")].alias("alias_name")),
                )
                .named(Some("prev".to_string())),
            r#"
prev = spark.table("main").withColumn("final_name", F.col("orig_name"))
prev
            "#,
        )
    }

    #[test]
    fn test_unique_ids() {
        DataFrame::reset_df_count();
        let df_1 = DataFrame::runtime(
            // None,
            None,
            "search".to_string(),
            vec![column_like!([col("x")] == [lit(3)]).into()],
            vec![],
        );
        let df_2 = DataFrame::runtime(
            Some(df_1),
            "eval".to_string(),
            vec![],
            vec![("y".to_string(), column_like!(length([col("x")])).into())],
        );
        assert_python_code_eq(
            df_2.to_spark_query().unwrap().to_string(),
            r#"
df_1 = commands.search(None, (F.col('x') == F.lit(3)))
df_2 = commands.eval(df_1, y=F.length(F.col('x')))
df_2
            "#
            .trim(),
            false,
        );
    }
}
