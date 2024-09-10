use crate::commands::cmd_rex::spl::RexCommand;
use crate::commands::regex_utils::get_groups;
use crate::pyspark::ast::*;
use crate::pyspark::transpiler::{PipelineTransformState, PipelineTransformer};
use anyhow::ensure;

impl PipelineTransformer for RexCommand {
    fn transform(&self, state: PipelineTransformState) -> anyhow::Result<PipelineTransformState> {
        let mut df = state.df;

        ensure!(
            self.mode != Some("sed".into()),
            "sed-mode rex commands are not yet supported."
        );
        ensure!(
            self.max_match == 1,
            "Rex not yet implemented for multiple matches"
        );

        let regex_groups = get_groups(self.regex.clone())?;

        for (group_index, group_name) in regex_groups {
            df = df.with_column(
                group_name.unwrap_or(group_index.to_string()),
                column_like!(regexp_extract(
                    [col(self.field.clone())],
                    [Expr::Raw(Raw(format!("r\"{}\"", self.regex)))],
                    [py_lit(group_index as i64)]
                )),
            );
        }

        Ok(PipelineTransformState { df })
    }
}
