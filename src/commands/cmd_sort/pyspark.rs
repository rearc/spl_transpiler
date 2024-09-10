use crate::commands::cmd_sort::spl::SortCommand;
use crate::pyspark::ast::*;
use crate::pyspark::transpiler::{PipelineTransformState, PipelineTransformer};
use crate::spl::ast;
use anyhow::{bail, Result};

/// Converts a generic column-like expression into an explicitly ascending or descending ordering
fn _ascending_or_descending(
    c: ColumnLike,
    sign: Option<String>,
    descending: bool,
) -> Result<ColumnLike> {
    let descending = match (sign, descending) {
        (None, descending) => descending,
        (Some(sign), descending) => match sign.as_str() {
            "+" => descending,
            "-" => !descending,
            _ => bail!("Invalid sort sign `{}`", sign),
        },
    };
    Ok(if descending {
        column_like!([c].desc())
    } else {
        column_like!([c].asc())
    })
}

/// Resolves the sort expression to something we know how to handle
fn _resolve_expr(e: ast::Expr) -> Result<ColumnLike> {
    match e {
        ast::Expr::Leaf(ast::LeafExpr::Constant(ast::Constant::Field(ast::Field(name)))) => {
            Ok(column_like!(col(name)))
        }
        ast::Expr::Call(ast::Call { name, .. }) => match name.as_str() {
            "auto" => unimplemented!(),
            "ip" => unimplemented!(),
            "num" => unimplemented!(),
            "str" => unimplemented!(),
            _ => bail!("Unsupported sort function: {}", name),
        },
        _ => bail!("Unsupported sort expression: {:?}", e),
    }
}

impl PipelineTransformer for SortCommand {
    fn transform(&self, state: PipelineTransformState) -> Result<PipelineTransformState> {
        let mut df = state.df;

        let sort_fields: Result<Vec<_>> = self
            .fields_to_sort
            .iter()
            .cloned()
            .map(|(sign, expr)| {
                let col = _resolve_expr(expr)?;
                _ascending_or_descending(col, sign, self.descending)
            })
            .collect();

        df = df.order_by(sort_fields?);

        if self.count != 0 {
            df = df.limit(self.count as u64);
        }

        Ok(PipelineTransformState { df })
    }
}
