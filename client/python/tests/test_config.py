import pytest

from vortexdb.config import VortexDBConfig, ConfigurationError


# Clean slate env vars for every test
@pytest.fixture
def clean_env(monkeypatch):
    for var in [
        "VORTEXDB_GRPC_URL",
        "VORTEXDB_API_KEY",
        "VORTEXDB_TIMEOUT",
    ]:
        monkeypatch.delenv(var, raising=False)


# Checking from_env


def test_config_requires_api_key(clean_env):
    with pytest.raises(ConfigurationError):
        VortexDBConfig.from_env()


def test_config_from_explicit_args(clean_env):
    cfg = VortexDBConfig.from_env(
        grpc_url="localhost:50051",
        api_key="secret",
        timeout=10.0,
    )

    assert cfg.grpc_url == "localhost:50051"
    assert cfg.api_key == "secret"
    assert cfg.timeout == 10.0


# Env vars fallback


def test_config_from_env_vars(clean_env, monkeypatch):
    monkeypatch.setenv("VORTEXDB_GRPC_URL", "127.0.0.1:1234")
    monkeypatch.setenv("VORTEXDB_API_KEY", "env-secret")
    monkeypatch.setenv("VORTEXDB_TIMEOUT", "7.5")

    cfg = VortexDBConfig.from_env()

    assert cfg.grpc_url == "127.0.0.1:1234"
    assert cfg.api_key == "env-secret"
    assert cfg.timeout == 7.5


# Defaults


def test_config_default_grpc_url(clean_env, monkeypatch):
    monkeypatch.setenv("VORTEXDB_API_KEY", "secret")

    cfg = VortexDBConfig.from_env()

    assert cfg.grpc_url == "localhost:50051"


def test_config_default_timeout(clean_env, monkeypatch):
    monkeypatch.setenv("VORTEXDB_API_KEY", "secret")

    cfg = VortexDBConfig.from_env()

    assert cfg.timeout == 5.0


# Invalid Timeout


def test_config_invalid_timeout(clean_env, monkeypatch):
    monkeypatch.setenv("VORTEXDB_API_KEY", "secret")
    monkeypatch.setenv("VORTEXDB_TIMEOUT", "not-a-number")

    with pytest.raises(ValueError):
        VortexDBConfig.from_env()
