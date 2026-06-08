import asyncio
from unittest.mock import AsyncMock, Mock, patch

import grpc

from vortexdb.async_connection import AsyncGRPCConnection
from vortexdb.config import VortexDBConfig
from vortexdb.exceptions import (
    AuthenticationError,
    InternalServerError,
    InvalidArgumentError,
    NotFoundError,
    ServiceUnavailableError,
    TimeoutError,
)


class FakeAioRpcError:
    """
    Minimal AioRpcError-compatible object for mapping tests.
    """

    def __init__(self, status_code: grpc.StatusCode, details: str):
        self._status_code = status_code
        self._details = details

    def code(self):
        return self._status_code

    def details(self):
        return self._details


def make_config() -> VortexDBConfig:
    return VortexDBConfig(
        grpc_url="localhost:50051",
        api_key="secret",
        timeout=3.0,
    )


def test_async_channel_created_with_correct_url():
    with patch("grpc.aio.insecure_channel") as mock_channel:
        mock_channel.return_value = Mock()
        AsyncGRPCConnection(make_config())
        mock_channel.assert_called_once_with("localhost:50051")


def test_async_metadata_is_attached():
    with patch("grpc.aio.insecure_channel") as mock_channel:
        mock_channel.return_value = Mock()
        connection = AsyncGRPCConnection(make_config())

    assert ("authorization", "Bearer secret") in connection._metadata


def test_successful_async_rpc_call():
    async def run():
        with patch("grpc.aio.insecure_channel") as mock_channel:
            mock_channel.return_value = Mock()
            connection = AsyncGRPCConnection(make_config())

        fake_rpc = AsyncMock(return_value="ok")

        result = await connection.call(fake_rpc, request="req")

        fake_rpc.assert_awaited_once_with(
            "req",
            timeout=3.0,
            metadata=connection._metadata,
        )
        assert result == "ok"

    asyncio.run(run())


def test_async_grpc_error_mapping():
    cases = [
        (grpc.StatusCode.UNAUTHENTICATED, AuthenticationError),
        (grpc.StatusCode.NOT_FOUND, NotFoundError),
        (grpc.StatusCode.INVALID_ARGUMENT, InvalidArgumentError),
        (grpc.StatusCode.DEADLINE_EXCEEDED, TimeoutError),
        (grpc.StatusCode.UNAVAILABLE, ServiceUnavailableError),
        (grpc.StatusCode.UNKNOWN, InternalServerError),
    ]

    for status_code, expected_exception in cases:
        error = FakeAioRpcError(status_code, "boom")
        mapped = AsyncGRPCConnection._map_grpc_error(error)
        assert isinstance(mapped, expected_exception)


def test_async_close_closes_channel():
    async def run():
        with patch("grpc.aio.insecure_channel") as mock_channel:
            channel = Mock()
            channel.close = AsyncMock()
            mock_channel.return_value = channel
            connection = AsyncGRPCConnection(make_config())

        await connection.close()

        channel.close.assert_awaited_once()

    asyncio.run(run())
