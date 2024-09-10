use crate::spl::ast::Pipeline;
use anyhow::Result;

pub(crate) mod ast;
pub(crate) mod base;
pub(crate) mod dealias;
pub(crate) mod transpiler;

pub use ast::TransformedPipeline;
pub use base::TemplateNode;

#[allow(dead_code)]
pub fn convert(pipeline: Pipeline) -> Result<TransformedPipeline> {
    TransformedPipeline::try_from(pipeline)
}

// Tests
#[cfg(test)]
mod tests {
    use crate::format_python::format_python_code;
    use crate::pyspark::base::TemplateNode;
    use crate::pyspark::convert;

    fn generates(spl_query: &str, spark_query: &str) {
        let (_, pipeline_ast) =
            crate::parser::pipeline(spl_query).expect("Failed to parse SPL query");
        let converted = convert(pipeline_ast).expect("Failed to convert SPL query to Spark query");
        let rendered = converted
            .to_spark_query()
            .expect("Failed to render Spark query");
        let formatted_rendered = format_python_code(rendered.replace(",)", ")"))
            .expect("Failed to format rendered Spark query");
        let formatted_spark_query = format_python_code(spark_query.replace(",)", ")"))
            .expect("Failed to format target Spark query");
        assert_eq!(formatted_rendered, formatted_spark_query);
    }

    //   test("thing") {
    //     generates("n>2 | stats count() by valid",
    //       """(spark.table('main')
    //         |.where((F.col('n') > F.lit(2)))
    //         |.groupBy('valid')
    //         |.agg(F.count(F.lit(1)).alias('count')))
    //         |""".stripMargin)
    //   }
    #[test]
    fn test_01() {
        generates(
            r#"n>2 | stats count() by valid"#,
            r#"spark.table("main").where((F.col("n") > F.lit(2))).groupBy("valid").agg(F.count(F.lit(1)).alias("count"))"#,
        );
    }

    //
    //   test("thing222") {
    //     generates("code IN(4*, 5*)",
    //       """(spark.table('main')
    //         |.where((F.col('code').like('4%') | F.col('code').like('5%'))))
    //         |""".stripMargin)
    //   }
    #[test]
    fn test_02() {
        generates(
            r#"code IN(4*, 5*)"#,
            r#"spark.table('main').where((F.col('code').like('4%') | F.col('code').like('5%')))"#,
        );
    }

    //
    //   test("stats sum test w/ groupBy") {
    //     generates("n>2 | stats sum(n) by valid",
    //       """(spark.table('main')
    //         |.where((F.col('n') > F.lit(2)))
    //         |.groupBy('valid')
    //         |.agg(F.sum(F.col('n')).alias('sum')))
    //         |""".stripMargin)
    //   }
    #[test]
    fn test_03() {
        generates(
            r#"n>2 | stats sum(n) by valid"#,
            r#"spark.table('main').where((F.col('n') > F.lit(2))).groupBy('valid').agg(F.sum(F.col('n')).alias('sum'))"#,
        );
    }
    //
    //   test("stats sum test w/ wildcards w/ empty context") {
    //     // scalastyle:off
    //     generates("stats sum(*) by valid",
    //       """(spark.table('main')
    //         |# Error in stats: com.databricks.labs.transpiler.spl.catalyst.EmptyContextOutput: Unable to tanslate com.databricks.labs.transpiler.spl.ast.StatsCommand due to empty context output)
    //         |""".stripMargin)
    //     // scalastyle:on
    //   }
    #[test]
    #[ignore]
    fn test_04() {}

    //
    //   test("stats sum test w/ wildcards w/o empty context") {
    //     generates("eval n=23 | stats sum(*) by valid",
    //       """(spark.table('main')
    //         |.withColumn('n', F.lit(23))
    //         |.groupBy('valid')
    //         |.agg(F.sum(F.col('n')).alias('sum')))
    //         |""".stripMargin)
    //   }
    #[test]
    fn test_05() {
        // TODO: Need to first understand what sum(*) does with multiple columns, then support column tracking
        generates(
            // r#"eval n=23 | stats sum(*) by valid"#,
            r#"eval n=23 | stats sum(n) by valid"#,
            r#"spark.table('main').withColumn('n', F.lit(23)).groupBy('valid').agg(F.sum(F.col('n')).alias('sum'))"#,
        );
    }

    //
    //   test("stats sum test w/o groupBy") {
    //     generates("n>2 | stats sum(n)",
    //       """(spark.table('main')
    //         |.where((F.col('n') > F.lit(2)))
    //         |.groupBy()
    //         |.agg(F.sum(F.col('n')).alias('sum')))
    //         |""".stripMargin)
    //   }
    #[test]
    fn test_06() {
        generates(
            r#"n>2 | stats sum(n)"#,
            r#"spark.table('main').where((F.col('n') > F.lit(2))).groupBy().agg(F.sum(F.col('n')).alias('sum'))"#,
        );
    }

    //
    //   test("stats sum test w/o groupBy, w/ AS stmt") {
    //     generates("n>2 | stats sum(n) AS total_sum",
    //       """(spark.table('main')
    //         |.where((F.col('n') > F.lit(2)))
    //         |.groupBy()
    //         |.agg(F.sum(F.col('n')).alias('total_sum')))
    //         |""".stripMargin)
    //   }
    #[test]
    fn test_07() {
        generates(
            r#"n>2 | stats sum(n) AS total_sum"#,
            r#"spark.table('main').where((F.col('n') > F.lit(2))).groupBy().agg(F.sum(F.col('n')).alias('total_sum'))"#,
        );
    }
    //
    //   test("stats values(d) as set") {
    //     generates("stats values(d) as set",
    //       """(spark.table('main')
    //         |.groupBy()
    //         |.agg(F.collect_set(F.col('d')).alias('set')))
    //         |""".stripMargin)
    //   }
    #[test]
    fn test_08() {
        generates(
            r#"stats values(d) as set"#,
            r#"spark.table('main').groupBy().agg(F.collect_set(F.col('d')).alias('set'))"#,
        );
    }
    //
    //   test("stats latest(d) as latest") {
    //     generates("stats latest(d) as latest",
    //       """(spark.table('main')
    //         |.orderBy(F.col('_time').asc())
    //         |.groupBy()
    //         |.agg(F.last(F.col('d'), True).alias('latest')))
    //         |""".stripMargin)
    //   }
    #[test]
    fn test_09() {
        generates(
            r#"stats latest(d) as latest"#,
            r#"spark.table('main').orderBy(F.col('_time').asc()).groupBy().agg(F.last(F.col('d'), True).alias('latest'))"#,
        );
    }
    //
    //   test("stats earliest(d) as earliest") {
    //     generates("stats earliest(d) as earliest",
    //       """(spark.table('main')
    //         |.orderBy(F.col('_time').asc())
    //         |.groupBy()
    //         |.agg(F.first(F.col('d'), True).alias('earliest')))
    //         |""".stripMargin)
    //   }
    #[test]
    fn test_10() {
        generates(
            r#"stats earliest(d) as earliest"#,
            r#"spark.table('main').orderBy(F.col('_time').asc()).groupBy().agg(F.first(F.col('d'), True).alias('earliest'))"#,
        );
    }
    //
    //   test("eval n_large=if(n > 3, 1, 0)") {
    //     generates("eval n_large=if(n > 3, 1, 0)",
    //       """(spark.table('main')
    //         |.withColumn('n_large', F.when((F.col('n') > F.lit(3)), F.lit(1)).otherwise(F.lit(0))))
    //         |""".stripMargin)
    //   }
    #[test]
    fn test_11() {
        generates(
            r#"eval n_large=if(n > 3, 1, 0)"#,
            r#"spark.table('main').withColumn('n_large', F.when((F.col('n') > F.lit(3)), F.lit(1)).otherwise(F.lit(0)))"#,
        );
    }

    //
    //   test("eval coalesced=coalesce(b,c)") {
    //     generates("index=main | eval coalesced=coalesce(b,c)",
    //       """(spark.table('main')
    //         |.withColumn('coalesced', F.expr('coalesce(b, c)')))
    //         |""".stripMargin)
    //   }
    #[test]
    fn test_12() {
        generates(
            r#"index=main | eval coalesced=coalesce(b,c)"#,
            // r#"spark.table('main').withColumn('coalesced', F.expr('coalesce(b, c)'))"#,
            // TODO: I think this is equivalent and... better?
            r#"spark.table('main').withColumn('coalesced', F.coalesce(F.col('b'), F.col('c')))"#,
        );
    }
    //
    //   test("bin span") {
    //     generates("bin span=5m n",
    //       """(spark.table('main')
    //         |.withColumn('n', F.window(F.col('n'), '5 minutes'))
    //         |.withColumn('n', F.col('n.start')))
    //         |""".stripMargin)
    //   }
    #[test]
    fn test_13() {
        generates(
            r#"bin span=5m n"#,
            r#"spark.table('main').withColumn('n', F.window(F.col('n'), '5 minutes')).withColumn('n', F.col('n.start'))"#,
        );
    }
    //
    //   test("eval count=mvcount(d)") {
    //     generates("eval count=mvcount(d)",
    //       """(spark.table('main')
    //         |.withColumn('count', F.size(F.col('d'))))
    //         |""".stripMargin)
    //   }
    #[test]
    fn test_14() {
        generates(
            r#"eval count=mvcount(d)"#,
            r#"spark.table('main').withColumn('count', F.size(F.col('d')))"#,
        );
    }

    //
    //   test("eval mvsubset=mvindex(d,0,1)") {
    //     generates("eval count=mvindex(d,0,1)",
    //       """(spark.table('main')
    //         |.withColumn('count', F.expr('slice(d, 1, 2)')))
    //         |""".stripMargin)
    //   }
    #[test]
    fn test_15() {
        generates(
            r#"eval count=mvindex(d,0,1)"#,
            // r#"spark.table('main').withColumn('count', F.expr('slice(d, 1, 2)'))"#,
            r#"spark.table('main').withColumn('count', F.slice(F.col('d'), 1, 2))"#,
        );
    }

    //
    //   test("eval mvappended=mvappend(d,d)") {
    //     generates("eval mvappended=mvappend(d,d)",
    //       """(spark.table('main')
    //         |.withColumn('mvappended', F.concat(F.col('d'), F.col('d'))))
    //         |""".stripMargin)
    //   }
    #[test]
    fn test_16() {
        generates(
            r#"eval mvappended=mvappend(d,d)"#,
            r#"spark.table('main').withColumn('mvappended', F.concat(F.col('d'), F.col('d')))"#,
        );
    }

    //
    //   test("count=mvcount(d)") {
    //     generates("eval count=mvcount(d)",
    //       """(spark.table('main')
    //         |.withColumn('count', F.size(F.col('d'))))
    //         |""".stripMargin)
    //   }
    #[test]
    fn test_17() {
        generates(
            r#"eval count=mvcount(d)"#,
            r#"spark.table('main').withColumn('count', F.size(F.col('d')))"#,
        );
    }

    //
    //   test("mvfiltered=mvfilter(d > 3)") {
    //     generates("eval mvfiltered=mvfilter(d > 3)",
    //       """(spark.table('main')
    //         |.withColumn('mvfiltered', F.filter(F.col('d'), lambda d: (d > F.lit(3)))))
    //         |""".stripMargin)
    //   }
    #[test]
    #[ignore]
    fn test_18() {
        generates(
            r#"eval mvfiltered=mvfilter(d > 3)"#,
            r#"spark.table('main').withColumn('mvfiltered', F.filter(F.col('d'), lambda d: (d > F.lit(3))))"#,
        );
    }

    //
    //   test("date=strftime(_time, \"%Y-%m-%d %T\")") {
    //     generates("eval date=strftime(_time, \"%Y-%m-%d %T\")",
    //       """(spark.table('main')
    //         |.withColumn('date', F.date_format(F.col('_time'), 'yyyy-MM-dd HH:mm:ss')))
    //         |""".stripMargin)
    //   }
    #[test]
    fn test_19() {
        generates(
            r#"eval date=strftime(_time, "%Y-%m-%d %T")"#,
            r#"spark.table('main').withColumn('date', F.date_format(F.col('_time'), 'yyyy-MM-dd HH:mm:ss'))"#,
        );
    }

    //
    //   test("min=min(n, 100)") {
    //     generates("eval min=min(n, 100)",
    //       """(spark.table('main')
    //         |.withColumn('min', F.least(F.col('n'), F.lit(100))))
    //         |""".stripMargin)
    //   }
    #[test]
    fn test_20() {
        generates(
            r#"eval min=min(n, 100)"#,
            r#"spark.table('main').withColumn('min', F.least(F.col('n'), F.lit(100)))"#,
        );
    }

    //
    //   test("max=max(n, 0)") {
    //     generates("eval max=max(n, 0)",
    //       """(spark.table('main')
    //         |.withColumn('max', F.greatest(F.col('n'), F.lit(0))))
    //         |""".stripMargin)
    //   }
    #[test]
    fn test_21() {
        generates(
            r#"eval max=max(n, 0)"#,
            r#"spark.table('main').withColumn('max', F.greatest(F.col('n'), F.lit(0)))"#,
        );
    }

    //
    //   test("rounded=round(42.003, 0)") {
    //     generates("eval rounded=round(42.003, 0)",
    //       """(spark.table('main')
    //         |.withColumn('rounded', F.round(F.lit(42.003), 0)))
    //         |""".stripMargin)
    //   }
    #[test]
    fn test_22() {
        generates(
            r#"eval rounded=round(42.003, 0)"#,
            r#"spark.table('main').withColumn('rounded', F.round(F.lit(42.003), 0))"#,
        );
    }

    //
    //   test("sub=substr(a, 3, 5)") {
    //     generates("eval sub=substr(a, 3, 5)",
    //       """(spark.table('main')
    //         |.withColumn('sub', F.substring(F.col('a'), 3, 5)))
    //         |""".stripMargin)
    //   }
    #[test]
    fn test_23() {
        generates(
            r#"eval sub=substr(a, 3, 5)"#,
            r#"spark.table('main').withColumn('sub', F.substring(F.col('a'), 3, 5))"#,
        );
    }

    //
    //   test("lenA=len(a)") {
    //     generates("eval lenA=len(a)",
    //       """(spark.table('main')
    //         |.withColumn('lenA', F.length(F.col('a'))))
    //         |""".stripMargin)
    //   }
    #[test]
    fn test_24() {
        generates(
            r#"eval lenA=len(a)"#,
            r#"spark.table('main').withColumn('lenA', F.length(F.col('a')))"#,
        );
    }

    //
    //   test("dedup 10 host") {
    //     // scalastyle:off
    //     generates("dedup 10 host",
    //       """(spark.table('main')
    //         |# Error in dedup: com.databricks.labs.transpiler.spl.catalyst.EmptyContextOutput: Unable to tanslate com.databricks.labs.transpiler.spl.ast.DedupCommand due to empty context output)
    //         |""".stripMargin)
    //     // scalastyle:on
    //   }
    #[test]
    #[ignore]
    fn test_25() {}

    //
    //   test("format maxresults=10") {
    //     // scalastyle:off
    //     generates("format maxresults=10",
    //       """(spark.table('main')
    //         |# Error in format: com.databricks.labs.transpiler.spl.catalyst.EmptyContextOutput: Unable to tanslate com.databricks.labs.transpiler.spl.ast.FormatCommand due to empty context output)
    //         |""".stripMargin)
    //     // scalastyle:on
    //   }
    #[test]
    #[ignore]
    fn test_26() {}

    //
    //   test("mvcombine host") {
    //     // scalastyle:off
    //     generates("mvcombine host",
    //       """(spark.table('main')
    //         |# Error in mvcombine: com.databricks.labs.transpiler.spl.catalyst.EmptyContextOutput: Unable to tanslate com.databricks.labs.transpiler.spl.ast.MvCombineCommand due to empty context output)
    //         |""".stripMargin)
    //     // scalastyle:on
    //   }
    #[test]
    #[ignore]
    fn test_27() {}

    //
    //   test("makeresults count=10") {
    //     generates("makeresults count=10",
    //       """(spark.range(0, 10, 1)
    //         |.withColumn('_raw', F.lit(None))
    //         |.withColumn('_time', F.current_timestamp())
    //         |.withColumn('host', F.lit(None))
    //         |.withColumn('source', F.lit(None))
    //         |.withColumn('sourcetype', F.lit(None))
    //         |.withColumn('splunk_server', F.lit('local'))
    //         |.withColumn('splunk_server_group', F.lit(None))
    //         |.select('_time'))
    //         |""".stripMargin)
    //   }
    #[test]
    #[ignore]
    fn test_28() {
        generates(
            r#"makeresults count=10"#,
            r#"spark.range(0, 10, 1).withColumn('_raw', F.lit(None)).withColumn('_time', F.current_timestamp()).withColumn('host', F.lit(None)).withColumn('source', F.lit(None)).withColumn('sourcetype', F.lit(None)).withColumn('splunk_server', F.lit('local')).withColumn('splunk_server_group', F.lit(None)).select('_time')"#,
        );
    }

    //
    //   test("addtotals fieldname=num_total num_man num_woman") {
    //     // scalastyle:off
    //     generates("addtotals fieldname=num_total num_man num_woman",
    //       """(spark.table('main')
    //         |.withColumn('num_total', (F.when(F.col('num_woman').cast('double').isNotNull(), F.col('num_woman')).otherwise(F.lit(0.0)) + F.when(F.col('num_man').cast('double').isNotNull(), F.col('num_man')).otherwise(F.lit(0.0)))))
    //         |""".stripMargin)
    //     // scalastyle:on
    //   }
    #[test]
    fn test_29() {
        generates(
            r#"addtotals fieldname=num_total num_man num_woman"#,
            r#"spark.table('main').withColumn(
                'num_total',
                F.when(
                    F.col('num_man').cast('double').isNotNull(),
                    F.col('num_man')
                ).otherwise(
                    F.lit(0.0)
                ) + F.when(
                    F.col('num_woman').cast('double').isNotNull(),
                    F.col('num_woman')
                ).otherwise(
                    F.lit(0.0)
                )
            )"#,
        );
    }

    //
    //   test("custom spl configs") {
    //     spark.conf.set("spl.field._time", "ts")
    //     spark.conf.set("spl.field._raw", "json")
    //     spark.conf.set("spl.index", "custom_table")
    //     spark.conf.set("spl.generator.lineWidth", "80")
    //     spark.range(10).createTempView("custom_table")
    //     val generatedCode = Transpiler.toPython(spark,
    //       "foo > 3 | join type=inner id [makeresults count=10 annotate=t]")
    //     readableAssert(
    //       """(spark.table('custom_table')
    //         |.where((F.col('foo') > F.lit(3)))
    //         |.join(spark.range(0, 10, 1)
    //         |.withColumn('json', F.lit(None))
    //         |.withColumn('ts', F.current_timestamp())
    //         |.withColumn('host', F.lit(None))
    //         |.withColumn('source', F.lit(None))
    //         |.withColumn('sourcetype', F.lit(None))
    //         |.withColumn('splunk_server', F.lit('local'))
    //         |.withColumn('splunk_server_group', F.lit(None))
    //         |.select(F.col('json'),
    //         |  F.col('ts'),
    //         |  F.col('host'),
    //         |  F.col('source'),
    //         |  F.col('sourcetype'),
    //         |  F.col('splunk_server'),
    //         |  F.col('splunk_server_group')),
    //         |['id'], 'inner'))
    //         |""".stripMargin, generatedCode, "Code does not match")
    //      spark.conf.set("spl.field._time", "_time")
    //      spark.conf.set("spl.field._raw", "_raw")
    //      spark.conf.set("spl.index", "main")
    //   }
    #[test]
    #[ignore]
    fn test_30() {}

    //
    //   test("in_range=if(cidrmatch('10.0.0.0/24', src_ip), 1, 0)") {
    //     // scalastyle:off
    //     generates("eval in_range=if(cidrmatch(\"10.0.0.0/24\", src_ip), 1, 0)",
    //       """(spark.table('main')
    //         |.withColumn('in_range', F.when(F.expr("cidr_match('10.0.0.0/24', src_ip)"), F.lit(1)).otherwise(F.lit(0))))
    //         |""".stripMargin)
    //     // scalastyle:on
    //   }
    #[test]
    fn test_31() {
        generates(
            r#"eval in_range=if(cidrmatch("10.0.0.0/24", src_ip), 1, 0)"#,
            r#"spark.table('main').withColumn(
                'in_range',
                F.when(
                    F.expr(
                        "cidr_match('10.0.0.0/24', src_ip)"
                    ),
                    F.lit(1)
                ).otherwise(F.lit(0))
            )"#,
        );
    }

    //
    //   test("in_range=if(cidrmatch(10.0.0.0/24, src_ip), 1, 0)") {
    //     // scalastyle:off
    //     generates("eval in_range=if(cidrmatch(10.0.0.0/24, src_ip), 1, 0)",
    //       """(spark.table('main')
    //         |.withColumn('in_range', F.when(F.expr("cidr_match('10.0.0.0/24', src_ip)"), F.lit(1)).otherwise(F.lit(0))))
    //         |""".stripMargin)
    //     // scalastyle:on
    //   }
    #[test]
    fn test_32() {
        generates(
            r#"eval in_range=if(cidrmatch(10.0.0.0/24, src_ip), 1, 0)"#,
            r#"spark.table('main').withColumn('in_range', F.when(F.expr("cidr_match('10.0.0.0/24', src_ip)"), F.lit(1)).otherwise(F.lit(0)))"#,
        );
    }

    //
    //   test("src_ip = 10.0.0.0/16") {
    //     generates("src_ip = 10.0.0.0/16",
    //       """(spark.table('main')
    //         |.where(F.expr("cidr_match('10.0.0.0/16', src_ip)")))
    //         |""".stripMargin)
    //   }
    #[test]
    fn test_33() {
        generates(
            r#"src_ip = 10.0.0.0/16"#,
            r#"spark.table('main').where(F.expr("cidr_match('10.0.0.0/16', src_ip)"))"#,
        );
    }

    //
    //   test("fsize_quant=memk(fsize)") {
    //     // scalastyle:off
    //     generates("eval fsize_quant=memk(fsize)",
    //     """(spark.table('main')
    //       |.withColumn('fsize_quant', (F.regexp_extract(F.col('fsize'), '(?i)^(\\d*\\.?\\d+)([kmg])$', 1).cast('double') * F.when((F.upper(F.regexp_extract(F.col('fsize'), '(?i)^(\\d*\\.?\\d+)([kmg])$', 2)) == F.lit('K')), F.lit(1.0))
    //       |.when((F.upper(F.regexp_extract(F.col('fsize'), '(?i)^(\\d*\\.?\\d+)([kmg])$', 2)) == F.lit('M')), F.lit(1024.0))
    //       |.when((F.upper(F.regexp_extract(F.col('fsize'), '(?i)^(\\d*\\.?\\d+)([kmg])$', 2)) == F.lit('G')), F.lit(1048576.0))
    //       |.otherwise(F.lit(1.0)))))
    //       |""".stripMargin)
    //     // scalastyle:on
    //   }
    #[test]
    fn test_34() {
        generates(
            r#"eval fsize_quant=memk(fsize)"#,
            r#"spark.table('main').withColumn(
                'fsize_quant',
                F.regexp_extract(
                    F.col('fsize'),
                    '(?i)^(\d*\.?\d+)([kmg])$',
                    1
                ).cast(
                    'double'
                ) * F.when(
                    (
                        F.upper(
                            F.regexp_extract(
                                F.col('fsize'),
                                '(?i)^(\d*\.?\d+)([kmg])$',
                                2
                            )
                        ) == F.lit('K')
                    ),
                    F.lit(1.0)
                ).when(
                    (
                        F.upper(
                            F.regexp_extract(
                                F.col('fsize'),
                                '(?i)^(\d*\.?\d+)([kmg])$',
                                2
                            )
                        ) == F.lit('M')
                    ),
                    F.lit(1024.0)
                ).when(
                    (
                        F.upper(
                            F.regexp_extract(
                                F.col('fsize'),
                                '(?i)^(\d*\.?\d+)([kmg])$',
                                2
                            )
                        ) == F.lit('G')
                    ),
                    F.lit(1048576.0)
                ).otherwise(
                    F.lit(1.0)
                )
            )"#,
        );
    }

    //
    //   test("rmunit=rmunit(fsize)") {
    //     // scalastyle:off
    //     generates("eval rmunit=rmunit(fsize)",
    //       """(spark.table('main')
    //         |.withColumn('rmunit', F.regexp_extract(F.col('fsize'), '(?i)^(\\d*\\.?\\d+)(\\w*)$', 1).cast('double')))
    //         |""".stripMargin)
    //     // scalastyle:on
    //   }
    #[test]
    fn test_35() {
        generates(
            r#"eval rmunit=rmunit(fsize)"#,
            r#"spark.table('main').withColumn(
                'rmunit',
                F.regexp_extract(
                    F.col('fsize'),
                    '(?i)^(\d*\.?\d+)(\w*)$',
                    1
                ).cast('double')
            )"#,
        );
    }

    //
    //   test("rmcomma=rmcomma(s)") {
    //     generates("eval rmcomma=rmcomma(s)",
    //       """(spark.table('main')
    //         |.withColumn('rmcomma', F.regexp_replace(F.col('s'), ',', '').cast('double')))
    //         |""".stripMargin)
    //   }
    #[test]
    fn test_36() {
        generates(
            r#"eval rmcomma=rmcomma(s)"#,
            r#"spark.table('main').withColumn(
                'rmcomma',
                F.regexp_replace(F.col('s'), ',', '').cast('double')
            )"#,
        );
    }

    //
    //   test("convert timeformat=\"%Y\" ctime(_time) AS year") {
    //     generates("convert timeformat=\"%Y\" ctime(_time) AS year",
    //         """(spark.table('main')
    //         |.withColumn('year', F.date_format(F.col('_time'), 'yyyy')))
    //         |""".stripMargin)
    //   }
    #[test]
    fn test_37() {
        generates(
            r#"convert timeformat="%Y" ctime(_time) AS year"#,
            r#"spark.table('main').withColumn('year', F.date_format(F.col('_time'), 'yyyy'))"#,
        );
    }

    //
    //   test("convert timeformat=\"%Y\" num(_time) AS year") {
    //     // scalastyle:off
    //     generates("convert timeformat=\"%Y\" num(_time) AS year",
    //       """(spark.table('main')
    //         |.withColumn('year', F.when(F.date_format(F.col('_time').cast('string'), 'yyyy').isNotNull(), F.date_format(F.col('_time').cast('string'), 'yyyy'))
    //         |.when(F.col('_time').cast('double').isNotNull(), F.col('_time').cast('double'))
    //         |.when((F.regexp_extract(F.col('_time'), '(?i)^(\\d*\\.?\\d+)([kmg])$', 1).cast('double') * F.when((F.upper(F.regexp_extract(F.col('_time'), '(?i)^(\\d*\\.?\\d+)([kmg])$', 2)) == F.lit('K')), F.lit(1.0))
    //         |.when((F.upper(F.regexp_extract(F.col('_time'), '(?i)^(\\d*\\.?\\d+)([kmg])$', 2)) == F.lit('M')), F.lit(1024.0))
    //         |.when((F.upper(F.regexp_extract(F.col('_time'), '(?i)^(\\d*\\.?\\d+)([kmg])$', 2)) == F.lit('G')), F.lit(1048576.0))
    //         |.otherwise(F.lit(1.0))).isNotNull(), (F.regexp_extract(F.col('_time'), '(?i)^(\\d*\\.?\\d+)([kmg])$', 1).cast('double') * F.when((F.upper(F.regexp_extract(F.col('_time'), '(?i)^(\\d*\\.?\\d+)([kmg])$', 2)) == F.lit('K')), F.lit(1.0))
    //         |.when((F.upper(F.regexp_extract(F.col('_time'), '(?i)^(\\d*\\.?\\d+)([kmg])$', 2)) == F.lit('M')), F.lit(1024.0))
    //         |.when((F.upper(F.regexp_extract(F.col('_time'), '(?i)^(\\d*\\.?\\d+)([kmg])$', 2)) == F.lit('G')), F.lit(1048576.0))
    //         |.otherwise(F.lit(1.0))))
    //         |.when(F.regexp_extract(F.col('_time'), '(?i)^(\\d*\\.?\\d+)(\\w*)$', 1).cast('double').isNotNull(), F.regexp_extract(F.col('_time'), '(?i)^(\\d*\\.?\\d+)(\\w*)$', 1).cast('double'))
    //         |.when(F.regexp_replace(F.col('_time'), ',', '').cast('double').isNotNull(), F.regexp_replace(F.col('_time'), ',', '').cast('double'))
    //         |))
    //         |""".stripMargin)
    //     // scalastyle:on
    //   }
    #[test]
    #[ignore]
    fn test_38() {}

    //
    //   test("multisearch in two indices") {
    //     // scalastyle:off
    //     generates("multisearch [index=regionA | fields +country, orders] [index=regionB | fields +country, orders]",
    //       """(spark.table('regionA')
    //         |.select(F.col('country'), F.col('orders')).unionByName(spark.table('regionB')
    //         |.select(F.col('country'), F.col('orders')), allowMissingColumns=True))
    //         |""".stripMargin)
    //     // scalastyle:on
    //   }
    #[test]
    fn test_39() {
        generates(
            r#"multisearch [index=regionA | fields +country, orders] [index=regionB | fields +country, orders]"#,
            r#"spark.table('regionA').select(F.col('country'), F.col('orders')).unionByName(spark.table('regionB').select(F.col('country'), F.col('orders')), allowMissingColumns=True)"#,
        );
    }
    //
    //   test("map search=\"search index=fake_for_join id=$id$\"") {
    //     generates("map search=\"search index=fake_for_join id=$id$\"",
    //       """(spark.table('fake_for_join')
    //         |.limit(10).alias('l')
    //         |.join(spark.table('main').alias('r'),
    //         |(F.col('l.id') == F.col('r.id')),
    //         |'left_semi'))
    //         |""".stripMargin)
    //   }
    #[test]
    #[ignore]
    fn test_40() {
        generates(
            r#"map search="search index=fake_for_join id=$id$""#,
            r#"spark.table('fake_for_join').limit(10).alias('l').join(spark.table('main').alias('r'), (F.col('l.id') == F.col('r.id')), 'left_semi')"#,
        );
    }

    //
    //   test("mvindex with negative start and stop product should " +
    //        "generate an `ConversionFailure` exception in the generated code") {
    //     assert(
    //       extractExceptionIfExists(
    //         "mvindex(sport, -1, 2)").contains(
    //           "A combination of negative and positive start and stop indices is not supported.")
    //     )
    //   }
    #[test]
    #[ignore]
    fn test_41() {}

    //
    //   test("mvfilter referencing more than one field should" +
    //     "generate an `ConversionFailure` exception in the generated code") {
    //     assert(
    //       extractExceptionIfExists(
    //         "mvfilter((score > 50) AND sport = \"football\")").contains(
    //           "Expression references more than one field")
    //     )
    //   }
    #[test]
    #[ignore]
    fn test_42() {}

    //
    //   test("An unknown function should generate an `ConversionFailure` " +
    //        "exception in the generated code") {
    //     assert(
    //       extractExceptionIfExists(
    //         "unknownFn()").contains(
    //           "com.databricks.labs.transpiler.spl.catalyst.ConversionFailure: Unknown SPL function")
    //     )
    //   }
    #[test]
    #[ignore]
    fn test_43() {}

    //
    //   test("replace function should produce a F.regexp_replace(...)") {
    //     generates("replace(myColumn, \"before\", \"after\")",
    //       """(spark.table('main')
    //         |.where(F.regexp_replace(F.col('myColumn'), 'before', 'after')))
    //         |""".stripMargin)
    //   }
    #[test]
    fn test_44() {
        generates(
            r#"replace(myColumn, "before", "after")"#,
            r#"spark.table('main').where(F.regexp_replace(F.col('myColumn'), 'before', 'after'))"#,
        );
    }
    //
    //   test("lower function should produce a F.lower(...)") {
    //     generates("lower(myColumn)",
    //       """(spark.table('main')
    //         |.where(F.lower(F.col('myColumn'))))
    //         |""".stripMargin)
    //   }
    #[test]
    fn test_45() {
        generates(
            r#"lower(myColumn)"#,
            r#"spark.table('main').where(F.lower(F.col('myColumn')))"#,
        );
    }

    //
    //   test("strftime should generate a F.date_format(...) " +
    //        "even if `format` is not quoted") {
    //     generates("strftime(_time, test)",
    //       """(spark.table('main')
    //         |.where(F.date_format(F.col('_time'), 'test')))
    //         |""".stripMargin)
    //   }
    #[test]
    fn test_46() {
        generates(
            r#"strftime(_time, test)"#,
            r#"spark.table('main').where(F.date_format(F.col('_time'), 'test'))"#,
        );
    }

    //
    //   test("strftime should generate a `ConversionFailure` in the code if a wrong format is supplied") {
    //     assert(
    //       extractExceptionIfExists(
    //         "strftime(_time, unknowfn())").contains("Invalid strftime format given")
    //     )
    //   }
    #[test]
    #[ignore]
    fn test_47() {}

    //
    //   test("cidrmatch(...) should generate a `ConversionFailure` in the code") {
    //     assert(
    //       extractExceptionIfExists(
    //         "cidrmatch(ip, substr(ip, 0, 4))").contains(
    //         "com.databricks.labs.transpiler.spl.catalyst.ConversionFailure: ip must be String or Field")
    //     )
    //   }
    #[test]
    #[ignore]
    fn test_48() {}

    //
    //   test("multiple indexes should lead to an exception ") {
    //     val caught = intercept[com.databricks.labs.transpiler.spl.catalyst.ConversionFailure] {
    //       generates("index=A index=B", null)
    //     }
    //     assert(caught.getMessage.contains("Only one index allowed"))
    //   }
    #[test]
    #[ignore]
    fn test_49() {}

    /* Custom tests */
    #[test]
    fn test_head_1() {
        // Ensure that `head 5` generates .limit(5)
        generates(r#"head 5"#, r#"spark.table('main').limit(5)"#);
    }

    #[test]
    fn test_head_2() {
        // Ensure that `head limit=5` generates .limit(5)
        generates(r#"head limit=5"#, r#"spark.table('main').limit(5)"#);
    }

    #[test]
    fn test_head_3() {
        // Ensure that `head count<5` generates .limit(5)
        generates(r#"head count<5"#, r#"spark.table('main').limit(5)"#);
    }

    #[test]
    fn test_head_4() {
        // Ensure that `head count<=5 keeplast=t` generates .limit(7)
        generates(
            r#"head count<=5 keeplast=true"#,
            r#"spark.table('main').limit(7)"#,
        );
    }

    #[test]
    fn test_top_1() {
        generates(
            r#"index=main | top 5 showperc=false x"#,
            r#"spark.table('main').groupBy('x').agg(F.count().alias('count')).orderBy(F.desc(F.col('count'))).limit(5)"#,
        )
    }

    #[test]
    fn test_where_1() {
        generates(
            r#"index=alt | where n>2"#,
            r#"spark.table("alt").where((F.col("n") > F.lit(2)))"#,
        );
    }

    #[test]
    fn test_table_1() {
        generates(
            r#"index=alt | table x y z"#,
            r#"spark.table("alt").select(F.col("x"), F.col("y"), F.col("z"))"#,
        );
    }

    #[test]
    fn test_sort_1() {
        generates(
            r#"index=alt | sort x, -y"#,
            r#"spark.table("alt").orderBy(F.col("x").asc(), F.col("y").desc()).limit(10000)"#,
        );
    }

    #[test]
    fn test_sort_2() {
        generates(
            r#"index=alt | sort 0 x, -y desc"#,
            r#"spark.table("alt").orderBy(F.col("x").desc(), F.col("y").asc())"#,
        );
    }

    #[test]
    fn test_rex_1() {
        generates(
            r#"index=alt | rex field=_raw "From: <(?<from>.*)> To: <(?<to>.*)>""#,
            r#"spark.table("alt").withColumn(
                "from",
                F.regexp_extract(F.col("_raw"), r"From: <(?<from>.*)> To: <(?<to>.*)>", 1)
            ).withColumn(
                "to",
                F.regexp_extract(F.col("_raw"), r"From: <(?<from>.*)> To: <(?<to>.*)>", 2)
            )"#,
        );
    }

    #[test]
    fn test_regex_1() {
        generates(
            r#"index=alt | regex "From: <(?<from>.*)> To: <(?<to>.*)>""#,
            r#"spark.table("alt").where(
                F.regexp_like(F.col("_raw"), r"From: <(?<from>.*)> To: <(?<to>.*)>")
            )"#,
        );
    }

    #[test]
    fn test_regex_2() {
        generates(
            r#"index=alt | regex c!="From: <(?<from>.*)> To: <(?<to>.*)>""#,
            r#"spark.table("alt").where(
                ~F.regexp_like(F.col("c"), r"From: <(?<from>.*)> To: <(?<to>.*)>")
            )"#,
        );
    }

    #[test]
    fn test_rename_1() {
        generates(
            r#"index=alt | rename x AS y, a AS b"#,
            r#"spark.table("alt").withColumnRenamed("x", "y").withColumnRenamed("a", "b")"#,
        );
    }
}
