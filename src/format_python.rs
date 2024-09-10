use anyhow::Result;
use log::{error, warn};
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug, Clone)]
struct BlackNotInstalled();

impl Display for BlackNotInstalled {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Black is not installed, please install it with `pip install black`."
        )
    }
}

impl Error for BlackNotInstalled {}

pub fn format_python_code(code: impl ToString) -> Result<String> {
    // TODO: When ruff format is available on cargo, use that instead of an embedded Python interpeter
    // from black import format_str, FileMode
    // res = format_str("some python code", mode=FileMode())
    let result: Result<String> = Python::with_gil(|py| {
        let black = PyModule::import_bound(py, "black").map_err(|_| BlackNotInstalled())?;
        let mode = black.getattr("FileMode")?.call0()?;
        let args = (code.to_string(),);
        let kwargs = [("mode", mode)].into_py_dict_bound(py);
        let formatted = black.getattr("format_str")?.call(args, Some(&kwargs))?;
        // TODO: ::from_object_bound is about to be deprecated, renamed to ::from_object in 0.23.0
        // let py_s: Bound<PyString> = formatted.to_string();
        // let s = py_s.to_cow()?;
        // Ok(String::from(s))
        Ok(formatted.to_string().trim().to_string())
    });
    result
}

pub fn safe_format_python_code(code: impl ToString) -> String {
    let s = code.to_string();
    match format_python_code(s.clone()) {
        Ok(formatted) => formatted,
        Err(e) if e.to_string().contains("Black is not installed") => {
            warn!("{:?}. Returning unformatted code", e);
            s
        }
        Err(e) => {
            error!(
                "Python formatting failed unexpectedly, resulting code may be invalid: {:?}",
                e
            );
            s
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // #[ignore]
    fn test_format_python_code() {
        assert_eq!(
            format_python_code(r#"some_func( 'yolo')"#.to_string()).unwrap(),
            r#"some_func("yolo")"#.to_string(),
        );
    }
}
