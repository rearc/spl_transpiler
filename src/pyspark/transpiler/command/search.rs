use crate::ast::ast;
use crate::pyspark::ast::*;
use crate::pyspark::transpiler::base::{PipelineTransformState, PipelineTransformer};
use anyhow::anyhow;

impl PipelineTransformer for ast::SearchCommand {
    fn transform(&self, state: PipelineTransformState) -> anyhow::Result<PipelineTransformState> {
        match self.expr.clone() {
            // index=lol should result in Source("lol")
            ast::Expr::Binary(ast::Binary {
                left,
                symbol,
                right,
            }) if symbol == "=" && *left == ast::Field::from("index").into() => match *right {
                ast::Expr::Leaf(ast::LeafExpr::Constant(ast::Constant::Field(ast::Field(
                    name,
                )))) => Ok(PipelineTransformState {
                    df: DataFrame::source(name),
                }),
                _ => Err(anyhow!("Unsupported index assignment: {:?}", right)),
            },
            exp => {
                let condition: Expr = exp.try_into()?;
                Ok(PipelineTransformState {
                    df: state.df.where_(condition),
                })
            }
        }
    }
}
