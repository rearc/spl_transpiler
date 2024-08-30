use crate::ast::ast;
use crate::pyspark::transpiler::base::{PipelineTransformState, PipelineTransformer};
use anyhow::anyhow;

mod add_totals;
mod bin;
mod convert;
mod convert_fns;
mod eval;
pub mod eval_fns;
mod fields;
mod map;
mod multisearch;
mod search;
mod stats;

impl PipelineTransformer for ast::Command {
    fn transform(&self, state: PipelineTransformState) -> anyhow::Result<PipelineTransformState> {
        match self {
            ast::Command::SearchCommand(command) => command.transform(state),
            ast::Command::EvalCommand(command) => command.transform(state),
            // ast::Command::FieldConversion(command) => command.transform(state),
            ast::Command::ConvertCommand(command) => command.transform(state),
            // ast::Command::LookupCommand(command) => command.transform(state),
            // ast::Command::CollectCommand(command) => command.transform(state),
            // ast::Command::WhereCommand(command) => command.transform(state),
            // ast::Command::TableCommand(command) => command.transform(state),
            // ast::Command::HeadCommand(command) => command.transform(state),
            ast::Command::FieldsCommand(command) => command.transform(state),
            // ast::Command::SortCommand(command) => command.transform(state),
            ast::Command::StatsCommand(command) => command.transform(state),
            // ast::Command::RexCommand(command) => command.transform(state),
            // ast::Command::RenameCommand(command) => command.transform(state),
            // ast::Command::RegexCommand(command) => command.transform(state),
            // ast::Command::JoinCommand(command) => command.transform(state),
            // ast::Command::ReturnCommand(command) => command.transform(state),
            // ast::Command::FillNullCommand(command) => command.transform(state),
            // ast::Command::EventStatsCommand(command) => command.transform(state),
            // ast::Command::StreamStatsCommand(command) => command.transform(state),
            // ast::Command::DedupCommand(command) => command.transform(state),
            // ast::Command::InputLookup(command) => command.transform(state),
            // ast::Command::FormatCommand(command) => command.transform(state),
            // ast::Command::MvCombineCommand(command) => command.transform(state),
            // ast::Command::MvExpandCommand(command) => command.transform(state),
            // ast::Command::MakeResults(command) => command.transform(state),
            ast::Command::AddTotals(command) => command.transform(state),
            ast::Command::BinCommand(command) => command.transform(state),
            ast::Command::MultiSearch(command) => command.transform(state),
            // ast::Command::MapCommand(command) => command.transform(state),
            // ast::Command::Pipeline(command) => command.transform(state),
            _ => Err(anyhow!("Unsupported command in pipeline: {:?}", self)),
        }
    }
}
