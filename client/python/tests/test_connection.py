import grpc
import pytest
from unittest.mock import Mock, patch

from vortexdb._grpc_common import map_grpc_error
from vortexdb.connection import GRPCConnection
from vortexdb.config import VortexDBConfig
from vortexdb.exceptions import (
    AuthenticationError,
    NotFoundError,
    InvalidArgumentError,
    TimeoutError,
    ServiceUnavailableError,
    InternalServerError,
)

# Fake gRPC error, required for testing


class FakeRpcError(grpc.RpcError):
    """
    RpcError implementation for unit testing.
    grpc.RpcError cannot be instantiated directly.
    """

    def __init__(self, status_code: grpc.StatusCode, details: str):
        self._status_code = status_code
        self._details = details

    def code(self):
        return self._status_code

    def details(self):
        return self._details


# Pytest fixtures for config and channel


@pytest.fixture
def config():
    return VortexDBConfig(
        grpc_url="localhost:50051",
        api_key="secret",
        timeout=3.0,
    )


@pytest.fixture
def connection(config):
    with patch("grpc.insecure_channel") as mock_channel:
        mock_channel.return_value = Mock()
        yield GRPCConnection(config)


# Basic connection testing


def test_channel_created_with_correct_url(config):
    with patch("grpc.insecure_channel") as mock_channel:
        GRPCConnection(config)
        mock_channel.assert_called_once_with("localhost:50051")


def test_metadata_is_attached(connection):
    assert ("authorization", "Bearer secret") in connection._metadata


def test_successful_rpc_call(connection):
    fake_rpc = Mock(return_value="ok")

    result = connection.call(fake_rpc, request="req")

    fake_rpc.assert_called_once_with(
        "req",
        timeout=3.0,
        metadata=connection._metadata,
    )
    assert result == "ok"


# Error mapping test


@pytest.mark.parametrize(
    "status_code,expected_exception",
    [
        (grpc.StatusCode.UNAUTHENTICATED, AuthenticationError),
        (grpc.StatusCode.NOT_FOUND, NotFoundError),
        (grpc.StatusCode.INVALID_ARGUMENT, InvalidArgumentError),
        (grpc.StatusCode.DEADLINE_EXCEEDED, TimeoutError),
        (grpc.StatusCode.UNAVAILABLE, ServiceUnavailableError),
    ],
)
def test_grpc_error_mapping(status_code, expected_exception, connection):
    error = FakeRpcError(status_code, "boom")

    mapped = map_grpc_error(error)

    assert isinstance(mapped, expected_exception)


def test_unknown_grpc_error_maps_to_internal_error(connection):
    error = FakeRpcError(grpc.StatusCode.UNKNOWN, "unknown")
    mapped = map_grpc_error(error)

    assert isinstance(mapped, InternalServerError)


# Clean connection closure test


def test_close_closes_channel(config):
    with patch("grpc.insecure_channel") as mock_channel:
        mock_channel.return_value = Mock()
        conn = GRPCConnection(config)
        conn.close()
        conn._channel.close.assert_called_once()
