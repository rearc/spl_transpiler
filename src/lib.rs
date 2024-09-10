use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use spl::parser;

pub(crate) mod commands;
pub(crate) mod format_python;
pub(crate) mod functions;
pub(crate) mod pyspark;
pub(crate) mod spl;

use pyspark::{TemplateNode, TransformedPipeline};

#[pymodule]
fn spl_transpiler(m: &Bound<'_, PyModule>) -> PyResult<()> {
    #[pyfn(m)]
    /// Parses SPL query code into a syntax tree.
    fn parse(spl_code: &str) -> PyResult<spl::ast::Pipeline> {
        match parser::pipeline(spl_code) {
            Ok(("", pipeline)) => Ok(pipeline),
            Ok(_) => Err(PyValueError::new_err("Failed to fully parse input")),
            Err(e) => Err(PyValueError::new_err(format!("Error parsing SPL: {}", e))),
        }
    }

    #[pyfn(m)]
    #[pyo3(signature = (pipeline, format=true))]
    /// Renders a parsed SPL syntax tree into equivalent PySpark query code, if possible.
    fn render_pyspark(pipeline: &spl::ast::Pipeline, format: bool) -> PyResult<String> {
        let transformed_pipeline: TransformedPipeline = pipeline.clone().try_into()?;
        let mut code = transformed_pipeline.to_spark_query()?;
        if format {
            code = format_python::safe_format_python_code(code);
        }
        Ok(code)
    }

    #[pyfn(m)]
    #[pyo3(signature = (spl_code, format=true))]
    /// Converts SPL query code directly into equivalent PySpark query code, if possible.
    fn convert_spl_to_pyspark(spl_code: &str, format: bool) -> PyResult<String> {
        let pipeline: spl::ast::Pipeline = parse(spl_code)?;
        render_pyspark(&pipeline, format)
    }

    let ast_m = PyModule::new_bound(m.py(), "ast")?;
    spl::python::ast(&ast_m)?;
    m.add_submodule(&ast_m)?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;

    Ok(())
}
