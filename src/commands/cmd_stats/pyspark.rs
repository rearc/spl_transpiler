use super::spl::*;
use crate::commands::stats_utils;
use crate::pyspark::alias::Aliasable;
use crate::pyspark::ast::ColumnLike::FunctionCall;
use crate::pyspark::ast::*;
use crate::pyspark::base::{PysparkTranspileContext, ToSparkExpr};
use crate::pyspark::transpiler::{PipelineTransformState, PipelineTransformer};
use crate::spl::ast;
use anyhow::bail;
use anyhow::Result;
use log::warn;

fn transform_stats_runtime_expr(
    expr: &ast::Expr,
    ctx: &PysparkTranspileContext,
) -> Result<(String, RuntimeExpr)> {
    let (e, maybe_name) = expr.clone().unaliased_with_name();
    let (name, args) = match e {
        ast::Expr::Call(ast::Call { name, args }) => (name.to_string(), args),
        _ => bail!(
            "All `stats` aggregations must be function calls, got {:?}",
            expr
        ),
    };
    let alias = maybe_name.unwrap_or(name.clone());
    let args: Result<Vec<Expr>> = args
        .into_iter()
        .map(|e| e.with_context(ctx).try_into())
        .collect();
    let expr: Expr = FunctionCall {
        func: format!("functions.stats.{}", name).to_string(),
        args: args?,
    }
    .into();
    Ok((alias, RuntimeExpr::from(expr)))
}

impl PipelineTransformer for StatsCommand {
    fn transform_for_runtime(
        &self,
        state: PipelineTransformState,
    ) -> Result<PipelineTransformState> {
        if self.partitions != 0 && self.partitions != 1 {
            warn!("`stats` `partitions` argument has no effect in PySpark")
        }
        if self.all_num {
            warn!("`stats` `all_num` argument has no effect in PySpark")
        }
        if self.dedup_split_vals {
            warn!("`stats` `dedup_split_vals` argument has no effect in PySpark")
        }
        if self.delim != " " {
            warn!("`stats` `delim` argument has no effect in PySpark")
        }

        let mut all_kwargs = py_dict! {};

        // By statement
        if let Some(by_fields) = &self.by {
            let by_fields: Vec<RuntimeExpr> = by_fields
                .iter()
                .cloned()
                .map(stats_utils::transform_by_field)
                .map(Into::into)
                .collect();
            all_kwargs.push("by", PyList(by_fields));
        }

        for e in self.funcs.iter() {
            let (name, e) = transform_stats_runtime_expr(e, &state.ctx)?;
            all_kwargs.push(name, e);
        }

        let df = DataFrame::runtime(state.df.clone(), "stats", vec![], all_kwargs.0, &state.ctx);

        Ok(state.with_df(df))
    }

    fn transform_standalone(
        &self,
        state: PipelineTransformState,
    ) -> Result<PipelineTransformState> {
        if self.partitions != 0 && self.partitions != 1 {
            warn!("`stats` `partitions` argument has no effect in PySpark")
        }
        if self.all_num {
            warn!("`stats` `all_num` argument has no effect in PySpark")
        }
        if self.dedup_split_vals {
            warn!("`stats` `dedup_split_vals` argument has no effect in PySpark")
        }
        if self.delim != " " {
            warn!("`stats` `delim` argument has no effect in PySpark")
        }

        let mut df = state.df.clone().unwrap_or_default();

        let mut aggs: Vec<ColumnLike> = vec![];
        for e in self.funcs.iter() {
            let (df_, e) = stats_utils::transform_stats_expr(df, e, &state.ctx)?;
            df = df_;
            aggs.push(e);
        }
        df = df.group_by(
            self.by
                .clone()
                .unwrap_or_default()
                .into_iter()
                .map(stats_utils::transform_by_field)
                .collect(),
        );

        df = df.agg(aggs);

        Ok(state.with_df(df))
    }
}

#[cfg(test)]
mod tests {
    use crate::pyspark::utils::test::{generates, generates_runtime};
    use rstest::rstest;

    #[rstest]
    fn test_xsl_script_execution_with_wmic_1() {
        generates(
            r#"stats
            count min(_time) as firstTime max(_time) as lastTime
            by Processes.parent_process_name Processes.parent_process Processes.process_name Processes.process_id Processes.process Processes.dest Processes.user"#,
            r#"spark.table("main").groupBy([
                "Processes.parent_process_name",
                "Processes.parent_process",
                "Processes.process_name",
                "Processes.process_id",
                "Processes.process",
                "Processes.dest",
                "Processes.user",
            ]).agg(
                F.count(F.lit(1)).alias("count"),
                F.min(F.col("_time")).alias("firstTime"),
                F.max(F.col("_time")).alias("lastTime"),
            )"#,
        )
    }

    #[rstest]
    fn test_stats_2() {
        generates(
            r#"stats
            count min(_time) as firstTime max(_time) as lastTime
            by Web.http_user_agent, Web.status Web.http_method, Web.url, Web.url_length, Web.src, Web.dest, sourcetype"#,
            r#"spark.table("main").groupBy([
                "Web.http_user_agent",
                "Web.status",
                "Web.http_method",
                "Web.url",
                "Web.url_length",
                "Web.src",
                "Web.dest",
                "sourcetype",
            ]).agg(
                F.count(F.lit(1)).alias("count"),
                F.min(F.col("_time")).alias("firstTime"),
                F.max(F.col("_time")).alias("lastTime"),
            )
            "#,
        )
    }

    #[rstest]
    fn test_stats_3() {
        let query = r#"stats
        count min(_time) AS firstTime max(_time) AS lastTime
        BY _time span=1h Processes.user Processes.process_id Processes.process_name Processes.process Processes.process_path Processes.dest Processes.parent_process_name Processes.parent_process Processes.process_guid"#;

        generates(
            query,
            r#"spark.table("main").groupBy([
                F.window("_time", "1 hours"),
                "Processes.user",
                "Processes.process_id",
                "Processes.process_name",
                "Processes.process",
                "Processes.process_path",
                "Processes.dest",
                "Processes.parent_process_name",
                "Processes.parent_process",
                "Processes.process_guid",
            ]).agg(
                F.count(F.lit(1)).alias("count"),
                F.min(F.col("_time")).alias("firstTime"),
                F.max(F.col("_time")).alias("lastTime"),
            )
            "#,
        )
    }

    #[rstest]
    fn test_stats_4() {
        let query = r#"stats
        count min(_time) as firstTime max(_time) as lastTime
        by Processes.original_file_name Processes.parent_process_name Processes.parent_process Processes.process_name Processes.process Processes.parent_process_id Processes.process_id  Processes.dest Processes.user"#;

        // TODO: The `like` strings should be r strings or escaped
        generates(
            query,
            r#"spark.table("main").groupBy([
                "Processes.original_file_name",
                "Processes.parent_process_name",
                "Processes.parent_process",
                "Processes.process_name",
                "Processes.process",
                "Processes.parent_process_id",
                "Processes.process_id",
                "Processes.dest",
                "Processes.user",
            ]).agg(
                F.count(F.lit(1)).alias("count"),
                F.min(F.col("_time")).alias("firstTime"),
                F.max(F.col("_time")).alias("lastTime"),
            )
            "#,
        )
    }

    #[rstest]
    fn test_stats_5() {
        let query = r#"stats
        count min(_time) as firstTime max(_time) as lastTime
        by _time span=1h Processes.original_file_name Processes.parent_process_name Processes.parent_process Processes.process_name Processes.process Processes.parent_process_id Processes.process_id  Processes.dest Processes.user"#;

        generates_runtime(
            query,
            r#"
df_1 = commands.stats(
    None,
    by=[
        F.window("_time", "1 hours"),
        F.col("Processes.original_file_name"),
        F.col("Processes.parent_process_name"),
        F.col("Processes.parent_process"),
        F.col("Processes.process_name"),
        F.col("Processes.process"),
        F.col("Processes.parent_process_id"),
        F.col("Processes.process_id"),
        F.col("Processes.dest"),
        F.col("Processes.user"),
    ],
    count=functions.stats.count(),
    firstTime=functions.stats.min(F.col("_time")),
    lastTime=functions.stats.max(F.col("_time")),
)
df_1
            "#,
        )
    }
}
