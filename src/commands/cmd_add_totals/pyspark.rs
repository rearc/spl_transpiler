use crate::pyspark::ast::*;
use crate::pyspark::transpiler::utils::join_as_binaries;
use crate::pyspark::transpiler::{PipelineTransformState, PipelineTransformer};

impl PipelineTransformer for super::spl::AddTotals {
    fn transform(&self, state: PipelineTransformState) -> anyhow::Result<PipelineTransformState> {
        let cast_columns: Vec<ColumnLike> = self
            .fields
            .iter()
            .map(|field| {
                let name = field.0.clone();
                column_like!([when(
                    [[[col(name)].cast([py_lit("double")])].isNotNull()],
                    [col(name)]
                )]
                .otherwise([lit(0.0)]))
            })
            .collect();

        let total: ColumnLike = join_as_binaries("+", cast_columns, column_like!(lit(0.0)));

        Ok(PipelineTransformState {
            df: state.df.with_column(self.field_name.clone(), total),
        })
    }
}
