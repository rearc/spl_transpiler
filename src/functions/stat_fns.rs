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
