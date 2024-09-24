#[cfg(test)]
pub mod test {
    use crate::format_python::format_python_code;
    use crate::pyspark::{convert, TemplateNode};

    pub fn generates(spl_query: &str, spark_query: &str) {
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
}
