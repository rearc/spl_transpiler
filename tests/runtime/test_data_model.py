from spl_transpiler.runtime import commands
from spl_transpiler import convert_spl_to_pyspark
from utils import assert_python_code_equals


def test_basic_data_model(sample_data_2):
    df = commands.data_model("Model")
    model_results = df.collect()
    known_results = sample_data_2.collect()
    assert model_results == known_results


def test_transpiled_data_model():
    transpiled_code = convert_spl_to_pyspark(
        "datamodel Model search",
        allow_runtime=True,
        format_code=True,
    )
    expected_code = r"""
    df_1 = commands.data_model(None, data_model_name="Model", dataset_name=None, search_mode="search", strict_fields=False, allow_old_summaries=False, summaries_only=True)
    df_1
    """
    assert_python_code_equals(transpiled_code, expected_code)
