#[cfg(test)]
pub mod test {
    use crate::format_python::format_python_code;
    use crate::pyspark::{convert, TemplateNode};

    pub fn assert_python_code_eq(generated_code: impl ToString, reference_code: impl ToString) {
        let formatted_generated = format_python_code(generated_code.to_string().replace(",)", ")"))
            .expect("Failed to format rendered Spark query");
        let formatted_reference = format_python_code(reference_code.to_string().replace(",)", ")"))
            .expect("Failed to format target Spark query");
        assert_eq!(formatted_generated, formatted_reference);
    }

    pub fn generates(spl_query: &str, spark_query: &str) {
        let (_, pipeline_ast) =
            crate::parser::pipeline(spl_query).expect("Failed to parse SPL query");
        let converted = convert(pipeline_ast).expect("Failed to convert SPL query to Spark query");
        let rendered = converted
            .to_spark_query()
            .expect("Failed to render Spark query");
        assert_python_code_eq(rendered, spark_query);
    }
}
