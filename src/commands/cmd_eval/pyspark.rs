use super::spl::*;
use crate::pyspark::alias::Aliasable;
use crate::pyspark::ast::*;
use crate::pyspark::transpiler::{PipelineTransformState, PipelineTransformer};
use crate::spl::ast;
use anyhow::{bail, Result};

impl EvalCommand {
    fn _eval_expr(&self, expr: ast::Expr) -> anyhow::Result<ColumnLike> {
        let expr: Expr = expr.try_into()?;
        match expr {
            Expr::Column(c) => Ok(c),
            _ => bail!("Rhs of eval expression evaluated to a non-columnar expression"),
        }
    }
}

impl PipelineTransformer for EvalCommand {
    fn transform_for_runtime(
        &self,
        _state: PipelineTransformState,
    ) -> anyhow::Result<PipelineTransformState> {
        let kwargs: Result<Vec<(String, RuntimeExpr)>> = self
            .fields
            .iter()
            .map(|(field, expr)| {
                Ok((
                    field.0.to_string(),
                    RuntimeExpr::from(self._eval_expr(expr.clone())?.unaliased()),
                ))
            })
            .collect();

        let df = DataFrame::runtime(Some(_state.df), "eval", vec![], kwargs?);

        Ok(PipelineTransformState { df })
    }

    fn transform_standalone(
        &self,
        state: PipelineTransformState,
    ) -> anyhow::Result<PipelineTransformState> {
        let mut df = state.df;
        for (ast::Field(name), value) in self.fields.iter().cloned() {
            df = df.with_column(name, self._eval_expr(value)?)
        }
        Ok(PipelineTransformState { df })
    }
}

#[cfg(test)]
mod tests {
    use crate::pyspark::utils::test::generates_runtime;

    #[test]
    fn test_eval_1() {
        generates_runtime(
            r#"index="main" | eval x=len(raw)"#,
            r#"
df_1 = commands.search(None, index=F.lit("main"))
df_2 = commands.eval(df_1, x=F.length(F.col("raw")))
df_2
            "#,
        )
    }
}
