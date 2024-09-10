use super::spl::*;
use crate::pyspark::ast::*;
use crate::pyspark::transpiler::{PipelineTransformState, PipelineTransformer};
use anyhow::bail;

impl PipelineTransformer for TopCommand {
    fn transform(&self, state: PipelineTransformState) -> anyhow::Result<PipelineTransformState> {
        let mut df = state.df;

        if self.use_other {
            bail!("UNIMPLEMENTED: No implementation yet for `useother=true` in `top`")
        }

        let groupby_fields: Vec<_> = self
            .by
            .clone()
            .unwrap_or(self.fields.clone())
            .iter()
            .map(|f| f.0.clone())
            .collect();
        df = df.group_by(groupby_fields);

        df = df.agg(vec![
            column_like!([count()].alias(self.count_field.clone())),
        ]);

        if self.show_percent {
            df = df.with_column(
                self.percent_field.clone(),
                column_like!([col("count")] / [sum([col("count")])]),
            )
        }

        df = df.order_by(vec![column_like!(desc([col("count")]))]);

        if self.n != 0 {
            df = df.limit(self.n);
        }

        Ok(PipelineTransformState { df })
    }
}
