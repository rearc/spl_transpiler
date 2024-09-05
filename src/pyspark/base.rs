// use crate::format_python::format_python_code;
use anyhow::Result;

pub trait TemplateNode {
    fn to_spark_query(&self) -> Result<String>;

    // fn to_formatted_spark_query(&self) -> Result<String> {
    //     Ok(format_python_code(self.to_spark_query()?)?)
    // }
}
