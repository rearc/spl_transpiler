/*
https://docs.splunk.com/Documentation/SplunkCloud/9.2.2406/SearchReference/CommonStatsFunctions#Function_list_by_category
Type of function                     	 Supported functions and syntax                                                                                                                                                                                                                                                                                   	 Description

Aggregate functions
avg(<value>)                         	 Returns the average of the values in the field specified.
count(<value>)                       	 Returns the number of occurrences where the field that you specify contains any value (is not empty). You can also count the occurrences of a specific value in the field by using the eval command with the count function. For example: count( eval(field_name="value")).
distinct_count(<value>)              	 Returns the count of distinct values in the field specified.
estdc(<value>)                       	 Returns the estimated count of the distinct values in the field specified.
estdc_error(<value>)                 	 Returns the theoretical error of the estimated count of the distinct values in the field specified. The error represents a ratio of the absolute_value(estimate_distinct_count - real_distinct_count)/real_distinct_count.
exactperc<percentile>(<value>)       	 Returns a percentile value of the numeric field specified. Provides the exact value, but is very resource expensive for high cardinality fields. An alternative is perc.
max(<value>)                         	 Returns the maximum value in the field specified. If the field values are non-numeric, the maximum value is found using lexicographical ordering. This function processes field values as numbers if possible, otherwise processes field values as strings.
mean(<value>)                        	 Returns the arithmetic mean of the values in the field specified.
median(<value>)                      	 Returns the middle-most value of the values in the field specified.
min(<value>)                         	 Returns the minimum value in the field specified. If the field values are non-numeric, the minimum value is found using lexicographical ordering.
mode(<value>)                        	 Returns the most frequent value in the field specified.
percentile<percentile>(<value>)      	 Returns the N-th percentile value of all the values in the numeric field specified. Valid field values are integers from 1 to 99. Additional percentile functions are upperperc<percentile>(<value>) and exactperc<percentile>(<value>).
range(<value>)                       	 If the field values are numeric, returns the difference between the maximum and minimum values in the field specified.
stdev(<value>)                       	 Returns the sample standard deviation of the values in the field specified.
stdevp(<value>)                      	 Returns the population standard deviation of the values in the field specified.
sum(<value>)                         	 Returns the sum of the values in the field specified.
sumsq(<value>)                       	 Returns the sum of the squares of the values in the field specified.
upperperc<percentile>(<value>)       	 Returns an approximate percentile value, based on the requested percentile of the numeric field. When there are more than 1000 values, the upperperc function gives the approximate upper bound for the percentile requested. Otherwise the upperperc function returns the same percentile as the perc function.
var(<value>)                         	 Returns the sample variance of the values in the field specified.
varp(<value>)                        	 Returns the population variance of the values in the field specified.

Event order functions
first(<value>                        	 Returns the first seen value in a field. In general, the first seen value of the field is the most recent instance of this field, relative to the input order of events into the stats command.
last(<value>)                        	 Returns the last seen value in a field. In general, the last seen value of the field is the oldest instance of this field relative to the input order of events into the stats command.

Multivalue stats and chart functions
list(<value>)                        	 Returns a list of up to 100 values in a field as a multivalue entry. The order of the values reflects the order of input events.
values(<value>)                      	 Returns the list of all distinct values in a field as a multivalue entry. The order of the values is lexicographical.

Time functions
earliest(<value>)                    	 Returns the chronologically earliest (oldest) seen occurrence of a value in a field.
earliest_time(<value>)               	 Returns the UNIX time of the earliest (oldest) occurrence of a value of the field. Used in conjunction with the earliest, latest, and latest_time functions to calculate the rate of increase for an accumulating counter.
latest(<value>)                      	 Returns the chronologically latest (most recent) seen occurrence of a value in a field.
latest_time(<value>)                 	 Returns the UNIX time of the latest (most recent) occurrence of a value of the field. Used in conjunction with the earliest, earliest_time, and latest functions to calculate the rate of increase for an accumulating counter.
per_day(<value>)                     	 Returns the values in a field or eval expression for each day.
per_hour(<value>)                    	 Returns the values in a field or eval expression for each hour.
per_minute(<value>)                  	 Returns the values in a field or eval expression for each minute.
per_second(<value>)                  	 Returns the values in a field or eval expression for each second.
rate(<value>)                        	 Returns the per-second rate change of the value of the field. Represents (latest - earliest) / (latest_time - earliest_time) Requires the earliest and latest values of the field to be numerical, and the earliest_time and latest_time values to be different.
rate_avg(<value>)                    	 Returns the average rates for the time series associated with a specified accumulating counter metric.
rate_sum(<value>)                    	 Returns the summed rates for the time series associated with a specified accumulating counter metric.
 */
use crate::functions::*;
use crate::pyspark::ast::ColumnLike;
use crate::pyspark::ast::*;
use crate::pyspark::base::{PysparkTranspileContext, RuntimeSelection, ToSparkExpr};
use crate::spl::ast;
use anyhow::{bail, ensure, Result};
use log::warn;

fn _as_call_and_alias(expr: &ast::Expr) -> Result<(ast::Call, Option<String>)> {
    match expr {
        ast::Expr::Alias(ast::Alias { expr, name }) => {
            let (expr, _) = _as_call_and_alias(expr.as_ref())?;
            Ok((expr, Some(name.clone())))
        }
        ast::Expr::Call(call) => Ok((call.clone(), None)),
        _ => bail!("Expected an alias or a call, got {:?}", expr),
    }
}

pub fn stats_fn(
    expr: ast::Expr,
    df: DataFrame,
    ctx: &PysparkTranspileContext,
) -> Result<(DataFrame, ColumnLike)> {
    let (expr, maybe_alias) = expr.unaliased_with_name();
    let (name, args) = match expr {
        ast::Expr::Call(ast::Call { name, args }) => (name, args),
        _ => bail!("Expected a stats function call, got {:?}", expr),
    };
    let args: Vec<_> = args.into_iter().map(|arg| arg.with_context(ctx)).collect();

    let (df, expr) = match ctx.runtime {
        RuntimeSelection::Disallow => stats_fn_bare(name, args, df),
        RuntimeSelection::Require => stats_fn_runtime(name, args, df),
        RuntimeSelection::Allow => stats_fn_runtime(name.clone(), args.clone(), df.clone())
            .or_else(|_| stats_fn_bare(name, args, df)),
    }?;

    Ok((df, expr.maybe_with_alias(maybe_alias)))
}

fn stats_fn_runtime(
    name: String,
    args: Vec<ContextualizedExpr<ast::Expr>>,
    df: DataFrame,
) -> Result<(DataFrame, ColumnLike)> {
    // match name.as_str() {
    //     name => {
    // warn!(
    //     "Unknown eval function encountered, returning as is: {}",
    //     name
    // );
    let func = format!("functions.stats.{}", name);
    let args: Result<Vec<Expr>> = map_args(args);
    Ok((df, ColumnLike::FunctionCall { func, args: args? }))
    // }
    // }
}

fn stats_fn_bare(
    name: String,
    args: Vec<ContextualizedExpr<ast::Expr>>,
    mut df: DataFrame,
) -> Result<(DataFrame, ColumnLike)> {
    let expr: ColumnLike = match name.as_str() {
        // avg(<value>)                         	 Returns the average of the values in the field specified.
        "avg" => function_transform!(avg [args] (x) { column_like!(avg([x])) }),
        // count(<value>)                       	 Returns the number of occurrences where the field that you specify contains any value (is not empty). You can also count the occurrences of a specific value in the field by using the eval command with the count function. For example: count( eval(field_name="value")).
        "count" => function_transform!(count [args -> mapped_args] () {
            match mapped_args.len() {
                0 => column_like!(count([lit(1)])),
                1 => column_like!(count([mapped_args[0].clone()])),
                _ => bail!("Invalid number of arguments for stats function `{}`", name),
            }
        }),
        // distinct_count(<value>)              	 Returns the count of distinct values in the field specified.
        "distinct_count" => {
            function_transform!(distinct_count [args] (x) { column_like!(count_distinct([x])) })
        }
        "dc" => function_transform!(dc [args] (x) { column_like!(count_distinct([x])) }),
        // estdc(<value>)                       	 Returns the estimated count of the distinct values in the field specified.
        "estdc" => {
            function_transform!(estdc [args] (x) { column_like!(approx_count_distinct([x])) })
        }
        // estdc_error(<value>)                 	 Returns the theoretical error of the estimated count of the distinct values in the field specified. The error represents a ratio of the absolute_value(estimate_distinct_count - real_distinct_count)/real_distinct_count.
        "estdc_error" => bail!(
            "UNIMPLEMENTED: No implementation yet for stats function `{}`",
            name
        ),
        // exactperc<percentile>(<value>)       	 Returns a percentile value of the numeric field specified. Provides the exact value, but is very resource expensive for high cardinality fields. An alternative is perc.
        exactperc if exactperc.starts_with("exactperc") => {
            function_transform!(exactperc [args] (x) {
                let percentile: u32 = exactperc.replace("exactperc", "").parse()?;
                column_like!(percentile([x], [py_lit(percentile as f64 / 100.0)]))
            })
        }
        // max(<value>)                         	 Returns the maximum value in the field specified. If the field values are non-numeric, the maximum value is found using lexicographical ordering. This function processes field values as numbers if possible, otherwise processes field values as strings.
        "max" => function_transform!(max [args] (x) { column_like!(max([x])) }),
        // mean(<value>)                        	 Returns the arithmetic mean of the values in the field specified.
        "mean" => function_transform!(mean [args] (x) { column_like!(avg([x])) }),
        // median(<value>)                      	 Returns the middle-most value of the values in the field specified.
        "median" => {
            function_transform!(median [args] (x) { column_like!(percentile_approx([x], [lit(0.5)])) })
        }
        // min(<value>)                         	 Returns the minimum value in the field specified. If the field values are non-numeric, the minimum value is found using lexicographical ordering.
        "min" => function_transform!(min [args] (x) { column_like!(min([x])) }),
        // mode(<value>)                        	 Returns the most frequent value in the field specified.
        "mode" => function_transform!(mode [args] (x) { column_like!(mode([x])) }),
        // percentile<percentile>(<value>)      	 Returns the N-th percentile value of all the values in the numeric field specified. Valid field values are integers from 1 to 99. Additional percentile functions are upperperc<percentile>(<value>) and exactperc<percentile>(<value>).
        percentile if percentile.starts_with("percentile") => {
            function_transform!(percentile [args] (x) {
                let percentile: u32 = percentile.replace("percentile", "").parse()?;
                column_like!(percentile_approx([x], [py_lit(percentile as f64 / 100.0)]))
            })
        }
        // range(<value>)                       	 If the field values are numeric, returns the difference between the maximum and minimum values in the field specified.
        "range" => {
            function_transform!(range [args] (x) { column_like!(max([[x.clone()] - [min([x])]])) })
        }
        // stdev(<value>)                       	 Returns the sample standard deviation of the values in the field specified.
        "stdev" => function_transform!(stdev [args] (x) { column_like!(stddev_samp([x])) }),
        // stdevp(<value>)                      	 Returns the population standard deviation of the values in the field specified.
        "stdevp" => function_transform!(stdevp [args] (x) { column_like!(stddev_pop([x])) }),
        // sum(<value>)                         	 Returns the sum of the values in the field specified.
        "sum" => function_transform!(sum [args] (x) { column_like!(sum([x])) }),
        // sumsq(<value>)                       	 Returns the sum of the squares of the values in the field specified.
        "sumsq" => function_transform!(sumsq [args] (x) { column_like!(sum([[x.clone()] * [x]])) }),
        // upperperc<percentile>(<value>)       	 Returns an approximate percentile value, based on the requested percentile of the numeric field. When there are more than 1000 values, the upperperc function gives the approximate upper bound for the percentile requested. Otherwise the upperperc function returns the same percentile as the perc function.
        upperperc if upperperc.starts_with("upperperc") => {
            function_transform!(upperperc [args] (x) {
                let percentile: u32 = upperperc.replace("upperperc", "").parse()?;
                column_like!(percentile_approx([x], [py_lit(percentile as f64 / 100.0)]))
            })
        } // var(<value>)                         	 Returns the sample variance of the values in the field specified.
        "var" => function_transform!(var [args] (x) { column_like!(var_samp([x])) }),
        // varp(<value>)                        	 Returns the population variance of the values in the field specified.
        "varp" => function_transform!(varp [args] (x) { column_like!(var_pop([x])) }),
        //
        // Event order functions
        // first(<value>)                        	 Returns the first seen value in a field. In general, the first seen value of the field is the most recent instance of this field, relative to the input order of events into the stats command.
        "first" => {
            function_transform!(first [args] (x) { column_like!(first([x], [py_lit(true)])) })
        }
        // last(<value>)                        	 Returns the last seen value in a field. In general, the last seen value of the field is the oldest instance of this field relative to the input order of events into the stats command.
        "last" => function_transform!(last [args] (x) { column_like!(last([x], [py_lit(true)])) }),
        //
        // Multivalue stats and chart functions
        // list(<value>)                        	 Returns a list of up to 100 values in a field as a multivalue entry. The order of the values reflects the order of input events.
        "list" => function_transform!(list [args] (x) { column_like!(collect_list([x])) }),
        // values(<value>)                      	 Returns the list of all distinct values in a field as a multivalue entry. The order of the values is lexicographical.
        "values" => function_transform!(values [args] (x) { column_like!(collect_set([x])) }),
        //
        // Time functions
        // earliest(<value>)                    	 Returns the chronologically earliest (oldest) seen occurrence of a value in a field.
        "earliest" => function_transform!(earliest [args] (x) {
            df = df.order_by(vec![column_like!([col("_time")].asc())]);
            column_like!(first([x], [py_lit(true)]))
        }),
        // earliest_time(<value>)               	 Returns the UNIX time of the earliest (oldest) occurrence of a value of the field. Used in conjunction with the earliest, latest, and latest_time functions to calculate the rate of increase for an accumulating counter.
        "earliest_time" => {
            function_transform!(earliest_time [args -> _mapped_args] () { column_like!(min([col("_time")])) })
        }
        // latest(<value>)                      	 Returns the chronologically latest (most recent) seen occurrence of a value in a field.
        "latest" => function_transform!(latest [args] (x) {
            df = df.order_by(vec![column_like!([col("_time")].asc())]);
            column_like!(last([x], [py_lit(true)]))
        }),
        // latest_time(<value>)                 	 Returns the UNIX time of the latest (most recent) occurrence of a value of the field. Used in conjunction with the earliest, earliest_time, and latest functions to calculate the rate of increase for an accumulating counter.
        "latest_time" => {
            function_transform!(latest_time [args -> _mapped_args] () { column_like!(max([col("_time")])) })
        }
        // per_day(<value>)                     	 Returns the values in a field or eval expression for each day.
        "per_day" => bail!(
            "UNIMPLEMENTED: No implementation yet for stats function `{}`",
            name
        ),
        // per_hour(<value>)                    	 Returns the values in a field or eval expression for each hour.
        "per_hour" => bail!(
            "UNIMPLEMENTED: No implementation yet for stats function `{}`",
            name
        ),
        // per_minute(<value>)                  	 Returns the values in a field or eval expression for each minute.
        "per_minute" => bail!(
            "UNIMPLEMENTED: No implementation yet for stats function `{}`",
            name
        ),
        // per_second(<value>)                  	 Returns the values in a field or eval expression for each second.
        "per_second" => bail!(
            "UNIMPLEMENTED: No implementation yet for stats function `{}`",
            name
        ),
        // rate(<value>)                        	 Returns the per-second rate change of the value of the field. Represents (latest - earliest) / (latest_time - earliest_time) Requires the earliest and latest values of the field to be numerical, and the earliest_time and latest_time values to be different.
        "rate" => function_transform!(rate [args] (x) {
            df = df.order_by(vec![column_like!([col("_time")].asc())]);
            let latest_value = column_like!(last([x.clone()]));
            let earliest_value = column_like!(first([x]));
            let latest_time = column_like!(max([col("_time")]));
            let earliest_time = column_like!(min([col("_time")]));
            column_like!([[latest_value] - [earliest_value]] / [[latest_time] - [earliest_time]])
        }),
        // rate_avg(<value>)                    	 Returns the average rates for the time series associated with a specified accumulating counter metric.
        "rate_avg" => bail!(
            "UNIMPLEMENTED: No implementation yet for stats function `{}`",
            name
        ),
        // rate_sum(<value>)                    	 Returns the summed rates for the time series associated with a specified accumulating counter metric.
        "rate_sum" => bail!(
            "UNIMPLEMENTED: No implementation yet for stats function `{}`",
            name
        ),

        // Fallback
        name => {
            warn!(
                "Unknown eval function encountered, returning as is: {}",
                name
            );
            let args: Vec<Expr> = map_args(args)?;
            anyhow::Ok(ColumnLike::Aliased {
                name: name.into(),
                col: Box::new(
                    ColumnLike::FunctionCall {
                        func: name.to_string(),
                        args,
                    }
                    .into(),
                ),
            })
        }
    }?;

    Ok((df, expr))
}
