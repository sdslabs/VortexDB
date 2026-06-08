import asyncio
from unittest.mock import AsyncMock, Mock

import pytest

from vortexdb.async_client import AsyncVortexDB
from vortexdb.async_connection import AsyncGRPCConnection
from vortexdb.models import ContentType, DenseVector, Payload, Point, Similarity


@pytest.fixture
def mock_connection(monkeypatch):
    """
    Replace AsyncGRPCConnection with a mock instance.
    """
    conn = Mock(spec=AsyncGRPCConnection)
    conn.stub = Mock()
    conn.call = AsyncMock()
    conn.close = AsyncMock()
    monkeypatch.setattr("vortexdb.async_client.AsyncGRPCConnection", lambda _: conn)
    return conn


@pytest.fixture
def client(mock_connection):
    return AsyncVortexDB(
        grpc_url="localhost:50051",
        api_key="secret",
    )


def test_async_insert_success(client, mock_connection):
    async def run():
        response = Mock()
        response.id = Mock()
        response.id.value = "point-123"

        mock_connection.call.return_value = response

        point_id = await client.insert(
            vector=DenseVector([1, 2, 3]),
            payload=Payload.text("hello"),
        )

        assert point_id == "point-123"

    asyncio.run(run())


def test_async_insert_rejects_invalid_vector(client):
    async def run():
        with pytest.raises(TypeError):
            await client.insert(
                vector=[1, 2, 3],
                payload=Payload.text("hello"),
            )

    asyncio.run(run())


def test_async_get_point_success(client, mock_connection):
    async def run():
        proto_point = Mock()
        proto_point.id.id.value = "point-123"
        proto_point.vector.values = [1, 2, 3]
        proto_point.payload.content_type = ContentType.TEXT.to_proto()
        proto_point.payload.content = "hello"

        mock_connection.call.return_value = proto_point

        point = await client.get(point_id="point-123")

        assert isinstance(point, Point)
        assert point.id == "point-123"
        assert point.payload.content == "hello"

    asyncio.run(run())


def test_async_get_point_not_found(client, mock_connection):
    async def run():
        mock_connection.call.return_value = None

        result = await client.get(point_id="missing")

        assert result is None

    asyncio.run(run())


def test_async_delete_success(client, mock_connection):
    async def run():
        mock_connection.call.return_value = None

        await client.delete(point_id="point-123")

        mock_connection.call.assert_awaited_once()

    asyncio.run(run())


def test_async_search_success(client, mock_connection):
    async def run():
        mock_connection.call.return_value = Mock(
            result_point_ids=[
                Mock(id=Mock(value="p1")),
                Mock(id=Mock(value="p2")),
            ]
        )

        results = await client.search(
            vector=DenseVector([1, 2, 3]),
            similarity=Similarity.COSINE,
            limit=2,
        )

        assert results == ["p1", "p2"]

    asyncio.run(run())


def test_async_search_invalid_vector(client):
    async def run():
        with pytest.raises(TypeError):
            await client.search(
                vector=[1, 2, 3],
                similarity=Similarity.COSINE,
                limit=2,
            )

    asyncio.run(run())


def test_async_close_closes_connection(client, mock_connection):
    async def run():
        await client.close()
        mock_connection.close.assert_awaited_once()

    asyncio.run(run())


def test_async_context_manager_closes_connection(monkeypatch):
    async def run():
        conn = Mock(spec=AsyncGRPCConnection)
        conn.stub = Mock()
        conn.call = AsyncMock()
        conn.close = AsyncMock()
        monkeypatch.setattr("vortexdb.async_client.AsyncGRPCConnection", lambda _: conn)

        async with AsyncVortexDB(
            grpc_url="localhost:50051",
            api_key="secret",
        ) as db:
            assert db is not None

        conn.close.assert_awaited_once()

    asyncio.run(run())
