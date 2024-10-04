#[cfg(test)]
pub mod test {
    use crate::format_python::format_python_code;
    use crate::pyspark::{convert, TemplateNode};
    use anyhow::Result;
    use regex::Regex;
    use std::ops::Deref;

    pub fn assert_python_code_eq(generated_code: impl ToString, reference_code: impl ToString) {
        let generated_code =
            _cleanup(generated_code).expect("Failed to format rendered Spark query");
        let reference_code = _cleanup(reference_code).expect("Failed to format target Spark query");
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

    fn _cleanup(code: impl ToString) -> Result<String> {
        let code = format_python_code(code.to_string().replace(",)", ")"))?;
        let code = _remove_extraneous_newlines(code)?;
        let code = _remove_trailing_commas(code)?;
        Ok(code)
    }

    pub fn generates(spl_query: &str, spark_query: &str) {
        let (_, pipeline_ast) =
            crate::parser::pipeline(spl_query).expect("Failed to parse SPL query");
        let rendered = convert(pipeline_ast)
            .expect("Failed to convert SPL query to Spark query")
            .to_spark_query()
            .expect("Failed to render Spark query");

        assert_python_code_eq(rendered, spark_query);
    }
}
