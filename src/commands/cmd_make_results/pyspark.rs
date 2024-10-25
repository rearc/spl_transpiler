use crate::pyspark::ast::*;
use crate::pyspark::transpiler::{PipelineTransformState, PipelineTransformer};

impl PipelineTransformer for super::spl::MakeResultsCommand {
    #[allow(unused_variables, unreachable_code)]
    fn transform_standalone(
        &self,
        state: PipelineTransformState,
    ) -> anyhow::Result<PipelineTransformState> {
        // let df = state.df.clone().unwrap_or_default();

        /*
            spark.range(0, 10, 1)
        //         |.withColumn('_raw', F.lit(None))
        //         |.withColumn('_time', F.current_timestamp())
        //         |.withColumn('host', F.lit(None))
        //         |.withColumn('source', F.lit(None))
        //         |.withColumn('sourcetype', F.lit(None))
        //         |.withColumn('splunk_server', F.lit('local'))
        //         |.withColumn('splunk_server_group', F.lit(None))
             */
        let mut df = DataFrame::raw_source(format!("spark.range(0, {}, 1)", self.count))
            .with_column("_time", column_like!(current_timestamp()));
        if self.annotate {
            df = df
                .with_column("_raw", column_like!(lit(None)))
                .with_column("host", column_like!(lit(None)))
                .with_column("source", column_like!(lit(None)))
                .with_column("sourcetype", column_like!(lit(None)))
                .with_column(
                    "splunk_server",
                    column_like!(lit(format!("\"{}\"", self.server.clone()))),
                )
                .with_column(
                    "splunk_server_group",
                    match self.server_group.clone() {
                        Some(group) => column_like!(lit(group)),
                        None => column_like!(lit(None)),
                    },
                );
        }

        Ok(state.with_df(df))
    }
}
