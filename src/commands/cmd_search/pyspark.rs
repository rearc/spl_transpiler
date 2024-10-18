use super::spl::*;
use crate::pyspark::ast::*;
use crate::pyspark::transpiler::{PipelineTransformState, PipelineTransformer};
use crate::spl::ast;
use anyhow::{bail, ensure, Result};
use std::collections::HashSet;

fn _is_index(expr: &ast::Expr) -> bool {
    matches!(
        expr,
        ast::Expr::Leaf(ast::LeafExpr::Constant(ast::Constant::Field(ast::Field(name)))) if name == "index"
    )
}

fn split_conditions(
    expr: &ast::Expr,
    all_ands: bool,
    indices: &mut HashSet<String>,
) -> Result<Option<ast::Expr>> {
    match expr.clone() {
        // index=lol should result in Source("lol")
        ast::Expr::Binary(ast::Binary {
            left,
            symbol,
            right,
        }) => {
            let left_is_index = _is_index(left.as_ref());
            let right_is_index = _is_index(right.as_ref());
            match (left_is_index, symbol.as_str(), right_is_index) {
                (true, "=", false) | (false, "=", true) => {
                    ensure!(all_ands, "Cannot specify an index under an OR branch");
                    let compare_value = if left_is_index { *right } else { *left };
                    indices.insert(compare_value.try_into()?);
                    Ok(None)
                }
                (true, _, true) | (true, _, _) | (_, _, true) => {
                    bail!("Invalid index comparison: {:?}", expr)
                }
                (false, op, false) => {
                    let still_all_and = all_ands && op == "AND";
                    let converted_left = split_conditions(left.as_ref(), still_all_and, indices)?;
                    let converted_right = split_conditions(right.as_ref(), still_all_and, indices)?;
                    match (converted_left, op, converted_right) {
                        (None, _, None) => Ok(None),
                        (Some(left), "AND", None) | (Some(left), "OR", None) => Ok(Some(left)),
                        (None, "AND", Some(right)) | (None, "OR", Some(right)) => Ok(Some(right)),
                        (None, _, _) | (_, _, None) => bail!("Cannot perform comparison {} when one side collapses into an index check", op),
                        (Some(left), symbol, Some(right)) => Ok(Some(ast::Binary::new(
                            left,
                            symbol,
                            right,
                        ).into()))
                    }
                }
            }
        }
        exp => Ok(Some(exp)),
    }
}

fn break_down_ands(expr: &ast::Expr, exprs: &mut Vec<ast::Expr>) {
    match expr {
        ast::Expr::Binary(ast::Binary {
            left,
            symbol,
            right,
        }) if symbol == "AND" => {
            break_down_ands(left, exprs);
            break_down_ands(right, exprs);
        }
        _ => exprs.push(expr.clone()),
    }
}

impl PipelineTransformer for SearchCommand {
    fn transform_for_runtime(
        &self,
        state: PipelineTransformState,
    ) -> Result<PipelineTransformState> {
        let mut exprs = vec![];
        break_down_ands(&self.expr, &mut exprs);

        let maybe_df = match state.df {
            DataFrame::Source { .. } => None,
            df_ => Some(df_),
        };

        let mut args = vec![];
        let mut kwargs = vec![];

        for expr in exprs {
            match expr.clone() {
                ast::Expr::Binary(ast::Binary {
                    left,
                    symbol,
                    right,
                }) => match (*left.clone(), symbol.as_str()) {
                    (
                        ast::Expr::Leaf(ast::LeafExpr::Constant(ast::Constant::Field(ast::Field(
                            name,
                        )))),
                        "=" | "==",
                    ) => kwargs.push((name.to_string(), Expr::try_from(*right)?.into())),
                    _ => args.push(Expr::try_from(expr)?.into()),
                },
                _ => args.push(Expr::try_from(expr)?.into()),
            }
        }

        let df = DataFrame::runtime(maybe_df, "search", args, kwargs);

        Ok(PipelineTransformState { df })
    }

    fn transform_standalone(
        &self,
        state: PipelineTransformState,
    ) -> Result<PipelineTransformState> {
        let mut indices = HashSet::new();
        let condition_expr = split_conditions(&self.expr, true, &mut indices)?;
        let mut df = if !indices.is_empty() {
            let mut _df: Option<DataFrame> = None;
            for new_index in indices.into_iter() {
                let new_source = DataFrame::source(new_index);
                _df = match (_df, new_source) {
                    (None, new_source) => Some(new_source),
                    (Some(cur_source), new_source) => Some(cur_source.union_by_name(new_source)),
                }
            }
            _df.unwrap()
        } else {
            state.df.clone()
        };

        df = match condition_expr {
            None => df,
            Some(condition) => {
                let condition: Expr = condition.try_into()?;
                df.where_(condition.into_search_expr())
            }
        };

        Ok(PipelineTransformState { df })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pyspark::utils::test::{generates, generates_runtime};

    fn check_split_results(
        expr: impl Into<ast::Expr>,
        mut expected_indices: Vec<String>,
        expected_condition: Option<Expr>,
    ) {
        let mut indices = HashSet::new();
        let expr = expr.into();
        let condition: Option<ast::Expr> = split_conditions(&expr, true, &mut indices).unwrap();
        let converted_condition: Option<Expr> = condition.map(|e| e.try_into().unwrap());
        let mut indices: Vec<_> = indices.into_iter().collect();
        indices.sort();
        expected_indices.sort();
        assert_eq!(indices, expected_indices);
        assert_eq!(converted_condition, expected_condition);
    }

    #[test]
    fn test_split_conditions_simple_index() {
        check_split_results(
            ast::Binary::new(ast::Field::from("index"), "=", ast::Field::from("lol")),
            vec!["lol".into()],
            None,
        );
    }

    #[test]
    fn test_split_conditions_no_index() {
        check_split_results(
            ast::Binary::new(ast::Field::from("x"), "=", ast::IntValue::from(2)),
            Vec::<String>::new(),
            Some(column_like!([col("x")] == [lit(2)]).into()),
        );
    }

    #[test]
    fn test_split_conditions_combined_index() {
        check_split_results(
            ast::Binary::new(
                ast::Binary::new(ast::Field::from("x"), "=", ast::IntValue::from(2)),
                "AND",
                ast::Binary::new(ast::Field::from("index"), "=", ast::Field::from("lol")),
            ),
            vec!["lol".into()],
            Some(column_like!([col("x")] == [lit(2)]).into()),
        );
    }

    #[test]
    fn test_multi_index_conditions() {
        check_split_results(
            ast::Binary::new(
                ast::Binary::new(
                    ast::Binary::new(
                        ast::Binary::new(ast::Field::from("index"), "=", ast::Field::from("lol")),
                        "AND",
                        ast::Binary::new(ast::Field::from("x"), "=", ast::IntValue::from(2)),
                    ),
                    "AND",
                    ast::Binary::new(ast::Field::from("index"), "=", ast::Field::from("two")),
                ),
                "AND",
                ast::Binary::new(ast::Field::from("y"), ">", ast::IntValue::from(3)),
            ),
            vec!["lol".into(), "two".into()],
            Some(column_like!([[col("x")] == [lit(2)]] & [[col("y")] > [lit(3)]]).into()),
        );
    }

    #[test]
    fn test_search_8() {
        let query = r#"search
        query!="SELECT * FROM Win32_ProcessStartTrace WHERE ProcessName = 'wsmprovhost.exe'"
        AND query!="SELECT * FROM __InstanceOperationEvent WHERE TargetInstance ISA 'AntiVirusProduct' OR TargetInstance ISA 'FirewallProduct' OR TargetInstance ISA 'AntiSpywareProduct'"
        "#;

        generates(
            query,
            r#"
            spark.table("main").where(
                (
                    ~F.col("query").like("SELECT % FROM Win32_ProcessStartTrace WHERE ProcessName = 'wsmprovhost.exe'") &
                    ~F.col("query").like("SELECT % FROM __InstanceOperationEvent WHERE TargetInstance ISA 'AntiVirusProduct' OR TargetInstance ISA 'FirewallProduct' OR TargetInstance ISA 'AntiSpywareProduct'")
                )
            )
            "#,
        )
    }

    #[test]
    fn test_search_9() {
        DataFrame::reset_df_count();
        let query = r#"index="lol" sourcetype="src1" len(x)=3"#;

        generates_runtime(
            query,
            r#"
df_1 = commands.search(None, (F.length(F.col("x")) == F.lit(3)), index=F.lit("lol"), sourcetype=F.lit("src1"))
df_1
            "#,
        )
    }
}
