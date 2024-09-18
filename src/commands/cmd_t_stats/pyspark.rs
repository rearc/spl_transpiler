use super::spl::*;
use crate::functions::stat_fns::stats_fn;
use crate::pyspark::alias::Aliasable;
use crate::pyspark::ast::*;
use crate::pyspark::transpiler::{PipelineTransformState, PipelineTransformer};
use crate::spl::ast;
use anyhow::{bail, ensure};
use log::warn;

impl PipelineTransformer for TStatsCommand {
    fn transform(&self, state: PipelineTransformState) -> anyhow::Result<PipelineTransformState> {
        ensure!(
            !self.prestats,
            "UNIMPLEMENTED: `tstats` command does not implement prestats=true"
        );
        ensure!(
            !self.local,
            "UNIMPLEMENTED: `tstats` command does not implement local=true"
        );
        ensure!(
            !self.include_reduced_buckets,
            "UNIMPLEMENTED: `tstats` command does not implement include_reduced_buckets=true"
        );
        ensure!(
            !self.append,
            "UNIMPLEMENTED: `tstats` command does not implement append=true"
        );
        if self.chunk_size != 10000000 {
            warn!("`tstats` `chunk_size` argument has no effect in PySpark")
        }
        if !self.summaries_only {
            warn!("`tstats` `summariesonly` argument has no effect in PySpark")
        }
        if !self.allow_old_summaries {
            warn!("`tstats` `allow_old_summaries` argument has no effect in PySpark")
        }

        let mut df = match (state.df, self.datamodel.clone(), self.nodename.clone()) {
            (src, None, None) => src,
            (DataFrame::Source { .. }, data_model, node_name) => DataFrame::source(match (data_model, node_name) {
                (Some(data_model), None) => data_model,
                (Some(data_model), Some(node_name)) => {
                    // data_model = <data_model_name>.<root_dataset_name>
                    // node_name = <root_dataset_name>.<...>.<target_dataset_name>
                    // return <data_model_name>.<root_dataset_name>.<...>.<target_dataset_name>
                    let truncated_node_name = node_name.split(".").skip(1).collect::<Vec<_>>().join(".");
                    format!("{}.{}", data_model, truncated_node_name)
                },
                _ => bail!("I think we received a nodename ({:?}) with no datamodel ({:?}) in `tstats`?", self.nodename, self.datamodel),
            }),
            _ => bail!("UNIMPLEMENTED: `tstats` command requires a source DataFrame and either a data model or a node name"),
        };

        if let Some(where_condition) = self.where_condition.clone() {
            let condition: Expr = where_condition.try_into()?;
            df = df.where_(condition);
        }

        let mut aggs: Vec<ColumnLike> = vec![];
        for e in self.exprs.iter() {
            let (e, maybe_name) = e.clone().unaliased_with_name();
            ensure!(
                matches!(e, ast::Expr::Call(_)),
                "All `stats` aggregations must be function calls"
            );
            let (df_, expr) = stats_fn(e, df)?;
            df = df_;
            aggs.push(expr.maybe_with_alias(maybe_name));
        }
        df = match (self.by_fields.clone(), self.by_prefix.clone()) {
            (Some(by_fields), None) => df.group_by(by_fields.iter().map(|f| f.0.clone()).collect()),
            (None, Some(_)) => {
                bail!("UNIMPLEMENTED: `tstats` command does not yet support BY prefix")
            }
            (None, None) => df.group_by(vec![]),
            (Some(_), Some(_)) => {
                bail!("Cannot specify both fields and a prefix for `tstats` BY clause")
            }
        };

        df = df.agg(aggs);

        Ok(PipelineTransformState { df })
    }
}
