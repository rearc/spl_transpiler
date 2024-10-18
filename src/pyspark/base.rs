use std::fmt::Display;
// use crate::format_python::format_python_code;
use anyhow::Result;

pub struct PythonCode {
    pub preface: Vec<String>,
    pub primary_df_code: String,
}

impl PythonCode {
    pub fn new(primary_df_code: String, preface: Vec<String>, root: Option<PythonCode>) -> Self {
        let mut new_preface = vec![];
        if let Some(root) = root {
            new_preface.extend(root.preface);
        }
        new_preface.extend(preface);
        Self {
            primary_df_code,
            preface: new_preface,
        }
    }
}

impl From<String> for PythonCode {
    fn from(primary_df_code: String) -> Self {
        Self {
            primary_df_code: primary_df_code.to_string(),
            preface: Vec::new(),
        }
    }
}

impl Display for PythonCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for line in self.preface.iter() {
            writeln!(f, "{}", line)?;
        }
        write!(f, "{}", self.primary_df_code)
    }
}

pub trait ToSparkQuery {
    fn to_spark_query(&self) -> Result<PythonCode>;
}
