use crate::commands::cmd_head::spl::HeadCommand;
use crate::pyspark::transpiler::{PipelineTransformState, PipelineTransformer};
use crate::spl::ast;
use anyhow::bail;

impl PipelineTransformer for HeadCommand {
    fn transform(&self, state: PipelineTransformState) -> anyhow::Result<PipelineTransformState> {
        let mut df = state.df;

        let limit = match self.eval_expr.clone() {
            ast::Expr::Leaf(ast::LeafExpr::Constant(ast::Constant::Int(ast::IntValue(limit)))) => {
                limit
            }
            ast::Expr::Binary(ast::Binary {
                left,
                symbol,
                right,
            }) => {
                let (was_left, limit_expr) = match (*left, *right) {
                    (ast::Expr::Leaf(ast::LeafExpr::Constant(ast::Constant::Field(ast::Field(name)))), right) if name == "count" => (true, right),
                    (left, ast::Expr::Leaf(ast::LeafExpr::Constant(ast::Constant::Field(ast::Field(name))))) if name == "count" => (false, left),
                    _ => bail!("Unable to find a comparison against `count`, no other head expression is expected")
                };
                let limit_value = match limit_expr {
                    ast::Expr::Leaf(ast::LeafExpr::Constant(ast::Constant::Int(
                        ast::IntValue(limit),
                    ))) => limit,
                    _ => bail!("Unable to find a constant integer being compared against `count`"),
                };
                let limit = match (was_left, symbol.as_str()) {
                    (true, "<") => limit_value,
                    (true, "<=") => limit_value + 1,
                    (false, ">") => limit_value,
                    (false, ">=") => limit_value + 1,
                    _ => bail!(
                        "Unsupported `count` comparison ({}, {}) for head command",
                        was_left,
                        symbol
                    ),
                } + if self.keep_last.0 { 1 } else { 0 };
                limit
            }
            _ => bail!(
                "Limit for head command must be an integer (or equivalent `count<N` expression)"
            ),
        };

        df = df.limit(limit as u64);
        Ok(PipelineTransformState { df })
    }
}
