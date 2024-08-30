use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

mod ast;
mod format_python;
mod pyspark;
mod spl;

use pyspark::{TemplateNode, TransformedPipeline};

#[pymodule]
fn spl_transpiler(m: &Bound<'_, PyModule>) -> PyResult<()> {
    #[pyfn(m)]
    /// Parses SPL query code into a syntax tree.
    fn parse(spl_code: &str) -> PyResult<ast::ast::Pipeline> {
        match spl::pipeline(spl_code) {
            Ok(("", pipeline)) => Ok(pipeline),
            Ok(_) => Err(PyValueError::new_err("Failed to fully parse input")),
            Err(e) => Err(PyValueError::new_err(format!("Error parsing SPL: {}", e))),
        }
    }

    #[pyfn(m)]
    #[pyo3(signature = (pipeline, format=true))]
    /// Renders a parsed SPL syntax tree into equivalent PySpark query code, if possible.
    fn render_pyspark(pipeline: &ast::ast::Pipeline, format: bool) -> PyResult<String> {
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
        let pipeline: ast::ast::Pipeline = parse(spl_code)?;
        Ok(render_pyspark(&pipeline, format)?)
    }

    let ast_m = PyModule::new_bound(m.py(), "ast")?;
    ast::python::ast(&ast_m)?;
    m.add_submodule(&ast_m)?;

    Ok(())
}
