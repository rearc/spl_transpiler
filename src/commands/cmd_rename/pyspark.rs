use crate::commands::cmd_rename::spl::RenameCommand;
use crate::pyspark::transpiler::{PipelineTransformState, PipelineTransformer};
use crate::spl::ast;
use anyhow::bail;

impl PipelineTransformer for RenameCommand {
    fn transform(&self, state: PipelineTransformState) -> anyhow::Result<PipelineTransformState> {
        let mut df = state.df;

        for alias in self.alias.clone() {
            let old_name = match *alias.expr {
                ast::Expr::Leaf(ast::LeafExpr::Constant(ast::Constant::Field(ast::Field(
                    name,
                )))) => name,
                ast::Expr::Leaf(ast::LeafExpr::Constant(ast::Constant::Wildcard(
                    ast::Wildcard(_name),
                ))) => bail!("UNIMPLEMENTED: Wildcard renaming is not supported yet"),
                _ => bail!("Unsupported rename source: {:?}", alias),
            };
            df = df.with_column_renamed(old_name, alias.name.clone());
        }

        Ok(PipelineTransformState { df })
    }
}
