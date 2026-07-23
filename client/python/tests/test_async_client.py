import asyncio
from unittest.mock import AsyncMock, Mock

import pytest

from vortexdb.async_client import AsyncVortexDB
from vortexdb.async_connection import AsyncGRPCConnection
from vortexdb.models import (
    ContentType,
    DenseVector,
    Payload,
    Point,
    Similarity,
    SearchQuery,
)


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


# Batch Insert


def test_async_batch_insert_success(client, mock_connection):
    async def run():
        response = Mock()
        response.ids = [
            Mock(id=Mock(value="p1")),
            Mock(id=Mock(value="p2")),
        ]
        mock_connection.call.return_value = response

        items = [
            (DenseVector([1, 2, 3]), Payload.text("a")),
            (DenseVector([4, 5, 6]), Payload.text("b")),
        ]
        result = await client.batch_insert(items=items)
        assert result == ["p1", "p2"]

    asyncio.run(run())


def test_async_batch_insert_invalid_items_type(client):
    async def run():
        with pytest.raises(TypeError):
            await client.batch_insert(items="not-a-list")

    asyncio.run(run())


def test_async_batch_insert_invalid_tuple_structure(client):
    async def run():
        items = [
            (DenseVector([1, 2, 3]),),  # only one element
        ]
        with pytest.raises(TypeError):
            await client.batch_insert(items=items)

    asyncio.run(run())


def test_async_batch_insert_invalid_vector(client):
    async def run():
        items = [
            ([1, 2, 3], Payload.text("a")),  # not DenseVector
        ]
        with pytest.raises(TypeError):
            await client.batch_insert(items=items)

    asyncio.run(run())


# Get


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


# Delete


def test_async_delete_success(client, mock_connection):
    async def run():
        mock_connection.call.return_value = None

        await client.delete(point_id="point-123")

        mock_connection.call.assert_awaited_once()

    asyncio.run(run())


# Search


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


def test_async_search_with_query_object(client, mock_connection):
    async def run():
        mock_connection.call.return_value = Mock(
            result_point_ids=[
                Mock(id=Mock(value="p1")),
            ]
        )

        q = SearchQuery(DenseVector([1, 2, 3]), Similarity.COSINE, 2)
        results = await client.search(query=q)

        assert results == ["p1"]

    asyncio.run(run())


def test_async_search_accepts_ef(client, mock_connection):
    async def run():
        mock_connection.call.return_value = Mock(result_point_ids=[])

        await client.search(
            vector=DenseVector([1, 2, 3]),
            similarity=Similarity.COSINE,
            limit=2,
            ef=128,
        )

        request = mock_connection.call.call_args.args[1]
        assert request.ef == 128

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


# Batch Search


def test_async_batch_search_full_tuple(client, mock_connection):
    async def run():
        mock_connection.call.return_value = Mock(
            results=[
                Mock(result_point_ids=[Mock(id=Mock(value="p1"))]),
                Mock(result_point_ids=[Mock(id=Mock(value="p2"))]),
            ]
        )
        queries = [
            (DenseVector([1, 2, 3]), Similarity.COSINE, 2),
            (DenseVector([4, 5, 6]), Similarity.EUCLIDEAN, 1),
        ]
        result = await client.batch_search(queries=queries)
        assert result == [["p1"], ["p2"]]

    asyncio.run(run())


def test_async_batch_search_searchquery_objects(client, mock_connection):
    async def run():
        mock_connection.call.return_value = Mock(
            results=[
                Mock(result_point_ids=[Mock(id=Mock(value="p1"))]),
            ]
        )
        queries = [
            SearchQuery(DenseVector([1, 2, 3]), Similarity.COSINE, 2),
        ]
        result = await client.batch_search(queries=queries)
        assert result == [["p1"]]

    asyncio.run(run())


def test_async_batch_search_vectors_with_global_params(client, mock_connection):
    async def run():
        mock_connection.call.return_value = Mock(
            results=[
                Mock(result_point_ids=[Mock(id=Mock(value="p1"))]),
            ]
        )
        queries = [DenseVector([1, 2, 3])]
        result = await client.batch_search(
            queries=queries,
            similarity=Similarity.MANHATTAN,
            limit=2,
        )
        assert result == [["p1"]]

    asyncio.run(run())


def test_async_batch_search_vector_similarity_with_global_limit(
    client, mock_connection
):
    async def run():
        mock_connection.call.return_value = Mock(
            results=[
                Mock(result_point_ids=[Mock(id=Mock(value="p1"))]),
            ]
        )
        queries = [
            (DenseVector([1, 2, 3]), Similarity.COSINE),
        ]
        result = await client.batch_search(
            queries=queries,
            limit=2,
        )
        assert result == [["p1"]]

    asyncio.run(run())


def test_async_batch_search_accepts_ef(client, mock_connection):
    async def run():
        mock_connection.call.return_value = Mock(results=[])

        await client.batch_search(
            queries=[
                (DenseVector([1, 2, 3]), Similarity.COSINE, 2),
                (DenseVector([4, 5, 6]), Similarity.COSINE, 1),
            ],
            ef=256,
        )

        request = mock_connection.call.call_args.args[1]
        assert [query.ef for query in request.queries] == [256, 256]

    asyncio.run(run())


def test_async_batch_search_missing_globals_for_vector(client):
    async def run():
        queries = [DenseVector([1, 2, 3])]
        with pytest.raises(ValueError):
            await client.batch_search(queries=queries)

    asyncio.run(run())


def test_async_batch_search_missing_limit(client):
    async def run():
        queries = [
            (DenseVector([1, 2, 3]), Similarity.COSINE),
        ]
        with pytest.raises(ValueError):
            await client.batch_search(queries=queries)

    asyncio.run(run())


def test_async_batch_search_invalid_format(client):
    async def run():
        queries = ["invalid"]
        with pytest.raises(TypeError):
            await client.batch_search(queries=queries)

    asyncio.run(run())


# Close / Context Manager


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
