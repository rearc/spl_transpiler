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
use crate::commands::cmd_add_totals::spl::AddTotalsCommand;
use crate::commands::cmd_bin::spl::BinCommand;
use crate::commands::cmd_collect::spl::CollectCommand;
use crate::commands::cmd_convert::spl::ConvertCommand;
use crate::commands::cmd_data_model::spl::DataModelCommand;
use crate::commands::cmd_dedup::spl::DedupCommand;
use crate::commands::cmd_eval::spl::EvalCommand;
use crate::commands::cmd_event_stats::spl::EventStatsCommand;
use crate::commands::cmd_fields::spl::FieldsCommand;
use crate::commands::cmd_fill_null::spl::FillNullCommand;
use crate::commands::cmd_format::spl::FormatCommand;
use crate::commands::cmd_head::spl::HeadCommand;
use crate::commands::cmd_input_lookup::spl::InputLookupCommand;
use crate::commands::cmd_join::spl::JoinCommand;
use crate::commands::cmd_lookup::spl::LookupCommand;
use crate::commands::cmd_make_results::spl::MakeResultsCommand;
use crate::commands::cmd_map::spl::MapCommand;
use crate::commands::cmd_multi_search::spl::MultiSearchCommand;
use crate::commands::cmd_mv_combine::spl::MvCombineCommand;
use crate::commands::cmd_mv_expand::spl::MvExpandCommand;
use crate::commands::cmd_rare::spl::RareCommand;
use crate::commands::cmd_regex::spl::RegexCommand;
use crate::commands::cmd_rename::spl::RenameCommand;
use crate::commands::cmd_return::spl::ReturnCommand;
use crate::commands::cmd_rex::spl::RexCommand;
use crate::commands::cmd_s_path::spl::SPathCommand;
use crate::commands::cmd_search::spl::SearchCommand;
use crate::commands::cmd_sort::spl::SortCommand;
use crate::commands::cmd_stats::spl::StatsCommand;
use crate::commands::cmd_stream_stats::spl::StreamStatsCommand;
use crate::commands::cmd_t_stats::spl::TStatsCommand;
use crate::commands::cmd_table::spl::TableCommand;
use crate::commands::cmd_tail::spl::TailCommand;
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

/// Syntax tree element representing an integer literal value.
#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct IntValue(pub i64);

/// Syntax tree element representing a floating-point literal value.
#[derive(Debug, PartialEq, Clone, FloatHash)]
#[pyclass(frozen, eq, hash)]
pub struct DoubleValue(pub f64);

/// Syntax tree element representing a string literal value.
#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct StrValue(pub String);

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

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct Wildcard(pub String);

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct Variable(pub String);

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct IPv4CIDR(pub String);

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

    pub fn get_span_option(&self, key: &str) -> Result<Option<TimeSpan>, anyhow::Error> {
        match self.inner.get(key) {
            Some(Constant::TimeSpan(span)) => Ok(Some(span.clone())),
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

impl Binary {
    pub fn new(left: impl Into<Expr>, symbol: impl ToString, right: impl Into<Expr>) -> Self {
        Self {
            left: Box::new(left.into()),
            symbol: symbol.to_string(),
            right: Box::new(right.into()),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct Unary {
    #[pyo3(get)]
    pub symbol: String,
    // #[pyo3(get)]
    pub right: Box<Expr>,
}

impl Unary {
    pub fn new(symbol: impl ToString, right: impl Into<Expr>) -> Self {
        Self {
            symbol: symbol.to_string(),
            right: Box::new(right.into()),
        }
    }
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

impl Alias {
    pub fn new(name: impl ToString, expr: impl Into<Expr>) -> Self {
        Self {
            name: name.to_string(),
            expr: Box::new(expr.into()),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub enum TimeModifier {
    StartTime(String),
    EndTime(String),
    Earliest(SnapTime),
    Latest(SnapTime),
}

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub struct FormattedTimeModifier {
    pub format: String,
    pub time_modifier: TimeModifier,
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
pub enum Constant {
    Null(NullValue),
    Bool(BoolValue),
    Int(IntValue),
    Double(DoubleValue),
    Str(StrValue),
    SnapTime(SnapTime),
    TimeSpan(TimeSpan),
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
pub enum SearchModifier {
    SourceType(String),
    Host(String),
    HostTag(String),
    EventType(String),
    EventTypeTag(String),
    SavedSplunk(String),
    Source(String),
    SplunkServer(String),
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
    TimeModifier(FormattedTimeModifier),
    SearchModifier(SearchModifier),
    SubSearch(Pipeline),
}

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub enum FieldLike {
    Field(Field),
    Wildcard(Wildcard),
    AliasedField(AliasedField),
    Alias(Alias),
}

#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub enum FieldOrAlias {
    Field(Field),
    Alias(Alias),
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, PartialEq, Clone, Hash)]
#[pyclass(frozen, eq, hash)]
pub enum Command {
    AddTotalsCommand(AddTotalsCommand),
    BinCommand(BinCommand),
    CollectCommand(CollectCommand),
    ConvertCommand(ConvertCommand),
    DataModelCommand(DataModelCommand),
    DedupCommand(DedupCommand),
    EvalCommand(EvalCommand),
    EventStatsCommand(EventStatsCommand),
    FieldsCommand(FieldsCommand),
    FillNullCommand(FillNullCommand),
    FormatCommand(FormatCommand),
    HeadCommand(HeadCommand),
    InputLookupCommand(InputLookupCommand),
    JoinCommand(JoinCommand),
    LookupCommand(LookupCommand),
    MakeResultsCommand(MakeResultsCommand),
    MapCommand(MapCommand),
    MultiSearchCommand(MultiSearchCommand),
    MvCombineCommand(MvCombineCommand),
    MvExpandCommand(MvExpandCommand),
    RareCommand(RareCommand),
    RegexCommand(RegexCommand),
    RenameCommand(RenameCommand),
    ReturnCommand(ReturnCommand),
    RexCommand(RexCommand),
    SearchCommand(SearchCommand),
    SPathCommand(SPathCommand),
    SortCommand(SortCommand),
    StatsCommand(StatsCommand),
    StreamStatsCommand(StreamStatsCommand),
    TableCommand(TableCommand),
    TailCommand(TailCommand),
    TopCommand(TopCommand),
    TStatsCommand(TStatsCommand),
    WhereCommand(WhereCommand),
}
