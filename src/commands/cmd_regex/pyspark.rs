use crate::commands::cmd_regex::spl::RegexCommand;
use crate::pyspark::ast::*;
use crate::pyspark::transpiler::{PipelineTransformState, PipelineTransformer};
use crate::spl::ast;
use anyhow::bail;

impl PipelineTransformer for RegexCommand {
    fn transform_standalone(
        &self,
        state: PipelineTransformState,
    ) -> anyhow::Result<PipelineTransformState> {
        let mut df = state.df.clone().unwrap_or_default();

        let (field, invert) = match self.item.clone() {
            None => ("_raw".to_string(), false),
            Some((ast::Field(field), comparison)) => match comparison.as_str() {
                "=" => (field, false),
                "!=" => (field, true),
                _ => bail!("Invalid regex comparison: {}", comparison),
            },
        };

        let mut col = column_like!(regexp_like(
            [col(field)],
            [Expr::PyLiteral(PyLiteral(format!("r\"{}\"", self.regex)))]
        ));
        if invert {
            col = column_like!(~[col]);
        }

        df = df.where_(col);

        Ok(state.with_df(df))
    }
}
