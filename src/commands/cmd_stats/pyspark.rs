use super::spl::*;
use crate::pyspark::ast::*;
use crate::pyspark::transpiler::{PipelineTransformState, PipelineTransformer};
use crate::spl::ast;
use anyhow::anyhow;

fn _stats_func(func: &ast::Expr, mut df: DataFrame) -> anyhow::Result<(DataFrame, ColumnLike)> {
    let expr = match func {
        ast::Expr::Alias(ast::Alias { expr, name }) => {
            let (df_, expr) = _stats_func(expr, df)?;
            df = df_;
            Ok(match expr {
                ColumnLike::Aliased { col, .. } => column_like!([*col].alias(name)),
                col => column_like!([col.clone()].alias(name)),
            })
        }
        // count() -> `count(1).alias("count")`
        ast::Expr::Call(ast::Call { name, args }) if name == "count" && args.is_empty() => {
            Ok(column_like!([count([lit(1)])].alias("count")))
        }
        // sum(x) -> `sum(x).alias("sum")`
        ast::Expr::Call(ast::Call { name, args }) if name == "sum" && args.len() == 1 => {
            match &args[0] {
                ast::Expr::Leaf(ast::LeafExpr::Constant(ast::Constant::Field(ast::Field(
                    name,
                )))) => Ok(column_like!([sum([col(name)])].alias("sum"))),
                _ => Err(anyhow!("Unsupported stats sum argument: {:?}", args[0])),
            }
        }
        // values(x) -> `collect_set(x).alias("values")`
        ast::Expr::Call(ast::Call { name, args }) if name == "values" && args.len() == 1 => {
            match &args[0] {
                ast::Expr::Leaf(ast::LeafExpr::Constant(ast::Constant::Field(ast::Field(
                    name,
                )))) => Ok(column_like!([collect_set([col(name)])].alias("values"))),
                _ => Err(anyhow!("Unsupported stats values argument: {:?}", args[0])),
            }
        }
        // earliest(x) -> `(order_by(_time)) first(x).alias("earliest")`
        ast::Expr::Call(ast::Call { name, args }) if name == "earliest" && args.len() == 1 => {
            match &args[0] {
                ast::Expr::Leaf(ast::LeafExpr::Constant(ast::Constant::Field(ast::Field(
                    name,
                )))) => {
                    df = df.order_by(vec![column_like!([col("_time")].asc())]);
                    Ok(column_like!(
                        [first([col(name)], [py_lit(true)])].alias("earliest")
                    ))
                }
                _ => Err(anyhow!(
                    "Unsupported stats earliest argument: {:?}",
                    args[0]
                )),
            }
        }
        // latest(x) -> `(order_by(time)) last(x).alias("earliest")`
        ast::Expr::Call(ast::Call { name, args }) if name == "latest" && args.len() == 1 => {
            match &args[0] {
                ast::Expr::Leaf(ast::LeafExpr::Constant(ast::Constant::Field(ast::Field(
                    name,
                )))) => {
                    df = df.order_by(vec![column_like!([col("_time")].asc())]);
                    Ok(column_like!(
                        [last([col(name)], [py_lit(true)])].alias("latest")
                    ))
                }
                _ => Err(anyhow!(
                    "Unsupported stats earliest argument: {:?}",
                    args[0]
                )),
            }
        }
        _ => Err(anyhow!("Unimplemented stats function: {:?}", func)),
    }?;
    Ok((df, expr))
}

impl PipelineTransformer for StatsCommand {
    fn transform(&self, state: PipelineTransformState) -> anyhow::Result<PipelineTransformState> {
        let mut df = state.df;
        let mut aggs: Vec<ColumnLike> = vec![];
        for e in self.funcs.iter() {
            let (df_, expr) = _stats_func(e, df)?;
            df = df_;
            aggs.push(expr);
        }
        let groupby_columns = self.by.iter().map(|f| f.0.clone()).collect();
        df = df.group_by(groupby_columns);
        df = df.agg(aggs);

        Ok(PipelineTransformState { df })
    }
}
