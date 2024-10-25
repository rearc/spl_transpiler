use crate::pyspark::ast::*;
use crate::pyspark::transpiler::utils::join_as_binaries;
use crate::pyspark::transpiler::{PipelineTransformState, PipelineTransformer};

impl PipelineTransformer for super::spl::AddTotalsCommand {
    fn transform_standalone(
        &self,
        state: PipelineTransformState,
    ) -> anyhow::Result<PipelineTransformState> {
        let cast_columns: Vec<ColumnLike> = self
            .fields
            .iter()
            .map(|field| {
                let name = field.0.clone();
                column_like!([when(
                    [[[col(name.clone())].cast([py_lit("double")])].isNotNull()],
                    [col(name)]
                )]
                .otherwise([lit(0.0)]))
            })
            .collect();

        let total: ColumnLike =
            join_as_binaries("+", cast_columns).unwrap_or(column_like!(lit(0.0)));

        let df = state
            .df
            .clone()
            .unwrap_or_default()
            .with_column(self.field_name.clone(), total);
        Ok(state.with_df(df))
    }
}
