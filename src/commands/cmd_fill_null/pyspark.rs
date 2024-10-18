use crate::commands::cmd_fill_null::spl::FillNullCommand;
use crate::pyspark::ast::*;
use crate::pyspark::transpiler::{PipelineTransformState, PipelineTransformer};

impl PipelineTransformer for FillNullCommand {
    #[allow(unused_variables, unreachable_code)]
    fn transform_standalone(
        &self,
        state: PipelineTransformState,
    ) -> anyhow::Result<PipelineTransformState> {
        let df = state.df;

        let df = df.arbitrary_method(
            "fillna",
            vec![column_like!(py_lit(self.value.clone())).into()],
        );

        Ok(PipelineTransformState { df })
    }
}

#[cfg(test)]
mod tests {
    use crate::pyspark::utils::test::generates;

    #[test]
    fn test_fill_null_1() {
        generates(
            r#"fillnull value=NULL"#,
            r#"spark.table('main').fillna("NULL")"#,
        )
    }
}
