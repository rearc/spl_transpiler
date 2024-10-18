use super::spl::*;
use crate::commands::cmd_data_model::spl::DataModelCommand;
use crate::functions::stat_fns::stats_fn;
use crate::pyspark::alias::Aliasable;
use crate::pyspark::ast::*;
use crate::pyspark::transpiler::{PipelineTransformState, PipelineTransformer};
use crate::spl::ast;
use anyhow::Result;
use anyhow::{anyhow, bail, ensure};
use log::warn;

fn transform_by_field(f: MaybeSpannedField) -> ColumnOrName {
    match f.clone() {
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
    }
}

fn transform_stats_expr(df: DataFrame, expr: &ast::Expr) -> Result<(DataFrame, ColumnLike)> {
    let (e, maybe_name) = expr.clone().unaliased_with_name();
    ensure!(
        matches!(e, ast::Expr::Call(_)),
        "All `stats` aggregations must be function calls"
    );
    let (df_, e) = stats_fn(e, df)?;
    Ok((df_, e.maybe_with_alias(maybe_name)))
}

impl TStatsCommand {
    fn get_df(
        &self,
        state: PipelineTransformState,
        allow_runtime: bool,
    ) -> Result<PipelineTransformState> {
        DataModelCommand {
            data_model_name: self.datamodel.clone(),
            dataset_name: self.nodename.clone(),
            search_mode: Some("search".to_string()),
            strict_fields: false,
            allow_old_summaries: self.allow_old_summaries,
            summaries_only: self.summaries_only,
        }
        .transform(state, allow_runtime)
    }
}

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

        let mut all_kwargs = py_dict! {};

        // Options
        all_kwargs.push("prestats", PyLiteral::from(self.prestats));
        all_kwargs.push("local", PyLiteral::from(self.local));
        all_kwargs.push("append", PyLiteral::from(self.append));
        all_kwargs.push(
            "include_reduced_buckets",
            PyLiteral::from(self.include_reduced_buckets),
        );
        if let Some(fill_null_value) = &self.fillnull_value {
            all_kwargs.push("fill_null_value", PyLiteral::from(fill_null_value.clone()));
        }

        // From statement
        let PipelineTransformState { mut df } = self.get_df(state, true)?;

        // Where statement
        if let Some(where_condition) = &self.where_condition {
            let where_condition: Expr = where_condition.clone().try_into()?;
            all_kwargs.push("where", where_condition);
        }

        // By statement
        if let Some(by_fields) = &self.by_fields {
            let by_fields: Vec<RuntimeExpr> = by_fields
                .iter()
                .cloned()
                .map(transform_by_field)
                .map(Into::into)
                .collect();
            all_kwargs.push("by", PyList(by_fields));
        }

        for e in self.exprs.iter() {
            let (df_, e) = transform_stats_expr(df, e)?;
            df = df_;
            let (e, maybe_name) = e.unaliased_with_name();
            let name = maybe_name.ok_or(anyhow!("All `stats` aggregations must have names"))?;
            all_kwargs.push(name, e);
        }

        df = DataFrame::runtime(Some(df), "tstats", vec![], all_kwargs.0);

        Ok(PipelineTransformState { df })
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

        let PipelineTransformState { mut df } = self.get_df(state, false)?;

        if let Some(where_condition) = self.where_condition.clone() {
            let condition: Expr = where_condition.try_into()?;
            df = df.where_(condition);
        }

        let mut aggs: Vec<ColumnLike> = vec![];
        for e in self.exprs.iter() {
            let (df_, e) = transform_stats_expr(df, e)?;
            df = df_;
            aggs.push(e);
        }
        df = match (self.by_fields.clone(), self.by_prefix.clone()) {
            (Some(by_fields), None) => {
                df.group_by(by_fields.into_iter().map(transform_by_field).collect())
            }
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
    use crate::pyspark::ast::DataFrame;
    use crate::pyspark::utils::test::{generates, generates_runtime};

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

    #[test]
    fn test_tstats_5() {
        DataFrame::reset_df_count();
        let query = r#"tstats
        summariesonly=false allow_old_summaries=true fillnull_value=null
        count min(_time) as firstTime max(_time) as lastTime
        from datamodel=Endpoint.Processes
        where (Processes.process_name ="7z.exe" OR Processes.process_name = "7za.exe" OR Processes.original_file_name = "7z.exe" OR Processes.original_file_name =  "7za.exe") AND (Processes.process="*\\C$\\*" OR Processes.process="*\\Admin$\\*" OR Processes.process="*\\IPC$\\*")
        by Processes.original_file_name Processes.parent_process_name Processes.parent_process Processes.process_name Processes.process Processes.parent_process_id Processes.process_id  Processes.dest Processes.user"#;

        generates_runtime(
            query,
            r#"
df_1 = commands.data_model(
    None,
    data_model_name="Endpoint.Processes",
    dataset_name=None,
    search_mode="search",
    strict_fields=False,
    allow_old_summaries=True,
    summaries_only=False
)
df_2 = commands.tstats(
    df_1,
    prestats=False,
    local=False,
    append=False,
    include_reduced_buckets=False,
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
    count=F.count(F.lit(1)),
    firstTime=F.min(F.col("_time")),
    lastTime=F.max(F.col("_time")),
)
df_2
            "#,
        )
    }
}
