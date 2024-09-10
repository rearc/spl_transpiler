use crate::commands::cmd_where::spl::WhereCommand;
use crate::pyspark::ast::Expr;
use crate::pyspark::transpiler::{PipelineTransformState, PipelineTransformer};

impl PipelineTransformer for WhereCommand {
    fn transform(&self, state: PipelineTransformState) -> anyhow::Result<PipelineTransformState> {
        let mut df = state.df;

        let condition: Expr = self.expr.clone().try_into()?;
        df = df.where_(condition);

        Ok(PipelineTransformState { df })
    }
}
