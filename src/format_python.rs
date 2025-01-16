use anyhow::Result;

pub fn format_python_code(code: impl ToString) -> Result<String> {
    let format_options = Default::default();
    let code = code.to_string();
    let code = textwrap::dedent(code.as_str());
    let code = code.trim();
    let code = ruff_python_formatter::format_module_source(code, format_options)?.into_code();
    let code = code.trim_end();
    Ok(code.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    fn test_format_python_code() {
        assert_eq!(
            format_python_code(r#"some_func( 'yolo')"#.to_string()).unwrap(),
            r#"some_func("yolo")"#.to_string(),
        );
    }
}
