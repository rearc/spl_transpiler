if __name__ == "__main__":
    try:
        from spl_transpiler.live_translator import SPLApp
    except ImportError as e:
        raise ImportError(
            "Cannot import spl_transpiler live translator, please make sure to `pip install spl_transpiler[cli]`"
        ) from e

    app = SPLApp()
    app.run()
