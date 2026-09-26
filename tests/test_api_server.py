def test_api_server_exports_fastapi_app():
    from api_server import app

    assert app.title == "Hermes Agent API"
