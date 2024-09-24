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
                }
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
            (Some(by_fields), None) => df.group_by(
                by_fields
                    .iter()
                    .map(|f| match f.clone() {
                        MaybeSpannedField {
                            field: ast::Field(field),
                            span: None,
                        } => ColumnOrName::Name(field),
                        MaybeSpannedField {
                            field: ast::Field(field),
                            span: Some(span),
                        } => {
                            let span_text = format!("{} {}", span.value, span.scale);
                            column_like!(window([py_lit(field)], [py_lit(span_text)])).into()
                        }
                    })
                    .collect(),
            ),
            (None, Some(_)) => {
                bail!("UNIMPLEMENTED: `tstats` command does not yet support BY prefix")
            }
            (None, None) => df.group_by(Vec::<String>::new()),
            (Some(_), Some(_)) => {
                bail!("Cannot specify both fields and a prefix for `tstats` BY clause")
            }
        };

        df = df.agg(aggs);

        Ok(PipelineTransformState { df })
    }
}

#[cfg(test)]
mod tests {
    use crate::pyspark::utils::test::generates;

    #[test]
    fn test_xsl_script_execution_with_wmic_1() {
        generates(
            r#"tstats
            summariesonly=false allow_old_summaries=true fillnull_value=null
            count min(_time) as firstTime max(_time) as lastTime
            from datamodel=Endpoint.Processes
            where (Processes.process_name=wmic.exe OR Processes.original_file_name=wmic.exe) Processes.process = "*os get*" Processes.process="*/format:*" Processes.process = "*.xsl*"
            by Processes.parent_process_name Processes.parent_process Processes.process_name Processes.process_id Processes.process Processes.dest Processes.user"#,
            r#"spark.table("Endpoint.Processes").where(
                (
                    (
                        (
                            (
                                (F.col("Processes.process_name") == F.lit("wmic.exe"))
                                | (F.col("Processes.original_file_name") == F.lit("wmic.exe"))
                            )
                            & F.col("Processes.process").like("%os get%")
                        )
                        & F.col("Processes.process").like("%/format:%")
                    )
                    & F.col("Processes.process").like("%.xsl%")
                )
            ).groupBy([
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

    #[test]
    fn test_tstats_2() {
        generates(
            r#"tstats
            count min(_time) as firstTime max(_time) as lastTime
            from datamodel=Web
            where Web.url IN ("/AHT/AhtApiService.asmx/AuthUser") Web.status=200 Web.http_method=POST
            by Web.http_user_agent, Web.status Web.http_method, Web.url, Web.url_length, Web.src, Web.dest, sourcetype"#,
            r#"spark.table("Web").where(
                (
                    (
                        (F.col("Web.url") == "/AHT/AhtApiService.asmx/AuthUser")
                        & (F.col("Web.status") == F.lit(200))
                    ) & (F.col("Web.http_method") == F.lit("POST"))
                )
            ).groupBy([
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

    #[test]
    fn test_tstats_3() {
        let query = r#"tstats
        summariesonly=false allow_old_summaries=true fillnull_value=null
        count min(_time) AS firstTime max(_time)
        AS lastTime
        FROM datamodel=Endpoint.Processes
        BY _time span=1h Processes.user Processes.process_id Processes.process_name Processes.process Processes.process_path Processes.dest Processes.parent_process_name Processes.parent_process Processes.process_guid"#;

        generates(
            query,
            r#"spark.table("Endpoint.Processes").groupBy([
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

    #[test]
    fn test_tstats_4() {
        let query = r#"tstats
        summariesonly=false allow_old_summaries=true fillnull_value=null
        count min(_time) as firstTime max(_time) as lastTime
        from datamodel=Endpoint.Processes
        where (Processes.process_name ="7z.exe" OR Processes.process_name = "7za.exe" OR Processes.original_file_name = "7z.exe" OR Processes.original_file_name =  "7za.exe") AND (Processes.process="*\\C$\\*" OR Processes.process="*\\Admin$\\*" OR Processes.process="*\\IPC$\\*")
        by Processes.original_file_name Processes.parent_process_name Processes.parent_process Processes.process_name Processes.process Processes.parent_process_id Processes.process_id  Processes.dest Processes.user"#;

        // TODO: The `like` strings should be r strings or escaped
        generates(
            query,
            r#"spark.table("Endpoint.Processes").where(
                (
                    (
                        (
                            (
                                (F.col("Processes.process_name") == F.lit("7z.exe")) |
                                (F.col("Processes.process_name") == F.lit("7za.exe"))
                            ) | (F.col("Processes.original_file_name") == F.lit("7z.exe"))
                        ) | (F.col("Processes.original_file_name") == F.lit("7za.exe"))
                    ) & (
                        (
                            F.col("Processes.process").like("%\\C$\\%")
                            | F.col("Processes.process").like("%\\Admin$\\%")
                        ) | F.col("Processes.process").like("%\\IPC$\\%")
                    )
                )
            ).groupBy([
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
}
