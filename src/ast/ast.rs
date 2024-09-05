/*
package com.databricks.labs.transpiler.spl.ast

sealed trait Expr
sealed trait LeafExpr extends Expr

sealed trait FieldLike
sealed trait Constant extends LeafExpr
sealed trait SplSpan extends Constant
sealed trait FieldOrAlias

case class Null() extends Constant
case class Bool(value: Boolean) extends Constant
case class IntValue(value: Int) extends Constant
case class DoubleValue(value: Double) extends Constant
case class StrValue(value: String) extends Constant
case class TimeSpan(value: Int, scale: String) extends SplSpan
case class SnapTime(span: Option[TimeSpan], snap: String,
                    snapOffset: Option[TimeSpan]) extends Constant
case class Field(value: String) extends Constant with FieldLike with FieldOrAlias
case class Wildcard(value: String) extends Constant with FieldLike
case class Variable(value: String) extends Constant
case class IPv4CIDR(value: String) extends Constant

case class FV(field: String, value: String) extends LeafExpr
case class FB(field: String, value: Boolean) extends LeafExpr
case class FC(field: String, value: Constant) extends LeafExpr

case class CommandOptions(options: Seq[FC]) {
  private val inner = options.map(y => y.field -> y.value).toMap

  private def throwIAE(msg: String) = throw new IllegalArgumentException(msg)

  def toMap: Map[String, Constant] = inner
  def getIntOption(key: String): Option[Int] = inner.get(key) map {
    case IntValue(value) => value
    case other: Constant => throwIAE(s"not an int: $other")
  }

  def getInt(key: String, default: Int = 0): Int =
    getIntOption(key).getOrElse(default)

  def getStringOption(key: String): Option[String] = inner.get(key) map {
    case Field(v) => v
    case StrValue(v) => v
    case other: Constant => throwIAE(s"not a string: $other")
  }

  def getString(key: String, default: String): String =
    getStringOption(key).getOrElse(default)

  def getSpanOption(key: String): Option[SplSpan] = inner.get(key) map {
    case span: SplSpan => span
    case other: Constant => throwIAE(s"not a span: $other")
  }

  def getBoolean(key: String, default: Boolean = false): Boolean = inner.get(key) map {
    case Bool(value) => value
    case Field("true") => true
    case Field("t") => true
    case Field("false") => false
    case Field("f") => false
    case other: Constant => throwIAE(s"not a bool: $other")
  } getOrElse default
}

case class AliasedField(field: Field, alias: String) extends Expr with FieldLike

case class Binary(left: Expr, symbol: OperatorSymbol, right: Expr) extends Expr

case class Unary(symbol: OperatorSymbol, right: Expr) extends Expr

case class Call(name: String, args: Seq[Expr] = Seq()) extends Expr

case class FieldIn(field: String, exprs: Seq[Expr]) extends Expr

case class Alias(expr: Expr, name: String) extends Expr with FieldLike with FieldOrAlias

sealed trait Command

case class SearchCommand(expr: Expr) extends Command

case class EvalCommand(fields: Seq[(Field, Expr)]) extends Command

case class FieldConversion(func: String, field: Field, alias: Option[Field]) extends Expr

case class ConvertCommand(timeformat: String = "%m/%d/%Y %H:%M:%S",
                          convs: Seq[FieldConversion]) extends Command

case class LookupOutput(kv: String, fields: Seq[FieldLike])

case class LookupCommand(dataset: String, fields: Seq[FieldLike],
                         output: Option[LookupOutput]) extends Command

case class CollectCommand(index: String,
                          fields: Seq[Field],
                          addTime: Boolean,
                          file: String,
                          host: String,
                          marker: String,
                          outputFormat: String,
                          runInPreview: Boolean,
                          spool: Boolean,
                          source: String,
                          sourceType: String,
                          testMode: Boolean) extends Command

case class WhereCommand(expr: Expr) extends Command

case class TableCommand(fields: Seq[Field]) extends Command

case class HeadCommand(evalExpr: Expr,
                       keepLast: Bool = Bool(false),
                       nullOption: Bool = Bool(false)) extends Command

case class FieldsCommand(removeFields: Boolean, fields: Seq[Field]) extends Command

case class SortCommand(fieldsToSort: Seq[(Option[String], Expr)]) extends Command

case class StatsCommand(partitions: Int,
                        allNum: Boolean,
                        delim: String,
                        funcs: Seq[Expr],
                        by: Seq[Field] = Seq(),
                        dedupSplitVals: Boolean = false) extends Command

case class RexCommand(field: Option[String],
                      maxMatch: Int,
                      offsetField: Option[String],
                      mode: Option[String],
                      regex: String) extends Command

case class RenameCommand(alias: Seq[Alias]) extends Command

case class RegexCommand(item: Option[(Field, String)], regex: String) extends Command

case class JoinCommand(joinType: String = "inner",
                       useTime: Boolean = false,
                       earlier: Boolean = true,
                       overwrite: Boolean = true,
                       max: Int = 1,
                       fields: Seq[Field],
                       subSearch: Pipeline) extends Command

case class ReturnCommand(count: IntValue, fields: Seq[FieldOrAlias]) extends Command

// TODO: Option[Seq[Value]] -> Seq[Value] = Seq()
case class FillNullCommand(value: Option[String], fields: Option[Seq[Field]]) extends Command

case class EventStatsCommand(allNum: Boolean, funcs: Seq[Expr],
                             by: Seq[Field] = Seq()) extends Command

case class StreamStatsCommand(funcs: Seq[Expr], by: Seq[Field] = Seq(), current: Boolean = true,
                              window: Int = 0) extends Command

case class DedupCommand(numResults: Int, fields: Seq[Field], keepEvents: Boolean,
                        keepEmpty: Boolean, consecutive: Boolean,
                        sortBy: SortCommand) extends Command

case class InputLookup(append: Boolean, strict: Boolean, start: Int, max: Int, tableName: String,
                       where: Option[Expr]) extends Command

case class FormatArgs(rowPrefix: String, colPrefix: String, colSep: String, colEnd: String,
                      rowSep: String, rowEnd: String)

case class FormatCommand(mvSep: String, maxResults: Int, rowPrefix: String, colPrefix: String,
                         colSep: String, colEnd: String, rowSep: String,
                         rowEnd: String) extends Command

case class MvCombineCommand(delim: Option[String], field: Field) extends Command

case class MvExpandCommand(field: Field, limit: Option[Int]) extends Command

case class MakeResults(count: Int, annotate: Boolean, server: String,
                       serverGroup: String) extends Command

case class AddTotals(fields: Seq[Field],
                     row: Boolean,
                     col: Boolean,
                     fieldName: String,
                     labelField: String,
                     label: String) extends Command

case class BinCommand(field: FieldOrAlias, span: Option[SplSpan] = None,
                      minSpan: Option[SplSpan] = None, bins: Option[Int] = None,
                      start: Option[Int] = None, end: Option[Int] = None,
                      alignTime: Option[String] = None) extends Command

case class MultiSearch(pipelines: Seq[Pipeline]) extends Command

case class MapCommand(search: Pipeline, maxSearches: Int) extends Command

case class Pipeline(commands: Seq[Command])
 */
use crate::commands::cmd_add_totals::spl::AddTotals;
use crate::commands::cmd_bin::spl::BinCommand;
use crate::commands::cmd_collect::spl::CollectCommand;
use crate::commands::cmd_convert::spl::{ConvertCommand, FieldConversion};
use crate::commands::cmd_dedup::spl::DedupCommand;
use crate::commands::cmd_eval::spl::EvalCommand;
use crate::commands::cmd_event_stats::spl::EventStatsCommand;
use crate::commands::cmd_fields::spl::FieldsCommand;
use crate::commands::cmd_fill_null::spl::FillNullCommand;
use crate::commands::cmd_format::spl::FormatCommand;
use crate::commands::cmd_head::spl::HeadCommand;
use crate::commands::cmd_input_lookup::spl::InputLookup;
use crate::commands::cmd_join::spl::JoinCommand;
use crate::commands::cmd_lookup::spl::LookupCommand;
use crate::commands::cmd_make_results::spl::MakeResults;
use crate::commands::cmd_map::spl::MapCommand;
use crate::commands::cmd_multi_search::spl::MultiSearch;
use crate::commands::cmd_mv_combine::spl::MvCombineCommand;
use crate::commands::cmd_mv_expand::spl::MvExpandCommand;
use crate::commands::cmd_regex::spl::RegexCommand;
use crate::commands::cmd_rename::spl::RenameCommand;
use crate::commands::cmd_return::spl::ReturnCommand;
use crate::commands::cmd_rex::spl::RexCommand;
use crate::commands::cmd_search::spl::SearchCommand;
use crate::commands::cmd_sort::spl::SortCommand;
use crate::commands::cmd_stats::spl::StatsCommand;
use crate::commands::cmd_stream_stats::spl::StreamStatsCommand;
use crate::commands::cmd_table::spl::TableCommand;
use crate::commands::cmd_top::spl::TopCommand;
use crate::commands::cmd_where::spl::WhereCommand;
use anyhow::anyhow;
use float_derive::FloatHash;
use pyo3::prelude::*;
use std::collections::HashMap;
use std::fmt::Debug;

/// Syntax tree element representing a null literal value.
#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct NullValue();

/// Syntax tree element representing a boolean literal value.
#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct BoolValue(pub bool);

impl<T: Into<bool>> From<T> for BoolValue {
    fn from(value: T) -> Self {
        BoolValue(value.into())
    }
}

/// Syntax tree element representing an integer literal value.
#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct IntValue(pub i64);

impl<T: Into<i64>> From<T> for IntValue {
    fn from(value: T) -> Self {
        IntValue(value.into())
    }
}

/// Syntax tree element representing a floating-point literal value.
#[derive(Debug, PartialEq, Clone, FloatHash)]
#[pyclass(frozen, eq, hash)]
pub struct DoubleValue(pub f64);

impl<T: Into<f64>> From<T> for DoubleValue {
    fn from(value: T) -> Self {
        DoubleValue(value.into())
    }
}

/// Syntax tree element representing a string literal value.
#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct StrValue(pub String);

impl<T: ToString> From<T> for StrValue {
    fn from(value: T) -> Self {
        StrValue(value.to_string())
    }
}

/// Syntax tree element representing a duration with a value and a scale.
///
/// # Fields
///
/// * `value` - An integer representing the duration of the time span.
/// * `scale` - A string representing the unit of the time span (e.g., "seconds", "minutes").
#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct TimeSpan {
    #[pyo3(get)]
    pub value: i64,
    #[pyo3(get)]
    pub scale: String,
}

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct SnapTime {
    #[pyo3(get)]
    pub span: Option<TimeSpan>,
    #[pyo3(get)]
    pub snap: String,
    #[pyo3(get)]
    pub snap_offset: Option<TimeSpan>,
}

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct Field(pub String);

impl<S: ToString> From<S> for Field {
    fn from(value: S) -> Field {
        Field(value.to_string())
    }
}

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct Wildcard(pub String);

impl<S: ToString> From<S> for Wildcard {
    fn from(value: S) -> Wildcard {
        Wildcard(value.to_string())
    }
}

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct Variable(pub String);

impl<S: ToString> From<S> for Variable {
    fn from(value: S) -> Variable {
        Variable(value.to_string())
    }
}

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct IPv4CIDR(pub String);

impl<S: ToString> From<S> for IPv4CIDR {
    fn from(value: S) -> IPv4CIDR {
        IPv4CIDR(value.to_string())
    }
}

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct FV {
    #[pyo3(get)]
    pub field: String,
    #[pyo3(get)]
    pub value: String,
}

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct FB {
    #[pyo3(get)]
    pub field: String,
    #[pyo3(get)]
    pub value: bool,
}

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct FC {
    #[pyo3(get)]
    pub field: String,
    #[pyo3(get)]
    pub value: Constant,
}

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct CommandOptions {
    #[pyo3(get)]
    pub options: Vec<FC>,
}

pub struct ParsedCommandOptions {
    inner: HashMap<String, Constant>,
}

impl From<CommandOptions> for ParsedCommandOptions {
    fn from(value: CommandOptions) -> Self {
        Self {
            inner: value
                .options
                .iter()
                .cloned()
                .map(|option| (option.field, option.value))
                .collect(),
        }
    }
}

/*
 private val inner = options.map(y => y.field -> y.value).toMap

 private def throwIAE(msg: String) = throw new IllegalArgumentException(msg)

 def toMap: Map[String, Constant] = inner
 def getIntOption(key: String): Option[Int] = inner.get(key) map {
   case IntValue(value) => value
   case other: Constant => throwIAE(s"not an int: $other")
 }

 def getInt(key: String, default: Int = 0): Int =
   getIntOption(key).getOrElse(default)

 def getStringOption(key: String): Option[String] = inner.get(key) map {
   case Field(v) => v
   case StrValue(v) => v
   case other: Constant => throwIAE(s"not a string: $other")
 }

 def getString(key: String, default: String): String =
   getStringOption(key).getOrElse(default)

 def getSpanOption(key: String): Option[SplSpan] = inner.get(key) map {
   case span: SplSpan => span
   case other: Constant => throwIAE(s"not a span: $other")
 }

 def getBoolean(key: String, default: Boolean = false): Boolean = inner.get(key) map {
   case Bool(value) => value
   case Field("true") => true
   case Field("t") => true
   case Field("false") => false
   case Field("f") => false
   case other: Constant => throwIAE(s"not a bool: $other")
 } getOrElse default
*/

impl ParsedCommandOptions {
    pub fn get_int_option(&self, key: &str) -> Result<Option<i64>, anyhow::Error> {
        match self.inner.get(key) {
            Some(Constant::Int(IntValue(value))) => Ok(Some(*value)),
            Some(_) => Err(anyhow!("not an int")),
            None => Ok(None),
        }
    }

    pub fn get_int(&self, key: &str, default: i64) -> Result<i64, anyhow::Error> {
        self.get_int_option(key).map(|v| v.unwrap_or(default))
    }

    pub fn get_string_option(&self, key: &str) -> Result<Option<String>, anyhow::Error> {
        match self.inner.get(key) {
            Some(Constant::Field(Field(value))) => Ok(Some(value.clone())),
            Some(Constant::Str(StrValue(value))) => Ok(Some(value.clone())),
            Some(_) => Err(anyhow!("not a string")),
            None => Ok(None),
        }
    }

    pub fn get_string(&self, key: &str, default: impl ToString) -> Result<String, anyhow::Error> {
        self.get_string_option(key)
            .map(|v| v.unwrap_or(default.to_string()))
    }

    pub fn get_span_option(&self, key: &str) -> Result<Option<SplSpan>, anyhow::Error> {
        match self.inner.get(key) {
            Some(Constant::SplSpan(span)) => Ok(Some(span.clone())),
            Some(_) => Err(anyhow!("not a span")),
            None => Ok(None),
        }
    }

    pub fn get_boolean(&self, key: &str, default: bool) -> Result<bool, anyhow::Error> {
        match self.inner.get(key) {
            Some(Constant::Bool(BoolValue(value))) => Ok(*value),
            Some(Constant::Field(Field(v))) if v == "true" => Ok(true),
            Some(Constant::Field(Field(v))) if v == "t" => Ok(true),
            Some(Constant::Field(Field(v))) if v == "false" => Ok(false),
            Some(Constant::Field(Field(v))) if v == "f" => Ok(false),
            Some(_) => Err(anyhow!("not a bool")),
            None => Ok(default),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct AliasedField {
    #[pyo3(get)]
    pub field: Field,
    #[pyo3(get)]
    pub alias: String,
}

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct Binary {
    // #[pyo3(get)]
    pub left: Box<Expr>,
    #[pyo3(get)]
    pub symbol: String,
    // #[pyo3(get)]
    pub right: Box<Expr>,
}

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct Unary {
    #[pyo3(get)]
    pub symbol: String,
    // #[pyo3(get)]
    pub right: Box<Expr>,
}

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct Call {
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub args: Vec<Expr>,
}

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct FieldIn {
    #[pyo3(get)]
    pub field: String,
    #[pyo3(get)]
    pub exprs: Vec<Expr>,
}

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct Alias {
    // #[pyo3(get)]
    pub expr: Box<Expr>,
    #[pyo3(get)]
    pub name: String,
}

/// A pipeline is a chain of commands where data is passed and processed by each command in turn.
#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct Pipeline {
    #[pyo3(get)]
    pub commands: Vec<Command>,
}

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub enum SplSpan {
    TimeSpan(TimeSpan),
}

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub enum Constant {
    Null(NullValue),
    Bool(BoolValue),
    Int(IntValue),
    Double(DoubleValue),
    Str(StrValue),
    SnapTime(SnapTime),
    SplSpan(SplSpan),
    Field(Field),
    Wildcard(Wildcard),
    Variable(Variable),
    IPv4CIDR(IPv4CIDR),
}

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub enum LeafExpr {
    Constant(Constant),
    FV(FV),
    FB(FB),
    FC(FC),
}

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub enum Expr {
    Leaf(LeafExpr),
    AliasedField(AliasedField),
    Binary(Binary),
    Unary(Unary),
    Call(Call),
    FieldIn(FieldIn),
    Alias(Alias),
    FieldConversion(FieldConversion),
}

impl Into<Constant> for TimeSpan {
    fn into(self) -> Constant {
        Constant::SplSpan(SplSpan::TimeSpan(self))
    }
}
impl Into<Constant> for BoolValue {
    fn into(self) -> Constant {
        Constant::Bool(self)
    }
}
impl Into<Constant> for IntValue {
    fn into(self) -> Constant {
        Constant::Int(self)
    }
}
impl Into<Constant> for DoubleValue {
    fn into(self) -> Constant {
        Constant::Double(self)
    }
}
impl Into<Constant> for StrValue {
    fn into(self) -> Constant {
        Constant::Str(self)
    }
}
impl Into<Constant> for SnapTime {
    fn into(self) -> Constant {
        Constant::SnapTime(self)
    }
}
impl Into<Constant> for Field {
    fn into(self) -> Constant {
        Constant::Field(self)
    }
}
impl Into<Constant> for Wildcard {
    fn into(self) -> Constant {
        Constant::Wildcard(self)
    }
}
impl Into<Constant> for Variable {
    fn into(self) -> Constant {
        Constant::Variable(self)
    }
}
impl Into<Constant> for IPv4CIDR {
    fn into(self) -> Constant {
        Constant::IPv4CIDR(self)
    }
}

impl Into<Expr> for TimeSpan {
    fn into(self) -> Expr {
        Expr::Leaf(LeafExpr::Constant(self.into()))
    }
}
impl Into<Expr> for BoolValue {
    fn into(self) -> Expr {
        Expr::Leaf(LeafExpr::Constant(self.into()))
    }
}
impl Into<Expr> for IntValue {
    fn into(self) -> Expr {
        Expr::Leaf(LeafExpr::Constant(self.into()))
    }
}
impl Into<Expr> for DoubleValue {
    fn into(self) -> Expr {
        Expr::Leaf(LeafExpr::Constant(self.into()))
    }
}
impl Into<Expr> for StrValue {
    fn into(self) -> Expr {
        Expr::Leaf(LeafExpr::Constant(self.into()))
    }
}
impl Into<Expr> for SnapTime {
    fn into(self) -> Expr {
        Expr::Leaf(LeafExpr::Constant(self.into()))
    }
}
impl Into<Expr> for Field {
    fn into(self) -> Expr {
        Expr::Leaf(LeafExpr::Constant(self.into()))
    }
}
impl Into<Expr> for Wildcard {
    fn into(self) -> Expr {
        Expr::Leaf(LeafExpr::Constant(self.into()))
    }
}
impl Into<Expr> for Variable {
    fn into(self) -> Expr {
        Expr::Leaf(LeafExpr::Constant(self.into()))
    }
}
impl Into<Expr> for IPv4CIDR {
    fn into(self) -> Expr {
        Expr::Leaf(LeafExpr::Constant(self.into()))
    }
}
impl Into<Expr> for FV {
    fn into(self) -> Expr {
        Expr::Leaf(LeafExpr::FV(self))
    }
}
impl Into<Expr> for FB {
    fn into(self) -> Expr {
        Expr::Leaf(LeafExpr::FB(self))
    }
}
impl Into<Expr> for FC {
    fn into(self) -> Expr {
        Expr::Leaf(LeafExpr::FC(self))
    }
}
impl Into<Expr> for AliasedField {
    fn into(self) -> Expr {
        Expr::AliasedField(self)
    }
}
impl Into<Expr> for Binary {
    fn into(self) -> Expr {
        Expr::Binary(self)
    }
}
impl Into<Expr> for Unary {
    fn into(self) -> Expr {
        Expr::Unary(self)
    }
}
impl Into<Expr> for Call {
    fn into(self) -> Expr {
        Expr::Call(self)
    }
}
impl Into<Expr> for FieldIn {
    fn into(self) -> Expr {
        Expr::FieldIn(self)
    }
}
impl Into<Expr> for Alias {
    fn into(self) -> Expr {
        Expr::Alias(self)
    }
}
impl Into<Expr> for FieldConversion {
    fn into(self) -> Expr {
        Expr::FieldConversion(self)
    }
}

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub enum FieldLike {
    Field(Field),
    Wildcard(Wildcard),
    AliasedField(AliasedField),
    Alias(Alias),
}

impl Into<FieldLike> for Field {
    fn into(self) -> FieldLike {
        FieldLike::Field(self)
    }
}
impl Into<FieldLike> for Wildcard {
    fn into(self) -> FieldLike {
        FieldLike::Wildcard(self)
    }
}
impl Into<FieldLike> for AliasedField {
    fn into(self) -> FieldLike {
        FieldLike::AliasedField(self)
    }
}
impl Into<FieldLike> for Alias {
    fn into(self) -> FieldLike {
        FieldLike::Alias(self)
    }
}

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub enum FieldOrAlias {
    Field(Field),
    Alias(Alias),
}

impl Into<FieldOrAlias> for Field {
    fn into(self) -> FieldOrAlias {
        FieldOrAlias::Field(self)
    }
}
impl Into<FieldOrAlias> for Alias {
    fn into(self) -> FieldOrAlias {
        FieldOrAlias::Alias(self)
    }
}

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub enum Command {
    AddTotals(AddTotals),
    BinCommand(BinCommand),
    CollectCommand(CollectCommand),
    ConvertCommand(ConvertCommand),
    DedupCommand(DedupCommand),
    EvalCommand(EvalCommand),
    EventStatsCommand(EventStatsCommand),
    FieldConversion(FieldConversion),
    FieldsCommand(FieldsCommand),
    FillNullCommand(FillNullCommand),
    FormatCommand(FormatCommand),
    HeadCommand(HeadCommand),
    InputLookup(InputLookup),
    JoinCommand(JoinCommand),
    LookupCommand(LookupCommand),
    MakeResults(MakeResults),
    MapCommand(MapCommand),
    MultiSearch(MultiSearch),
    MvCombineCommand(MvCombineCommand),
    MvExpandCommand(MvExpandCommand),
    Pipeline(Pipeline),
    RegexCommand(RegexCommand),
    RenameCommand(RenameCommand),
    ReturnCommand(ReturnCommand),
    RexCommand(RexCommand),
    SearchCommand(SearchCommand),
    SortCommand(SortCommand),
    StatsCommand(StatsCommand),
    StreamStatsCommand(StreamStatsCommand),
    TableCommand(TableCommand),
    TopCommand(TopCommand),
    WhereCommand(WhereCommand),
}

impl Into<Command> for SearchCommand {
    fn into(self) -> Command {
        Command::SearchCommand(self)
    }
}
impl Into<Command> for EvalCommand {
    fn into(self) -> Command {
        Command::EvalCommand(self)
    }
}
impl Into<Command> for FieldConversion {
    fn into(self) -> Command {
        Command::FieldConversion(self)
    }
}
impl Into<Command> for ConvertCommand {
    fn into(self) -> Command {
        Command::ConvertCommand(self)
    }
}
impl Into<Command> for LookupCommand {
    fn into(self) -> Command {
        Command::LookupCommand(self)
    }
}
impl Into<Command> for CollectCommand {
    fn into(self) -> Command {
        Command::CollectCommand(self)
    }
}
impl Into<Command> for WhereCommand {
    fn into(self) -> Command {
        Command::WhereCommand(self)
    }
}
impl Into<Command> for TableCommand {
    fn into(self) -> Command {
        Command::TableCommand(self)
    }
}
impl Into<Command> for TopCommand {
    fn into(self) -> Command {
        Command::TopCommand(self)
    }
}
impl Into<Command> for HeadCommand {
    fn into(self) -> Command {
        Command::HeadCommand(self)
    }
}
impl Into<Command> for FieldsCommand {
    fn into(self) -> Command {
        Command::FieldsCommand(self)
    }
}
impl Into<Command> for SortCommand {
    fn into(self) -> Command {
        Command::SortCommand(self)
    }
}
impl Into<Command> for StatsCommand {
    fn into(self) -> Command {
        Command::StatsCommand(self)
    }
}
impl Into<Command> for RexCommand {
    fn into(self) -> Command {
        Command::RexCommand(self)
    }
}
impl Into<Command> for RenameCommand {
    fn into(self) -> Command {
        Command::RenameCommand(self)
    }
}
impl Into<Command> for RegexCommand {
    fn into(self) -> Command {
        Command::RegexCommand(self)
    }
}
impl Into<Command> for JoinCommand {
    fn into(self) -> Command {
        Command::JoinCommand(self)
    }
}
impl Into<Command> for ReturnCommand {
    fn into(self) -> Command {
        Command::ReturnCommand(self)
    }
}
impl Into<Command> for FillNullCommand {
    fn into(self) -> Command {
        Command::FillNullCommand(self)
    }
}
impl Into<Command> for EventStatsCommand {
    fn into(self) -> Command {
        Command::EventStatsCommand(self)
    }
}
impl Into<Command> for StreamStatsCommand {
    fn into(self) -> Command {
        Command::StreamStatsCommand(self)
    }
}
impl Into<Command> for DedupCommand {
    fn into(self) -> Command {
        Command::DedupCommand(self)
    }
}
impl Into<Command> for InputLookup {
    fn into(self) -> Command {
        Command::InputLookup(self)
    }
}
impl Into<Command> for FormatCommand {
    fn into(self) -> Command {
        Command::FormatCommand(self)
    }
}
impl Into<Command> for MvCombineCommand {
    fn into(self) -> Command {
        Command::MvCombineCommand(self)
    }
}
impl Into<Command> for MvExpandCommand {
    fn into(self) -> Command {
        Command::MvExpandCommand(self)
    }
}
impl Into<Command> for MakeResults {
    fn into(self) -> Command {
        Command::MakeResults(self)
    }
}
impl Into<Command> for AddTotals {
    fn into(self) -> Command {
        Command::AddTotals(self)
    }
}
impl Into<Command> for BinCommand {
    fn into(self) -> Command {
        Command::BinCommand(self)
    }
}
impl Into<Command> for MultiSearch {
    fn into(self) -> Command {
        Command::MultiSearch(self)
    }
}
impl Into<Command> for MapCommand {
    fn into(self) -> Command {
        Command::MapCommand(self)
    }
}
impl Into<Command> for Pipeline {
    fn into(self) -> Command {
        Command::Pipeline(self)
    }
}
