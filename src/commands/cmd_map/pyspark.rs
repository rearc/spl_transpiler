use crate::commands::cmd_map::spl::MapCommand;
use crate::pyspark::transpiler::{PipelineTransformState, PipelineTransformer};

impl PipelineTransformer for MapCommand {
    fn transform(&self, _state: PipelineTransformState) -> anyhow::Result<PipelineTransformState> {
        unimplemented!("Need some way to figure out what non-index search terms (left- and right-hand sides) exist in the subquery")
        //     generates("map search=\"search index=fake_for_join id=$id$\"",
        //       """(spark.table('fake_for_join')
        //         |.limit(10).alias('l')
        //         |.join(spark.table('main').alias('r'),
        //         |(F.col('l.id') == F.col('r.id')),  <-- these column names come from id=$id$
        //         |'left_semi'))
        //         |""".stripMargin)
        //   }

        // let mut df = state.df;
        //
        // let sub_pipeline: TransformedPipeline = self.search.clone().try_into()?;
        // let sub_df: DataFrame = sub_pipeline.try_into()?;
        //
        // let left = sub_df.limit(self.max_searches as u64).alias("l");
        // let right = df.alias("r");
        //
        // let df = left.join(
        //     right,
        //     column_like!([col("l.id")] == [col("r.id")]),
        //     "left_semi",
        // );
        //
        // Ok(PipelineTransformState { df })
    }
}
