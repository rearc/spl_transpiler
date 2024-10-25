use crate::commands::cmd_lookup::spl::{LookupCommand, LookupOutput};
use crate::pyspark::ast::*;
use crate::pyspark::transpiler::utils::join_as_binaries;
use crate::pyspark::transpiler::{PipelineTransformState, PipelineTransformer};
use crate::spl::ast;
use crate::spl::ast::FieldLike;
use anyhow::{bail, Result};

impl PipelineTransformer for LookupCommand {
    #[allow(unused_variables, unreachable_code)]
    fn transform_standalone(
        &self,
        state: PipelineTransformState,
    ) -> anyhow::Result<PipelineTransformState> {
        let mut df = state.df.clone().unwrap_or_default();

        df = df.alias("main");
        let lookup_df = DataFrame::source(self.dataset.clone()).alias("lookup");

        let join_columns: Result<Vec<_>> = self
            .fields
            .iter()
            .map(|f| match f.clone() {
                FieldLike::AliasedField(ast::AliasedField { field, alias }) => Ok(column_like!(
                    [col(format!("main.{}", alias))] == [col(format!("lookup.{}", field.0))]
                )),
                FieldLike::Field(ast::Field(name)) => Ok(column_like!(
                    [col(format!("main.{}", name))] == [col(format!("lookup.{}", name))]
                )),
                _ => bail!(
                    "UNIMPLEMENTED: Unsupported lookup field definition: {:?}",
                    f
                ),
            })
            .collect();
        let join_condition = join_as_binaries("&", join_columns?).unwrap_or(column_like!(lit(1)));

        let joined_df = df.join(lookup_df, join_condition, "left");

        let result_df = joined_df.select(match self.output.clone() {
            None => {
                vec![column_like!(col("main.*")), column_like!(col("lookup.*"))]
            }
            Some(LookupOutput { kv, fields }) if kv.to_ascii_uppercase() == "OUTPUT" => {
                let mut cols = vec![column_like!(col("main.*"))];
                for field in fields {
                    cols.push(match field {
                        FieldLike::Field(ast::Field(name)) => {
                            column_like!([col(format!("lookup.{}", name))].alias(name))
                        }
                        FieldLike::AliasedField(ast::AliasedField { field, alias }) => {
                            column_like!([col(format!("lookup.{}", field.0))].alias(alias))
                        }
                        // FieldLike::Alias(_) => {}
                        _ => bail!(
                            "UNIMPLEMENTED: Unsupported lookup output definition: {:?}",
                            field
                        ),
                    });
                }
                cols
            }
            Some(LookupOutput { kv, fields }) if kv.to_ascii_uppercase() == "OUTPUTNEW" => {
                bail!("UNIMPLEMENTED: `lookup` command with `OUTPUTNEW` not supported")
            }
            output => bail!("Unsupported output definition for `lookup`: {:?}", output),
        });

        Ok(state.with_df(df))
    }
}
