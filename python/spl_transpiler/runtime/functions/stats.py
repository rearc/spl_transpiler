from typing import Any, Literal

from pydantic import BaseModel
from pyspark.sql import DataFrame, functions as F, Window

from spl_transpiler.runtime.base import Expr, enforce_types


class StatsFunction(BaseModel):
    name: Literal[None] = None

    def to_pyspark_expr(self, df) -> tuple[DataFrame, Any]:
        return df, self.to_pyspark_expr_simple()

    def to_pyspark_expr_simple(self) -> Any:
        raise NotImplementedError()


#         // avg(<value>)                         	 Returns the average of the values in the field specified.
#         "avg" => function_transform!(avg [args] (x) { column_like!(avg([x])) }),
class AvgStatsFn(StatsFunction):
    name: Literal["avg"] = "avg"
    value: Expr

    def to_pyspark_expr_simple(self):
        return F.avg(self.value.to_pyspark_expr())


@enforce_types
def avg(value: Expr) -> AvgStatsFn:
    return AvgStatsFn(value=value)


#         // count(<value>)                       	 Returns the number of occurrences where the field that you specify contains any value (is not empty). You can also count the occurrences of a specific value in the field by using the eval command with the count function. For example: count( eval(field_name="value")).
#         "count" => function_transform!(count [args -> mapped_args] () {
#             match mapped_args.len() {
#                 0 => column_like!(count([lit(1)])),
#                 1 => column_like!(count([mapped_args[0].clone()])),
#                 _ => bail!("Invalid number of arguments for stats function `{}`", name),
#             }
#         }),
class CountStatsFn(StatsFunction):
    name: Literal["count"] = "count"

    def to_pyspark_expr_simple(self):
        return F.count(F.lit(1))


@enforce_types
def count() -> CountStatsFn:
    return CountStatsFn()


#         // distinct_count(<value>)              	 Returns the count of distinct values in the field specified.
#         "distinct_count" => {
#             function_transform!(distinct_count [args] (x) { column_like!(count_distinct([x])) })
#         }
#         "dc" => function_transform!(dc [args] (x) { column_like!(count_distinct([x])) }),


class DistinctCountStatsFn(StatsFunction):
    name: Literal["distinct_count"] = "distinct_count"
    x: Expr

    def to_pyspark_expr_simple(self):
        return F.approx_count_distinct(self.x.to_pyspark_expr())


@enforce_types
def distinct_count(x: Expr) -> DistinctCountStatsFn:
    return DistinctCountStatsFn(x=x)


dc = distinct_count


#         // estdc(<value>)                       	 Returns the estimated count of the distinct values in the field specified.
#         "estdc" => {
#             function_transform!(estdc [args] (x) { column_like!(approx_count_distinct([x])) })
#         }


class EstdcStatsFn(StatsFunction):
    name: Literal["estdc"] = "estdc"
    x: Expr

    def to_pyspark_expr_simple(self):
        return F.approx_count_distinct(self.x.to_pyspark_expr())


def estdc(x: Expr) -> EstdcStatsFn:
    return EstdcStatsFn(x=x)


#         // estdc_error(<value>)                 	 Returns the theoretical error of the estimated count of the distinct values in the field specified. The error represents a ratio of the absolute_value(estimate_distinct_count - real_distinct_count)/real_distinct_count.
#         "estdc_error" => bail!(
#             "UNIMPLEMENTED: No implementation yet for stats function `{}`",
#             name
#         ),
class EstdcErrorStatsFn(StatsFunction):
    name: Literal["estdc_error"] = "estdc_error"
    value: Expr

    def to_pyspark_expr_simple(self):
        # Since this function is not implemented in the original code,
        # we'll raise a NotImplementedError
        raise NotImplementedError(
            "No implementation yet for stats function 'estdc_error'"
        )


@enforce_types
def estdc_error(value: Expr) -> EstdcErrorStatsFn:
    return EstdcErrorStatsFn(value=value)


#         // exactperc<percentile>(<value>)       	 Returns a percentile value of the numeric field specified. Provides the exact value, but is very resource expensive for high cardinality fields. An alternative is perc.
#         exactperc if exactperc.starts_with("exactperc") => {
#             function_transform!(exactperc [args] (x) {
#                 let percentile: u32 = exactperc.replace("exactperc", "").parse()?;
#                 column_like!(percentile([x], [py_lit(percentile as f64 / 100.0)]))
#             })
#         }
class ExactPercStatsFn(StatsFunction):
    name: Literal["exactperc"] = "exactperc"
    value: Expr
    percentile: int | float

    def to_pyspark_expr_simple(self):
        return F.percentile(self.value.to_pyspark_expr(), self.percentile / 100.0)


@enforce_types
def exactperc(value: Expr, percentile: int) -> ExactPercStatsFn:
    return ExactPercStatsFn(value=value, percentile=percentile)


#         // max(<value>)                         	 Returns the maximum value in the field specified. If the field values are non-numeric, the maximum value is found using lexicographical ordering. This function processes field values as numbers if possible, otherwise processes field values as strings.
#         "max" => function_transform!(max [args] (x) { column_like!(max([x])) }),
class MaxStatsFn(StatsFunction):
    name: Literal["max"] = "max"
    value: Expr

    def to_pyspark_expr_simple(self):
        return F.max(self.value.to_pyspark_expr())


@enforce_types
def max(value: Expr) -> MaxStatsFn:
    return MaxStatsFn(value=value)


#         // mean(<value>)                        	 Returns the arithmetic mean of the values in the field specified.
#         "mean" => function_transform!(mean [args] (x) { column_like!(avg([x])) }),


class MeanStatsFn(StatsFunction):
    name: Literal["mean"] = "mean"
    value: Expr

    def to_pyspark_expr_simple(self):
        return F.avg(self.value.to_pyspark_expr())


@enforce_types
def mean(value: Expr) -> MeanStatsFn:
    return MeanStatsFn(value=value)


#         // median(<value>)                      	 Returns the middle-most value of the values in the field specified.
#         "median" => {
#             function_transform!(median [args] (x) { column_like!(percentile_approx([x], [lit(0.5)])) })
#         }


class MedianStatsFn(StatsFunction):
    name: Literal["median"] = "median"
    value: Expr

    def to_pyspark_expr_simple(self):
        return F.percentile_approx(self.value.to_pyspark_expr(), 0.5)


@enforce_types
def median(value: Expr) -> MedianStatsFn:
    return MedianStatsFn(value=value)


#         // min(<value>)                         	 Returns the minimum value in the field specified. If the field values are non-numeric, the minimum value is found using lexicographical ordering.
#         "min" => function_transform!(min [args] (x) { column_like!(min([x])) }),


class MinStatsFn(StatsFunction):
    name: Literal["min"] = "min"
    value: Expr

    def to_pyspark_expr_simple(self):
        return F.min(self.value.to_pyspark_expr())


@enforce_types
def min(value: Expr) -> MinStatsFn:
    return MinStatsFn(value=value)


#         // mode(<value>)                        	 Returns the most frequent value in the field specified.
#         "mode" => function_transform!(mode [args] (x) { column_like!(mode([x])) }),


class ModeStatsFn(StatsFunction):
    name: Literal["mode"] = "mode"
    value: Expr

    def to_pyspark_expr_simple(self):
        return F.mode(self.value.to_pyspark_expr())


@enforce_types
def mode(value: Expr) -> ModeStatsFn:
    return ModeStatsFn(value=value)


#         // percentile<percentile>(<value>)      	 Returns the N-th percentile value of all the values in the numeric field specified. Valid field values are integers from 1 to 99. Additional percentile functions are upperperc<percentile>(<value>) and exactperc<percentile>(<value>).
#         percentile if percentile.starts_with("percentile") => {
#             function_transform!(percentile [args] (x) {
#                 let percentile: u32 = percentile.replace("percentile", "").parse()?;
#                 column_like!(percentile_approx([x], [py_lit(percentile as f64 / 100.0)]))
#             })
#         }


class PercentileStatsFn(StatsFunction):
    name: Literal["percentile"] = "percentile"
    value: Expr
    percentile: int | float

    def to_pyspark_expr_simple(self):
        return F.percentile_approx(
            self.value.to_pyspark_expr(), self.percentile / 100.0
        )


@enforce_types
def percentile(value: Expr, percentile: int) -> PercentileStatsFn:
    return PercentileStatsFn(value=value, percentile=percentile)


#         // range(<value>)                       	 If the field values are numeric, returns the difference between the maximum and minimum values in the field specified.
#         "range" => {
#             function_transform!(range [args] (x) { column_like!(max([[x.clone()] - [min([x])]])) })
#         }


class RangeStatsFn(StatsFunction):
    name: Literal["range"] = "range"
    value: Expr

    def to_pyspark_expr_simple(self):
        return F.max(self.value.to_pyspark_expr()) - F.min(self.value.to_pyspark_expr())


@enforce_types
def range(value: Expr) -> RangeStatsFn:
    return RangeStatsFn(value=value)


#         // stdev(<value>)                       	 Returns the sample standard deviation of the values in the field specified.
#         "stdev" => function_transform!(stdev [args] (x) { column_like!(stddev_samp([x])) }),


class StdevStatsFn(StatsFunction):
    name: Literal["stdev"] = "stdev"
    value: Expr

    def to_pyspark_expr_simple(self):
        return F.stddev_samp(self.value.to_pyspark_expr())


@enforce_types
def stdev(value: Expr) -> StdevStatsFn:
    return StdevStatsFn(value=value)


#         // stdevp(<value>)                      	 Returns the population standard deviation of the values in the field specified.
#         "stdevp" => function_transform!(stdevp [args] (x) { column_like!(stddev_pop([x])) }),


class StddevStatsFn(StatsFunction):
    name: Literal["stddev"] = "stddev"
    value: Expr

    def to_pyspark_expr_simple(self):
        return F.stddev_samp(self.value.to_pyspark_expr())


@enforce_types
def stddev(value: Expr) -> StddevStatsFn:
    return StddevStatsFn(value=value)


#         // sum(<value>)                         	 Returns the sum of the values in the field specified.
#         "sum" => function_transform!(sum [args] (x) { column_like!(sum([x])) }),


class SumStatsFn(StatsFunction):
    name: Literal["sum"] = "sum"
    value: Expr

    def to_pyspark_expr_simple(self):
        return F.sum(self.value.to_pyspark_expr())


@enforce_types
def sum_(value: Expr) -> SumStatsFn:
    return SumStatsFn(value=value)


#         // sumsq(<value>)                       	 Returns the sum of the squares of the values in the field specified.
#         "sumsq" => function_transform!(sumsq [args] (x) { column_like!(sum([[x.clone()] * [x]])) }),


class SumSqStatsFn(StatsFunction):
    name: Literal["sumsq"] = "sumsq"
    value: Expr

    def to_pyspark_expr_simple(self):
        return F.sum(F.pow(self.value.to_pyspark_expr(), 2))


@enforce_types
def sumsq(value: Expr) -> SumSqStatsFn:
    return SumSqStatsFn(value=value)


#         // upperperc<percentile>(<value>)       	 Returns an approximate percentile value, based on the requested percentile of the numeric field. When there are more than 1000 values, the upperperc function gives the approximate upper bound for the percentile requested. Otherwise the upperperc function returns the same percentile as the perc function.
#         upperperc if upperperc.starts_with("upperperc") => {
#             function_transform!(upperperc [args] (x) {
#                 let percentile: u32 = upperperc.replace("upperperc", "").parse()?;
#                 column_like!(percentile_approx([x], [py_lit(percentile as f64 / 100.0)]))
#             })


class UpperPercStatsFn(StatsFunction):
    name: Literal["upperperc"] = "upperperc"
    value: Expr
    percentile: int | float

    def to_pyspark_expr_simple(self):
        return F.percentile_approx(
            self.value.to_pyspark_expr(), self.percentile / 100.0
        )


@enforce_types
def upperperc(value: Expr, percentile: int) -> UpperPercStatsFn:
    return UpperPercStatsFn(value=value, percentile=percentile)


#         } // var(<value>)                         	 Returns the sample variance of the values in the field specified.
#         "var" => function_transform!(var [args] (x) { column_like!(var_samp([x])) }),


class VarianceStatsFn(StatsFunction):
    name: Literal["var"] = "var"
    value: Expr

    def to_pyspark_expr_simple(self):
        return F.var_samp(self.value.to_pyspark_expr())


@enforce_types
def var(value: Expr) -> VarianceStatsFn:
    return VarianceStatsFn(value=value)


#         // varp(<value>)                        	 Returns the population variance of the values in the field specified.
#         "varp" => function_transform!(varp [args] (x) { column_like!(var_pop([x])) }),


class VarpStatsFn(StatsFunction):
    name: Literal["varp"] = "varp"
    value: Expr

    def to_pyspark_expr_simple(self):
        return F.var_pop(self.value.to_pyspark_expr())


@enforce_types
def varp(value: Expr) -> VarpStatsFn:
    return VarpStatsFn(value=value)


#         //
#         // Event order functions
#         // first(<value>)                        	 Returns the first seen value in a field. In general, the first seen value of the field is the most recent instance of this field, relative to the input order of events into the stats command.
#         "first" => {
#             function_transform!(first [args] (x) { column_like!(first([x], [py_lit(true)])) })
#         }


class FirstStatsFn(StatsFunction):
    name: Literal["first"] = "first"
    value: Expr

    def to_pyspark_expr_simple(self):
        return F.first(self.value.to_pyspark_expr(), ignorenulls=True)


@enforce_types
def first(value: Expr) -> FirstStatsFn:
    return FirstStatsFn(value=value)


#         // last(<value>)                        	 Returns the last seen value in a field. In general, the last seen value of the field is the oldest instance of this field relative to the input order of events into the stats command.
#         "last" => function_transform!(last [args] (x) { column_like!(last([x], [py_lit(true)])) }),


class LastStatsFn(StatsFunction):
    name: Literal["last"] = "last"
    value: Expr

    def to_pyspark_expr_simple(self):
        return F.last(self.value.to_pyspark_expr(), ignorenulls=True)


@enforce_types
def last(value: Expr) -> LastStatsFn:
    return LastStatsFn(value=value)


#         //
#         // Multivalue stats and chart functions
#         // list(<value>)                        	 Returns a list of up to 100 values in a field as a multivalue entry. The order of the values reflects the order of input events.
#         "list" => function_transform!(list [args] (x) { column_like!(collect_list([x])) }),


class ListStatsFn(StatsFunction):
    name: Literal["list"] = "list"
    value: Expr

    def to_pyspark_expr_simple(self):
        return F.collect_list(self.value.to_pyspark_expr())


@enforce_types
def list_(value: Expr) -> ListStatsFn:
    return ListStatsFn(value=value)


#         // values(<value>)                      	 Returns the list of all distinct values in a field as a multivalue entry. The order of the values is lexicographical.
#         "values" => function_transform!(values [args] (x) { column_like!(collect_set([x])) }),


class ValuesStatsFn(StatsFunction):
    name: Literal["values"] = "values"
    value: Expr

    def to_pyspark_expr_simple(self):
        return F.collect_set(self.value.to_pyspark_expr())


@enforce_types
def values(value: Expr) -> ValuesStatsFn:
    return ValuesStatsFn(value=value)


#         //
#         // Time functions
#         // earliest(<value>)                    	 Returns the chronologically earliest (oldest) seen occurrence of a value in a field.
#         "earliest" => function_transform!(earliest [args] (x) {
#             df = df.order_by(vec![column_like!([col("_time")].asc())]);
#             column_like!(first([x], [py_lit(true)]))
#         }),


class EarliestStatsFn(StatsFunction):
    name: Literal["earliest"] = "earliest"
    value: Expr

    def to_pyspark_expr(self, df: DataFrame):
        df = df.orderBy(F.col("_time").asc())
        return (df, F.first(self.value.to_pyspark_expr(), ignorenulls=True))


@enforce_types
def earliest(value: Expr) -> EarliestStatsFn:
    return EarliestStatsFn(value=value)


#         // earliest_time(<value>)               	 Returns the UNIX time of the earliest (oldest) occurrence of a value of the field. Used in conjunction with the earliest, latest, and latest_time functions to calculate the rate of increase for an accumulating counter.
#         "earliest_time" => {
#             function_transform!(earliest_time [args -> _mapped_args] () { column_like!(min([col("_time")])) })
#         }


class EarliestTimeStatsFn(StatsFunction):
    name: Literal["earliest_time"] = "earliest_time"

    def to_pyspark_expr_simple(self):
        return F.min(F.col("_time"))


@enforce_types
def earliest_time() -> EarliestTimeStatsFn:
    return EarliestTimeStatsFn()


#         // latest(<value>)                      	 Returns the chronologically latest (most recent) seen occurrence of a value in a field.
#         "latest" => function_transform!(latest [args] (x) {
#             df = df.order_by(vec![column_like!([col("_time")].asc())]);
#             column_like!(last([x], [py_lit(true)]))
#         }),


class LatestStatsFn(StatsFunction):
    name: Literal["latest"] = "latest"
    value: Expr

    def to_pyspark_expr(self, df: DataFrame):
        df = df.orderBy(F.col("_time").asc())
        return (df, F.last(self.value.to_pyspark_expr(), ignorenulls=True))


@enforce_types
def latest(value: Expr) -> LatestStatsFn:
    return LatestStatsFn(value=value)


#         // latest_time(<value>)                 	 Returns the UNIX time of the latest (most recent) occurrence of a value of the field. Used in conjunction with the earliest, earliest_time, and latest functions to calculate the rate of increase for an accumulating counter.
#         "latest_time" => {
#             function_transform!(latest_time [args -> _mapped_args] () { column_like!(max([col("_time")])) })
#         }


class LatestTimeStatsFn(StatsFunction):
    name: Literal["latest_time"] = "latest_time"

    def to_pyspark_expr_simple(self):
        return F.max(F.col("_time"))


@enforce_types
def latest_time() -> LatestTimeStatsFn:
    return LatestTimeStatsFn()


#         // per_day(<value>)                     	 Returns the values in a field or eval expression for each day.
#         "per_day" => bail!(
#             "UNIMPLEMENTED: No implementation yet for stats function `{}`",
#             name
#         ),


class PerDayStatsFn(StatsFunction):
    name: Literal["per_day"] = "per_day"
    value: Expr

    def to_pyspark_expr_simple(self):
        raise NotImplementedError(
            f"No implementation yet for stats function `{self.name}`"
        )


@enforce_types
def per_day(value: Expr) -> PerDayStatsFn:
    return PerDayStatsFn(value=value)


#         // per_hour(<value>)                    	 Returns the values in a field or eval expression for each hour.
#         "per_hour" => bail!(
#             "UNIMPLEMENTED: No implementation yet for stats function `{}`",
#             name
#         ),


class PerHourStatsFn(StatsFunction):
    name: Literal["per_hour"] = "per_hour"
    value: Expr

    def to_pyspark_expr_simple(self):
        raise NotImplementedError(
            f"No implementation yet for stats function `{self.name}`"
        )


@enforce_types
def per_hour(value: Expr) -> PerHourStatsFn:
    return PerHourStatsFn(value=value)


#         // per_minute(<value>)                  	 Returns the values in a field or eval expression for each minute.
#         "per_minute" => bail!(
#             "UNIMPLEMENTED: No implementation yet for stats function `{}`",
#             name
#         ),


class PerMinuteStatsFn(StatsFunction):
    name: Literal["per_minute"] = "per_minute"
    value: Expr

    def to_pyspark_expr_simple(self):
        raise NotImplementedError(
            f"No implementation yet for stats function `{self.name}`"
        )


@enforce_types
def per_minute(value: Expr) -> PerMinuteStatsFn:
    return PerMinuteStatsFn(value=value)


#         // per_second(<value>)                  	 Returns the values in a field or eval expression for each second.
#         "per_second" => bail!(
#             "UNIMPLEMENTED: No implementation yet for stats function `{}`",
#             name
#         ),


class PerSecondStatsFn(StatsFunction):
    name: Literal["per_second"] = "per_second"
    value: Expr

    def to_pyspark_expr_simple(self):
        raise NotImplementedError(
            f"No implementation yet for stats function `{self.name}`"
        )


@enforce_types
def per_second(value: Expr) -> PerSecondStatsFn:
    return PerSecondStatsFn(value=value)


#         // rate(<value>)                        	 Returns the per-second rate change of the value of the field. Represents (latest - earliest) / (latest_time - earliest_time) Requires the earliest and latest values of the field to be numerical, and the earliest_time and latest_time values to be different.
#         "rate" => function_transform!(rate [args] (x) {
#             df = df.order_by(vec![column_like!([col("_time")].asc())]);
#             let latest_value = column_like!(last([x.clone()]));
#             let earliest_value = column_like!(first([x]));
#             let latest_time = column_like!(max([col("_time")]));
#             let earliest_time = column_like!(min([col("_time")]));
#             column_like!([[latest_value] - [earliest_value]] / [[latest_time] - [earliest_time]])
#         }),


class RateStatsFn(StatsFunction):
    name: Literal["rate"] = "rate"
    value: Expr

    def to_pyspark_expr(self, df: DataFrame):
        df = df.orderBy(F.col("_time").asc())
        latest_value = F.last(self.value.to_pyspark_expr())
        earliest_value = F.first(self.value.to_pyspark_expr())
        latest_time = F.max(F.col("_time"))
        earliest_time = F.min(F.col("_time"))
        return (df, (latest_value - earliest_value) / (latest_time - earliest_time))


@enforce_types
def rate(value: Expr) -> RateStatsFn:
    return RateStatsFn(value=value)


#         // rate_avg(<value>)                    	 Returns the average rates for the time series associated with a specified accumulating counter metric.
#         "rate_avg" => bail!(
#             "UNIMPLEMENTED: No implementation yet for stats function `{}`",
#             name
#         ),


class RateAvgStatsFn(StatsFunction):
    name: Literal["rate_avg"] = "rate_avg"
    value: Expr

    def to_pyspark_expr(self, df: DataFrame):
        """
        Calculates the average rate of change for a time series in a PySpark DataFrame.

        This function computes the average rate of change between consecutive rows
        in a time-ordered DataFrame. It uses a window function to compare each row
        with its previous row, calculating the rate of change when the time differs.

        Parameters:
        df (DataFrame): The input PySpark DataFrame containing the time series data.
                        It must have a '_time' column and a column corresponding to
                        the value being analyzed.

        Returns:
        tuple: A tuple containing two elements:
               - The input DataFrame sorted by '_time' in ascending order.
               - A PySpark Column expression representing the average rate of change,
                 where rate is calculated as (current_value - previous_value) / (current_time - previous_time)
                 for each pair of consecutive rows with different timestamps.

        Note:
        The function assumes that 'self.value' is an Expr object that can be
        converted to a PySpark expression using the to_pyspark_expr() method.
        """
        df = df.orderBy(F.col("_time").asc())
        value_expr = self.value.to_pyspark_expr()

        window_spec = Window.orderBy("_time").rowsBetween(
            Window.unboundedPreceding, Window.currentRow
        )

        prev_value = F.lag(value_expr).over(window_spec)
        prev_time = F.lag(F.col("_time")).over(window_spec)

        rate = (value_expr - prev_value) / (F.col("_time") - prev_time)

        return (df, F.avg(F.when(F.col("_time") != prev_time, rate).otherwise(None)))


@enforce_types
def rate_avg(value: Expr) -> RateAvgStatsFn:
    return RateAvgStatsFn(value=value)


#         // rate_sum(<value>)                    	 Returns the summed rates for the time series associated with a specified accumulating counter metric.
#         "rate_sum" => bail!(
#             "UNIMPLEMENTED: No implementation yet for stats function `{}`",
#             name
#         ),


class RateSumStatsFn(StatsFunction):
    name: Literal["rate_sum"] = "rate_sum"
    value: Expr

    def to_pyspark_expr(self, df: DataFrame):
        """
        This function calculates the summed rates for the time series associated with a specified accumulating counter metric.
        It orders the DataFrame by time, calculates the rate of change between consecutive rows,
        and returns the sum of these rates.

        Parameters:
        df (DataFrame): The input DataFrame containing the time series data.
                        It should have a '_time' column and a column corresponding to the value being analyzed.

        Returns:
        tuple: A tuple containing:
               - The input DataFrame ordered by '_time'
               - A PySpark Column expression representing the sum of rates,
                 where rate is calculated as (current_value - previous_value) / (current_time - previous_time)
                 for each pair of consecutive rows.

        Note:
        The function assumes that the 'value' attribute of the class is an Expr object
        that can be converted to a PySpark expression using the to_pyspark_expr() method.
        """
        df = df.orderBy(F.col("_time").asc())
        value_expr = self.value.to_pyspark_expr()

        window_spec = Window.orderBy("_time").rowsBetween(
            Window.unboundedPreceding, Window.currentRow
        )

        prev_value = F.lag(value_expr).over(window_spec)
        prev_time = F.lag(F.col("_time")).over(window_spec)

        rate = (value_expr - prev_value) / (F.col("_time") - prev_time)

        return (df, F.sum(F.when(F.col("_time") != prev_time, rate).otherwise(0)))


@enforce_types
def rate_sum(value: Expr) -> RateSumStatsFn:
    return RateSumStatsFn(value=value)
