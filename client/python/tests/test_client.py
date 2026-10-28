import pytest
from unittest.mock import Mock

from vortexdb.client import VortexDB
from vortexdb.connection import GRPCConnection
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


# Fixtures for a mock connection and client layer


@pytest.fixture
def mock_connection(monkeypatch):
    """
    Replace GRPCConnection with a mock instance.
    """
    conn = Mock(spec=GRPCConnection)
    monkeypatch.setattr("vortexdb.client.GRPCConnection", lambda _: conn)
    return conn


@pytest.fixture
def client(mock_connection):
    return VortexDB(
        grpc_url="localhost:50051",
        api_key="secret",
    )


# Insert


def test_insert_dense_point(client, mock_connection):
    mock_connection.call.return_value = pb.PointID(id=pb.UUID(value="point-123"))

    point_id = client.insert(
        dense_vector=DenseVector([1, 2, 3]),
        payload=Payload.text("hello"),
    )

    assert point_id == "point-123"
    request = mock_connection.call.call_args.args[1]
    assert list(request.dense_vector.values) == [1.0, 2.0, 3.0]
    assert not request.HasField("sparse_vector")
    assert not request.HasField("id")


def test_insert_sparse_point_with_id(client, mock_connection):
    mock_connection.call.return_value = pb.PointID(id=pb.UUID(value="point-1"))

    client.insert(
        sparse_vector=SparseVector([1, 5], [0.5, 2.0]),
        point_id="point-1",
    )

    request = mock_connection.call.call_args.args[1]
    assert request.id.id.value == "point-1"
    assert list(request.sparse_vector.token_ids) == [1, 5]
    assert not request.HasField("dense_vector")


def test_insert_rejects_invalid_vector(client):
    with pytest.raises(TypeError):
        client.insert(
            dense_vector=[1, 2, 3],  # not DenseVector
            payload=Payload.text("hello"),
        )


def test_insert_rejects_point_without_vectors(client):
    with pytest.raises(ValueError):
        client.insert(payload=Payload.text("hello"))


# Batch Insert


def test_batch_insert_success(client, mock_connection):
    mock_connection.call.return_value = pb.InsertPointsBatchResponse(
        ids=[pb.PointID(id=pb.UUID(value="p1")), pb.PointID(id=pb.UUID(value="p2"))]
    )
    points = [
        PointInput(dense_vector=DenseVector([1, 2, 3]), payload=Payload.text("a")),
        PointInput(sparse_vector=SparseVector([2], [1.0])),
    ]

    result = client.batch_insert(points=points)

    assert result == ["p1", "p2"]
    request = mock_connection.call.call_args.args[1]
    assert len(request.points) == 2


def test_batch_insert_invalid_points_type(client):
    with pytest.raises(TypeError):
        client.batch_insert(points="not-a-list")


def test_batch_insert_rejects_non_point_input(client):
    with pytest.raises(TypeError):
        client.batch_insert(points=[(DenseVector([1, 2, 3]), Payload.text("a"))])


def test_batch_insert_rejects_empty(client):
    with pytest.raises(ValueError):
        client.batch_insert(points=[])


# Get


def test_get_point_success(client, mock_connection):
    mock_connection.call.return_value = pb.Point(
        id=pb.PointID(id=pb.UUID(value="point-123")),
        sparse_vector=pb.SparseVector(token_ids=[1, 4], weights=[0.5, 2.0]),
        payload=pb.Payload(content_type=pb.Text, content="hello"),
    )

    point = client.get(point_id="point-123")

    assert isinstance(point, Point)
    assert point.id == "point-123"
    assert point.dense_vector is None
    assert point.sparse_vector == SparseVector([1, 4], [0.5, 2.0])
    assert point.payload.content == "hello"


def test_get_point_not_found(client, mock_connection):
    mock_connection.call.return_value = None

    result = client.get(point_id="missing")

    assert result is None


# Delete


def test_delete_success(client, mock_connection):
    mock_connection.call.return_value = None

    client.delete(point_id="point-123")

    mock_connection.call.assert_called_once()


# Search


def test_dense_search_success(client, mock_connection):
    mock_connection.call.return_value = pb.SearchResponse(
        hits=[
            pb.SearchHit(id=pb.PointID(id=pb.UUID(value="p1")), score=0.25),
            pb.SearchHit(id=pb.PointID(id=pb.UUID(value="p2")), score=0.5),
        ]
    )
    query = SearchQuery(
        limit=2, query=DenseQuery(DenseVector([1, 2, 3]), Similarity.COSINE, ef=128)
    )

    results = client.search(query=query)

    assert results == [SearchHit("p1", 0.25), SearchHit("p2", 0.5)]
    request = mock_connection.call.call_args.args[1]
    assert request.limit == 2
    assert request.dense.ef == 128


def test_sparse_search_success(client, mock_connection):
    mock_connection.call.return_value = pb.SearchResponse(
        hits=[pb.SearchHit(id=pb.PointID(id=pb.UUID(value="p1")), score=2.0)]
    )

    results = client.search(query=SearchQuery(limit=5, query=SparseVector([4], [1.0])))

    assert results == [SearchHit("p1", 2.0)]
    request = mock_connection.call.call_args.args[1]
    assert request.WhichOneof("query") == "sparse"


def test_search_rejects_non_search_query(client):
    with pytest.raises(TypeError):
        client.search(query=DenseVector([1, 2, 3]))


# Batch Search


def test_batch_search_mixes_dense_and_sparse(client, mock_connection):
    mock_connection.call.return_value = pb.SearchBatchResponse(
        results=[
            pb.SearchResponse(
                hits=[pb.SearchHit(id=pb.PointID(id=pb.UUID(value="p1")), score=0.5)]
            ),
            pb.SearchResponse(),
        ]
    )
    queries = [
        SearchQuery(2, DenseQuery(DenseVector([1, 2, 3]), Similarity.EUCLIDEAN)),
        SearchQuery(1, SparseVector([7], [1.0])),
    ]

    result = client.batch_search(queries=queries)

    assert result == [[SearchHit("p1", 0.5)], []]
    request = mock_connection.call.call_args.args[1]
    assert [q.WhichOneof("query") for q in request.queries] == ["dense", "sparse"]


def test_batch_search_rejects_invalid_format(client):
    with pytest.raises(TypeError):
        client.batch_search(queries=["invalid"])


def test_batch_search_rejects_empty(client):
    with pytest.raises(ValueError):
        client.batch_search(queries=[])


# Close


def test_close_closes_connection(client, mock_connection):
    client.close()
    mock_connection.close.assert_called_once()


def test_context_manager_closes_connection(monkeypatch):
    conn = Mock(spec=GRPCConnection)
    monkeypatch.setattr("vortexdb.client.GRPCConnection", lambda _: conn)

    with VortexDB(
        grpc_url="localhost:50051",
        api_key="secret",
    ) as db:
        assert db is not None

    conn.close.assert_called_once()
