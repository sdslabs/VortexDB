import asyncio
from unittest.mock import AsyncMock, Mock

import pytest

from vortexdb.async_client import AsyncVortexDB
from vortexdb.async_connection import AsyncGRPCConnection
from vortexdb.grpc import vector_db_pb2 as pb
from vortexdb.models import (
    DenseQuery,
    DenseVector,
    Payload,
    Point,
    PointInput,
    SearchHit,
    SearchQuery,
    Similarity,
    SparseVector,
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


def test_async_insert_dense_point(client, mock_connection):
    async def run():
        mock_connection.call.return_value = pb.PointID(id=pb.UUID(value="point-123"))

        point_id = await client.insert(
            dense_vector=DenseVector([1, 2, 3]),
            payload=Payload.text("hello"),
        )

        assert point_id == "point-123"
        request = mock_connection.call.call_args.args[1]
        assert list(request.dense_vector.values) == [1.0, 2.0, 3.0]
        assert not request.HasField("sparse_vector")
        assert not request.HasField("id")

    asyncio.run(run())


def test_async_insert_sparse_point_with_id(client, mock_connection):
    async def run():
        mock_connection.call.return_value = pb.PointID(id=pb.UUID(value="point-1"))

        await client.insert(
            sparse_vector=SparseVector([1, 5], [0.5, 2.0]),
            point_id="point-1",
        )

        request = mock_connection.call.call_args.args[1]
        assert request.id.id.value == "point-1"
        assert list(request.sparse_vector.token_ids) == [1, 5]
        assert not request.HasField("dense_vector")

    asyncio.run(run())


def test_async_insert_rejects_invalid_vector(client):
    async def run():
        with pytest.raises(TypeError):
            await client.insert(
                dense_vector=[1, 2, 3],  # not DenseVector
                payload=Payload.text("hello"),
            )

    asyncio.run(run())


def test_async_insert_rejects_point_without_vectors(client):
    async def run():
        with pytest.raises(ValueError):
            await client.insert(payload=Payload.text("hello"))

    asyncio.run(run())


# Batch Insert


def test_async_batch_insert_success(client, mock_connection):
    async def run():
        mock_connection.call.return_value = pb.InsertPointsBatchResponse(
            ids=[pb.PointID(id=pb.UUID(value="p1")), pb.PointID(id=pb.UUID(value="p2"))]
        )
        points = [
            PointInput(dense_vector=DenseVector([1, 2, 3]), payload=Payload.text("a")),
            PointInput(sparse_vector=SparseVector([2], [1.0])),
        ]

        result = await client.batch_insert(points=points)

        assert result == ["p1", "p2"]
        request = mock_connection.call.call_args.args[1]
        assert len(request.points) == 2

    asyncio.run(run())


def test_async_batch_insert_invalid_points_type(client):
    async def run():
        with pytest.raises(TypeError):
            await client.batch_insert(points="not-a-list")

    asyncio.run(run())


def test_async_batch_insert_rejects_non_point_input(client):
    async def run():
        with pytest.raises(TypeError):
            await client.batch_insert(
                points=[(DenseVector([1, 2, 3]), Payload.text("a"))]
            )

    asyncio.run(run())


def test_async_batch_insert_rejects_empty(client):
    async def run():
        with pytest.raises(ValueError):
            await client.batch_insert(points=[])

    asyncio.run(run())


# Get


def test_async_get_point_success(client, mock_connection):
    async def run():
        mock_connection.call.return_value = pb.Point(
            id=pb.PointID(id=pb.UUID(value="point-123")),
            sparse_vector=pb.SparseVector(token_ids=[1, 4], weights=[0.5, 2.0]),
            payload=pb.Payload(content_type=pb.Text, content="hello"),
        )

        point = await client.get(point_id="point-123")

        assert isinstance(point, Point)
        assert point.id == "point-123"
        assert point.dense_vector is None
        assert point.sparse_vector == SparseVector([1, 4], [0.5, 2.0])
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

        mock_connection.call.assert_called_once()

    asyncio.run(run())


# Search


def test_async_dense_search_success(client, mock_connection):
    async def run():
        mock_connection.call.return_value = pb.SearchResponse(
            hits=[
                pb.SearchHit(id=pb.PointID(id=pb.UUID(value="p1")), score=0.25),
                pb.SearchHit(id=pb.PointID(id=pb.UUID(value="p2")), score=0.5),
            ]
        )
        query = SearchQuery(
            limit=2, query=DenseQuery(DenseVector([1, 2, 3]), Similarity.COSINE, ef=128)
        )

        results = await client.search(query=query)

        assert results == [SearchHit("p1", 0.25), SearchHit("p2", 0.5)]
        request = mock_connection.call.call_args.args[1]
        assert request.limit == 2
        assert request.dense.ef == 128

    asyncio.run(run())


def test_async_sparse_search_success(client, mock_connection):
    async def run():
        mock_connection.call.return_value = pb.SearchResponse(
            hits=[pb.SearchHit(id=pb.PointID(id=pb.UUID(value="p1")), score=2.0)]
        )

        results = await client.search(
            query=SearchQuery(limit=5, query=SparseVector([4], [1.0]))
        )

        assert results == [SearchHit("p1", 2.0)]
        request = mock_connection.call.call_args.args[1]
        assert request.WhichOneof("query") == "sparse"

    asyncio.run(run())


def test_async_search_rejects_non_search_query(client):
    async def run():
        with pytest.raises(TypeError):
            await client.search(query=DenseVector([1, 2, 3]))

    asyncio.run(run())


# Batch Search


def test_async_batch_search_mixes_dense_and_sparse(client, mock_connection):
    async def run():
        mock_connection.call.return_value = pb.SearchBatchResponse(
            results=[
                pb.SearchResponse(
                    hits=[
                        pb.SearchHit(id=pb.PointID(id=pb.UUID(value="p1")), score=0.5)
                    ]
                ),
                pb.SearchResponse(),
            ]
        )
        queries = [
            SearchQuery(2, DenseQuery(DenseVector([1, 2, 3]), Similarity.EUCLIDEAN)),
            SearchQuery(1, SparseVector([7], [1.0])),
        ]

        result = await client.batch_search(queries=queries)

        assert result == [[SearchHit("p1", 0.5)], []]
        request = mock_connection.call.call_args.args[1]
        assert [q.WhichOneof("query") for q in request.queries] == ["dense", "sparse"]

    asyncio.run(run())


def test_async_batch_search_rejects_invalid_format(client):
    async def run():
        with pytest.raises(TypeError):
            await client.batch_search(queries=["invalid"])

    asyncio.run(run())


def test_async_batch_search_rejects_empty(client):
    async def run():
        with pytest.raises(ValueError):
            await client.batch_search(queries=[])

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
