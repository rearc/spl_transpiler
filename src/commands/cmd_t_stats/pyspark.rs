use super::spl::*;
use crate::commands::cmd_data_model::spl::DataModelCommand;
use crate::commands::stats_utils;
use crate::pyspark::ast::*;
use crate::pyspark::base::ToSparkExpr;
use crate::pyspark::transpiler::{PipelineTransformState, PipelineTransformer};
use anyhow::Result;
use anyhow::{bail, ensure};
use log::warn;

impl PipelineTransformer for TStatsCommand {
    fn transform_for_runtime(
        &self,
        state: PipelineTransformState,
    ) -> Result<PipelineTransformState> {
        if self.chunk_size != 10000000 {
            warn!("`tstats` `chunk_size` argument has no effect in PySpark")
        }
        if !self.summaries_only {
            warn!("`tstats` `summariesonly` argument has no effect in PySpark")
        }
        if !self.allow_old_summaries {
            warn!("`tstats` `allow_old_summaries` argument has no effect in PySpark")
        }
        // all_kwargs.push("prestats", PyLiteral::from(self.prestats));
        // all_kwargs.push("local", PyLiteral::from(self.local));
        // all_kwargs.push("append", PyLiteral::from(self.append));
        // all_kwargs.push(
        //     "include_reduced_buckets",
        //     PyLiteral::from(self.include_reduced_buckets),
        // );
        ensure!(!self.prestats, "`tstats` only supports `prestats`=false");
        ensure!(!self.local, "`tstats` only supports `local`=false");
        ensure!(!self.append, "`tstats` only supports `append`=false");
        ensure!(
            !self.include_reduced_buckets,
            "`tstats` only supports `include_reduced_buckets`=false"
        );

        let mut all_kwargs = py_dict! {};

        // Options
        all_kwargs.push(
            "from_",
            py_dict! {
                datamodel=self.datamodel.clone().map(PyLiteral::from),
                nodename=self.nodename.clone().map(PyLiteral::from),
            },
        );

        if let Some(fill_null_value) = &self.fillnull_value {
            all_kwargs.push("fill_null_value", PyLiteral::from(fill_null_value.clone()));
        }

        // From statement
        // let PipelineTransformState { mut df } = self.get_df(state, true)?;

        // Where statement
        if let Some(where_condition) = &self.where_condition {
            let where_condition: Expr = where_condition
                .clone()
                .with_context(&state.ctx)
                .try_into()?;
            all_kwargs.push("where", where_condition);
        }

        // By statement
        if let Some(by_fields) = &self.by_fields {
            let by_fields: Vec<RuntimeExpr> = by_fields
                .iter()
                .cloned()
                .map(stats_utils::transform_by_field)
                .map(Into::into)
                .collect();
            all_kwargs.push("by", PyList(by_fields));
        }

        for e in self.exprs.iter() {
            let (name, e) = stats_utils::transform_stats_runtime_expr(e, &state.ctx)?;
            all_kwargs.push(name, e);
        }

        let df = DataFrame::runtime(state.df.clone(), "tstats", vec![], all_kwargs.0, &state.ctx);

        Ok(state.with_df(df))
    }

    fn transform_standalone(
        &self,
        state: PipelineTransformState,
    ) -> Result<PipelineTransformState> {
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

        let state = DataModelCommand {
            data_model_name: self.datamodel.clone(),
            dataset_name: self.nodename.clone(),
            search_mode: Some("search".to_string()),
            strict_fields: false,
            allow_old_summaries: self.allow_old_summaries,
            summaries_only: self.summaries_only,
        }
        .transform(state)?;
        let mut df = state.df.clone().unwrap_or_default();

        if let Some(where_condition) = self.where_condition.clone() {
            let condition: Expr = where_condition.with_context(&state.ctx).try_into()?;
            df = df.where_(condition);
        }

        let mut aggs: Vec<ColumnLike> = vec![];
        for e in self.exprs.iter() {
            let (df_, e) = stats_utils::transform_stats_expr(df, e, &state.ctx)?;
            df = df_;
            aggs.push(e);
        }
        df = match (self.by_fields.clone(), self.by_prefix.clone()) {
            (Some(by_fields), None) => df.group_by(
                by_fields
                    .into_iter()
                    .map(stats_utils::transform_by_field)
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

    #[rstest]
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

    #[rstest]
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

    #[rstest]
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

    #[rstest]
    fn test_tstats_5() {
        let query = r#"tstats
        summariesonly=false allow_old_summaries=true fillnull_value=null
        count min(_time) as firstTime max(_time) as lastTime
        from datamodel=Endpoint.Processes
        where (Processes.process_name ="7z.exe" OR Processes.process_name = "7za.exe" OR Processes.original_file_name = "7z.exe" OR Processes.original_file_name =  "7za.exe") AND (Processes.process="*\\C$\\*" OR Processes.process="*\\Admin$\\*" OR Processes.process="*\\IPC$\\*")
        by _time span=1h Processes.original_file_name Processes.parent_process_name Processes.parent_process Processes.process_name Processes.process Processes.parent_process_id Processes.process_id  Processes.dest Processes.user"#;

        generates_runtime(
            query,
            r#"
df_1 = commands.tstats(
    None,
    from_={"datamodel": "Endpoint.Processes", "nodename": None},
    fill_null_value="null",
    where=(
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
    ),
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
