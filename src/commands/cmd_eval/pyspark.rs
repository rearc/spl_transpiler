use super::spl::*;
use crate::pyspark::alias::Aliasable;
use crate::pyspark::ast::*;
use crate::pyspark::base::ToSparkExpr;
use crate::pyspark::transpiler::{PipelineTransformState, PipelineTransformer};
use crate::spl::ast;
use anyhow::{bail, Result};

impl EvalCommand {
    fn _eval_expr(&self, expr: impl TryInto<Expr, Error = anyhow::Error>) -> Result<ColumnLike> {
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
        state: PipelineTransformState,
    ) -> Result<PipelineTransformState> {
        let kwargs: Result<Vec<(String, RuntimeExpr)>> = self
            .fields
            .iter()
            .map(|(field, expr)| {
                Ok((
                    field.0.to_string(),
                    RuntimeExpr::from(
                        self._eval_expr(expr.clone().with_context(&state.ctx))?
                            .unaliased(),
                    ),
                ))
            })
            .collect();

        let df = DataFrame::runtime(state.df.clone(), "eval", vec![], kwargs?, &state.ctx);

        Ok(state.with_df(df))
    }

    fn transform_standalone(
        &self,
        state: PipelineTransformState,
    ) -> Result<PipelineTransformState> {
        let mut df = state.df.clone().unwrap_or_default();
        for (ast::Field(name), value) in self.fields.iter().cloned() {
            df = df.with_column(name, self._eval_expr(value.with_context(&state.ctx))?)
        }
        Ok(state.with_df(df))
    }
}

#[cfg(test)]
mod tests {
    use crate::pyspark::utils::test::generates_runtime;
    use rstest::rstest;

    #[rstest]
    fn test_eval_1() {
        generates_runtime(
            r#"index="main" | eval x=len(raw)"#,
            r#"
df_1 = commands.search(None, index="main")
df_2 = commands.eval(df_1, x=functions.eval.len_(F.col("raw")))
df_2
            "#,
        )
    }
}
