use crate::commands::cmd_where::spl::WhereCommand;
use crate::pyspark::ast::Expr;
use crate::pyspark::base::ToSparkExpr;
use crate::pyspark::transpiler::{PipelineTransformState, PipelineTransformer};

impl PipelineTransformer for WhereCommand {
    fn transform_standalone(
        &self,
        state: PipelineTransformState,
    ) -> anyhow::Result<PipelineTransformState> {
        let mut df = state.df.clone().unwrap_or_default();

        let condition: Expr = self.expr.clone().with_context(&state.ctx).try_into()?;
        df = df.where_(condition);

        Ok(state.with_df(df))
    }
}
