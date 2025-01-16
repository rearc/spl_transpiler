#[cfg(test)]
pub mod test {
    use crate::format_python::format_python_code;
    use crate::pyspark::base::test::test_pyspark_transpile_context;
    use crate::pyspark::base::RuntimeSelection;
    use crate::pyspark::{ToSparkQuery, TransformedPipeline};
    use anyhow::Result;
    use log::debug;
    use regex::Regex;
    use std::ops::Deref;

    pub fn assert_python_code_eq(
        generated_code: impl ToString,
        reference_code: impl ToString,
        remove_newlines: bool,
    ) {
        let generated_code = _cleanup(generated_code, remove_newlines)
            .expect("Failed to format rendered Spark query");
        let reference_code =
            _cleanup(reference_code, remove_newlines).expect("Failed to format target Spark query");
        assert_eq!(generated_code, reference_code);
    }

    fn _remove_extraneous_newlines(code: impl ToString) -> Result<String> {
        let re: Regex = Regex::new(r#"[\n ]+"#)?;
        Ok(re
            .replace_all(code.to_string().as_str(), " ")
            .deref()
            .to_string())
    }

    fn _remove_trailing_commas(code: impl ToString) -> Result<String> {
        let re: Regex = Regex::new(r#",\s*([)\]])"#)?;
        Ok(re
            .replace_all(code.to_string().as_str(), "$1")
            .deref()
            .to_string())
    }

    fn _cleanup(code: impl ToString, remove_newlines: bool) -> Result<String> {
        let mut code = format_python_code(code.to_string().replace(",)", ")"))?;
        if remove_newlines {
            code = _remove_extraneous_newlines(code)?;
        }
        code = _remove_trailing_commas(code)?;
        Ok(code)
    }

    pub fn generates(spl_query: &str, spark_query: &str) {
        let ctx = test_pyspark_transpile_context(RuntimeSelection::Disallow);
        let (_, pipeline_ast) =
            crate::parser::pipeline(spl_query).expect("Failed to parse SPL query");
        let rendered = TransformedPipeline::transform(pipeline_ast, ctx.clone())
            .expect("Failed to convert SPL query to Spark query")
            .to_spark_query(&ctx)
            .expect("Failed to render Spark query");

        assert_python_code_eq(rendered, spark_query, true);
    }

    pub fn generates_runtime(spl_query: &str, spark_query: &str) {
        let ctx = test_pyspark_transpile_context(RuntimeSelection::Require);
        let (_, pipeline_ast) =
            crate::parser::pipeline(spl_query).expect("Failed to parse SPL query");
        let rendered = TransformedPipeline::transform(pipeline_ast, ctx.clone())
            .expect("Failed to convert SPL query to Spark query")
            .to_spark_query(&ctx)
            .expect("Failed to render Spark query");

        assert_python_code_eq(rendered, spark_query, false);
    }

    pub fn generates_maybe_runtime(spl_query: &str, spark_query: &str) {
        let ctx = test_pyspark_transpile_context(RuntimeSelection::Allow);
        let (_, pipeline_ast) =
            crate::parser::pipeline(spl_query).expect("Failed to parse SPL query");
        let rendered = TransformedPipeline::transform(pipeline_ast, ctx.clone())
            .expect("Failed to convert SPL query to Spark query")
            .to_spark_query(&ctx)
            .expect("Failed to render Spark query");

        debug!("Generated code: {}", rendered.to_string());

        assert_python_code_eq(rendered, spark_query, false);
    }
}
