use crate::commands::cmd_join::spl::JoinCommand;
use crate::pyspark::ast::*;
use crate::pyspark::transpiler::utils::join_as_binaries;
use crate::pyspark::transpiler::{PipelineTransformState, PipelineTransformer};
use crate::spl::ast;
use anyhow::{anyhow, bail, ensure};

impl PipelineTransformer for JoinCommand {
    #[allow(unused_variables, unreachable_code)]
    fn transform(&self, state: PipelineTransformState) -> anyhow::Result<PipelineTransformState> {
        let df = state.df.alias("LEFT");

        ensure!(
            self.max == 1,
            "UNIMPLEMENTED: Join with max != 1 not yet supported"
        );

        let right_df: TransformedPipeline = self.sub_search.clone().try_into()?;
        let right_df = right_df
            .dataframes
            .first()
            .ok_or(anyhow!("No dataframe found for sub_search"))?
            .alias("RIGHT");

        let join_type = match self.join_type.clone().as_str() {
            "inner" => "inner",
            "left" => "left",
            "outer" => "outer",
            _ => bail!("Unsupported join type: {}", self.join_type),
        };

        let condition = join_as_binaries(
            "&",
            self.fields
                .clone()
                .into_iter()
                .map(|ast::Field(name)| {
                    column_like!(
                        [col(format!("LEFT.{}", name))] == [col(format!("RIGHT.{}", name))]
                    )
                })
                .collect(),
        )
        .unwrap();

        let condition = match (self.use_time, self.earlier) {
            (true, true) => {
                column_like!([condition] & [[col("LEFT._time")] >= [col("RIGHT._time")]])
            }
            (true, false) => {
                column_like!([condition] & [[col("LEFT._time")] <= [col("RIGHT._time")]])
            }
            (false, _) => condition,
        };

        let df = df.join(right_df, condition, join_type);

        Ok(PipelineTransformState { df })
    }
}

#[cfg(test)]
mod tests {
    use crate::pyspark::utils::test::*;

    #[test]
    fn test_join_1() {
        generates(
            r#"join product_id [search vendors]"#,
            r#"spark.table('main').alias("LEFT").join(
                spark.table('main').where(F.col("_raw").ilike("%vendors%")).alias("RIGHT"),
                (F.col("LEFT.product_id") == F.col("RIGHT.product_id")),
                "inner"
            )"#,
        )
    }
}
