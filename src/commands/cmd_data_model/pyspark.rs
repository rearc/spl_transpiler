//noinspection RsDetachedFile
use super::spl::*;
use crate::pyspark::ast::*;
use crate::pyspark::transpiler::{PipelineTransformState, PipelineTransformer};
use anyhow::{bail, ensure};
use log::warn;

impl PipelineTransformer for DataModelCommand {
    fn transform_for_runtime(
        &self,
        state: PipelineTransformState,
    ) -> anyhow::Result<PipelineTransformState> {
        let df = state.df.clone();
        let mut kwargs = py_dict! {};

        kwargs.push(
            "data_model_name",
            PyLiteral::from(self.data_model_name.clone()),
        );
        kwargs.push("dataset_name", PyLiteral::from(self.dataset_name.clone()));
        kwargs.push("search_mode", PyLiteral::from(self.search_mode.clone()));
        kwargs.push("strict_fields", PyLiteral::from(self.strict_fields));
        kwargs.push(
            "allow_old_summaries",
            PyLiteral::from(self.allow_old_summaries),
        );
        kwargs.push("summaries_only", PyLiteral::from(self.summaries_only));

        let df = DataFrame::runtime(df, "data_model", vec![], kwargs.0, &state.ctx);

        Ok(state.with_df(df))
    }

    fn transform_standalone(
        &self,
        state: PipelineTransformState,
    ) -> anyhow::Result<PipelineTransformState> {
        if !self.summaries_only {
            warn!("`datamodel` `summariesonly` argument has no effect in PySpark")
        }
        if !self.allow_old_summaries {
            warn!("`datamodel` `allow_old_summaries` argument has no effect in PySpark")
        }
        if !self.strict_fields {
            warn!("`datamodel` `strict_fields` argument has no effect in PySpark")
        }
        ensure!(
            self.search_mode == Some("search".to_string()),
            "UNIMPLEMENTED: `datamodel` command does not support search_mode other than 'search'"
        );

        let df = match (state.df.clone(), self.data_model_name.clone(), self.dataset_name.clone()) {
            (Some(src), None, None) => src,
            (None, data_model, node_name) => DataFrame::source(match (data_model, node_name) {
                (Some(data_model), None) => data_model,
                (Some(data_model), Some(node_name)) => {
                    // data_model = <data_model_name>.<root_dataset_name>
                    // node_name = <root_dataset_name>.<...>.<target_dataset_name>
                    // return <data_model_name>.<root_dataset_name>.<...>.<target_dataset_name>
                    let truncated_node_name = node_name.split(".").skip(1).collect::<Vec<_>>().join(".");
                    format!("{}.{}", data_model, truncated_node_name)
                }
                _ => bail!("I think we received a dataset ({:?}) with no data model ({:?}) in `datamodel`?", self.dataset_name, self.data_model_name),
            }),
            _ => bail!("UNIMPLEMENTED: `tstats` command requires a source DataFrame and either a data model or a node name"),
        };

        Ok(state.with_df(df))
    }
}
