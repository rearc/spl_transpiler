use crate::ast::ast;
use crate::pyspark::ast::*;
use crate::pyspark::transpiler::base::{PipelineTransformState, PipelineTransformer};
use anyhow::bail;

impl ast::EvalCommand {
    fn _eval_expr(&self, expr: ast::Expr) -> anyhow::Result<ColumnLike> {
        let expr: Expr = expr.try_into()?;
        match expr {
            Expr::Column(c) => Ok(c),
            _ => bail!("Rhs of eval expression evaluated to a non-columnar expression"),
        }
    }
}

impl PipelineTransformer for ast::EvalCommand {
    fn transform(&self, state: PipelineTransformState) -> anyhow::Result<PipelineTransformState> {
        let mut df = state.df;
        for (ast::Field(name), value) in self.fields.iter().cloned() {
            df = df.with_column(name, self._eval_expr(value)?)
        }
        Ok(PipelineTransformState { df })
    }
}
