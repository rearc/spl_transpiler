use super::spl::*;
use crate::functions::stat_fns::stats_fn;
use crate::pyspark::alias::Aliasable;
use crate::pyspark::ast::*;
use crate::pyspark::transpiler::{PipelineTransformState, PipelineTransformer};
use crate::spl::ast;
use anyhow::ensure;

impl PipelineTransformer for StatsCommand {
    fn transform(&self, state: PipelineTransformState) -> anyhow::Result<PipelineTransformState> {
        let mut df = state.df;
        let mut aggs: Vec<ColumnLike> = vec![];
        for e in self.funcs.iter() {
            let (e, maybe_name) = e.clone().unaliased_with_name();
            ensure!(
                matches!(e, ast::Expr::Call(_)),
                "All `stats` aggregations must be function calls"
            );
            let (df_, expr) = stats_fn(e, df)?;
            df = df_;
            aggs.push(expr.maybe_with_alias(maybe_name));
        }
        let groupby_columns = self.by.iter().map(|f| f.0.clone()).collect();
        df = df.group_by(groupby_columns);
        df = df.agg(aggs);

        Ok(PipelineTransformState { df })
    }
}
