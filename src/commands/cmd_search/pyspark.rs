use super::spl::*;
use crate::pyspark::ast::*;
use crate::pyspark::base::{PysparkTranspileContext, ToSparkExpr};
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
    ctx: &PysparkTranspileContext,
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
                    indices.insert(compare_value.with_context(ctx).try_into()?);
                    Ok(None)
                }
                (true, _, true) | (true, _, _) | (_, _, true) => {
                    bail!("Invalid index comparison: {:?}", expr)
                }
                (false, op, false) => {
                    let still_all_and = all_ands && op == "AND";
                    let converted_left =
                        split_conditions(left.as_ref(), still_all_and, indices, ctx)?;
                    let converted_right =
                        split_conditions(right.as_ref(), still_all_and, indices, ctx)?;
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

const INDEX: &str = "index";
const SOURCE: &str = "source";
const SOURCE_TYPE: &str = "sourcetype";
const HOST: &str = "host";
const HOST_TAG: &str = "hosttag";
const EVENT_TYPE: &str = "eventtype";
const EVENT_TYPE_TAG: &str = "eventtypetag";
const SAVED_SPLUNK: &str = "savedsplunk";
const SPLUNK_SERVER: &str = "splunk_server";

impl PipelineTransformer for SearchCommand {
    fn transform_for_runtime(
        &self,
        state: PipelineTransformState,
    ) -> Result<PipelineTransformState> {
        let mut exprs = vec![];
        break_down_ands(&self.expr, &mut exprs);

        let df = state.df.clone();

        let mut args = vec![];
        let mut kwargs = vec![];

        for expr in exprs {
            match expr.clone() {
                ast::Expr::Binary(ast::Binary {
                    left,
                    symbol,
                    right,
                }) => {
                    match (*left.clone(), symbol.as_str()) {
                        (
                            ast::Expr::Leaf(ast::LeafExpr::Constant(ast::Constant::Field(
                                ast::Field(name),
                            ))),
                            op,
                        ) if matches!(
                            name.as_str(),
                            INDEX
                                | SOURCE
                                | SOURCE_TYPE
                                | HOST
                                | HOST_TAG
                                | EVENT_TYPE
                                | EVENT_TYPE_TAG
                                | SAVED_SPLUNK
                                | SPLUNK_SERVER
                        ) =>
                        {
                            ensure!(
                                matches!(op, "=" | "=="),
                                format!("`{}` must use equality comparison", name)
                            );
                            let name = match name.as_str() {
                                INDEX => INDEX,
                                SOURCE => SOURCE,
                                SOURCE_TYPE => "source_type",
                                HOST => HOST,
                                HOST_TAG => "host_tag",
                                EVENT_TYPE => "event_type",
                                EVENT_TYPE_TAG => "event_type_tag",
                                SAVED_SPLUNK => "saved_splunk",
                                SPLUNK_SERVER => SPLUNK_SERVER,
                                _ => panic!("How on earth did we get here?!?"),
                            };
                            let rhs_value = match *right {
                            ast::Expr::Leaf(ast::LeafExpr::Constant(ast::Constant::Field(ast::Field(
                                                                                             val,
                                                                                         )))) => val,
                            ast::Expr::Leaf(ast::LeafExpr::Constant(ast::Constant::Str(ast::StrValue(val)))) => val,
                            _ => bail!("Right-hand side of comparison to {} must be a string-like value", name),
                        };
                            kwargs.push((name.to_string(), column_like!(py_lit(rhs_value)).into()))
                        }
                        (
                            ast::Expr::Leaf(ast::LeafExpr::Constant(ast::Constant::Field(
                                ast::Field(name),
                            ))),
                            "=" | "==",
                        ) => kwargs.push((
                            name.to_string(),
                            (*right).with_context(&state.ctx).try_into()?,
                        )),
                        _ => args.push(expr.with_context(&state.ctx).try_into()?),
                    }
                }
                _ => args.push(expr.with_context(&state.ctx).try_into()?),
            }
        }

        let df = DataFrame::runtime(df, "search", args, kwargs, &state.ctx);

        Ok(state.with_df(df))
    }

    fn transform_standalone(
        &self,
        state: PipelineTransformState,
    ) -> Result<PipelineTransformState> {
        let mut indices = HashSet::new();
        let condition_expr = split_conditions(&self.expr, true, &mut indices, &state.ctx)?;
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
            state.df.clone().unwrap_or_default()
        };

        df = match condition_expr {
            None => df,
            Some(condition) => {
                let condition: Expr = condition.with_context(&state.ctx).try_into()?;
                df.where_(condition.into_search_expr())
            }
        };

        Ok(state.with_df(df))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pyspark::utils::test::{generates, generates_runtime};
    use rstest::rstest;

    fn check_split_results(
        expr: impl Into<ast::Expr>,
        mut expected_indices: Vec<String>,
        expected_condition: Option<Expr>,
        ctx: &PysparkTranspileContext,
    ) {
        let mut indices = HashSet::new();
        let expr = expr.into();
        let condition: Option<ast::Expr> =
            split_conditions(&expr, true, &mut indices, ctx).unwrap();
        let converted_condition: Option<Expr> =
            condition.map(|e| e.with_context(ctx).try_into().unwrap());
        let mut indices: Vec<_> = indices.into_iter().collect();
        indices.sort();
        expected_indices.sort();
        assert_eq!(indices, expected_indices);
        assert_eq!(converted_condition, expected_condition);
    }

    #[rstest]
    fn test_split_conditions_simple_index(
        #[from(crate::pyspark::base::test::ctx_bare)] ctx: PysparkTranspileContext,
    ) {
        check_split_results(
            ast::Binary::new(ast::Field::from("index"), "=", ast::Field::from("lol")),
            vec!["lol".into()],
            None,
            &ctx,
        );
    }

    #[rstest]
    fn test_split_conditions_no_index(
        #[from(crate::pyspark::base::test::ctx_bare)] ctx: PysparkTranspileContext,
    ) {
        check_split_results(
            ast::Binary::new(ast::Field::from("x"), "=", ast::IntValue::from(2)),
            Vec::<String>::new(),
            Some(column_like!([col("x")] == [lit(2)]).into()),
            &ctx,
        );
    }

    #[rstest]
    fn test_split_conditions_combined_index(
        #[from(crate::pyspark::base::test::ctx_bare)] ctx: PysparkTranspileContext,
    ) {
        check_split_results(
            ast::Binary::new(
                ast::Binary::new(ast::Field::from("x"), "=", ast::IntValue::from(2)),
                "AND",
                ast::Binary::new(ast::Field::from("index"), "=", ast::Field::from("lol")),
            ),
            vec!["lol".into()],
            Some(column_like!([col("x")] == [lit(2)]).into()),
            &ctx,
        );
    }

    #[rstest]
    fn test_multi_index_conditions(
        #[from(crate::pyspark::base::test::ctx_bare)] ctx: PysparkTranspileContext,
    ) {
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
            &ctx,
        );
    }

    #[rstest]
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

    #[rstest]
    fn test_search_9() {
        let query = r#"index="lol" sourcetype="src1" len(x)=3"#;

        generates_runtime(
            query,
            r#"
df_1 = commands.search(None, (functions.eval.len_(F.col("x")) == F.lit(3)), index="lol", source_type="src1")
df_1
            "#,
        )
    }

    #[rstest]
    fn test_search_10() {
        let query = r#"sourcetype=cisco"#;

        generates_runtime(
            query,
            r#"
df_1 = commands.search(None, source_type="cisco")
df_1
            "#,
        )
    }
}
