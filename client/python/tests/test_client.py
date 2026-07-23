import pytest
from unittest.mock import Mock

from vortexdb.client import VortexDB
from vortexdb.connection import GRPCConnection
from vortexdb.models import DenseVector, Payload, Similarity, ContentType, Point
from vortexdb.models import SearchQuery


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


def test_insert_success(client, mock_connection):
    response = Mock()
    response.id = Mock()
    response.id.value = "point-123"

    mock_connection.call.return_value = response

    vector = DenseVector([1, 2, 3])
    payload = Payload.text("hello")

    point_id = client.insert(vector=vector, payload=payload)

    assert point_id == "point-123"


def test_insert_rejects_invalid_vector(client):
    with pytest.raises(TypeError):
        client.insert(
            vector=[1, 2, 3],  # not DenseVector
            payload=Payload.text("hello"),
        )


# Batch Insert


def test_batch_insert_success(client, mock_connection):
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
    result = client.batch_insert(items=items)
    assert result == ["p1", "p2"]


def test_batch_insert_invalid_items_type(client):
    with pytest.raises(TypeError):
        client.batch_insert(items="not-a-list")


def test_batch_insert_invalid_tuple_structure(client):
    items = [
        (DenseVector([1, 2, 3]),),  # only one element
    ]
    with pytest.raises(TypeError):
        client.batch_insert(items=items)


def test_batch_insert_invalid_vector(client):
    items = [
        ([1, 2, 3], Payload.text("a")),  # not DenseVector
    ]
    with pytest.raises(TypeError):
        client.batch_insert(items=items)


# Get


def test_get_point_success(client, mock_connection):
    proto_point = Mock()
    proto_point.id.id.value = "point-123"
    proto_point.vector.values = [1, 2, 3]
    proto_point.payload.content_type = ContentType.TEXT.to_proto()
    proto_point.payload.content = "hello"

    mock_connection.call.return_value = proto_point

    point = client.get(point_id="point-123")

    assert isinstance(point, Point)
    assert point.id == "point-123"
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


def test_search_success(client, mock_connection):
    mock_connection.call.return_value = Mock(
        result_point_ids=[
            Mock(id=Mock(value="p1")),
            Mock(id=Mock(value="p2")),
        ]
    )

    results = client.search(
        vector=DenseVector([1, 2, 3]),
        similarity=Similarity.COSINE,
        limit=2,
    )

    assert results == ["p1", "p2"]


def test_search_accepts_ef(client, mock_connection):
    mock_connection.call.return_value = Mock(result_point_ids=[])

    client.search(
        vector=DenseVector([1, 2, 3]),
        similarity=Similarity.COSINE,
        limit=2,
        ef=128,
    )

    request = mock_connection.call.call_args.args[1]
    assert request.ef == 128


def test_search_invalid_vector(client):
    with pytest.raises(TypeError):
        client.search(
            vector=[1, 2, 3],
            similarity=Similarity.COSINE,
            limit=2,
        )


def test_batch_search_accepts_ef(client, mock_connection):
    mock_connection.call.return_value = Mock(results=[])

    client.batch_search(
        queries=[
            (DenseVector([1, 2, 3]), Similarity.COSINE, 2),
            (DenseVector([4, 5, 6]), Similarity.COSINE, 1),
        ],
        ef=256,
    )

    request = mock_connection.call.call_args.args[1]
    assert [query.ef for query in request.queries] == [256, 256]


# Batch Search


def test_batch_search_full_tuple(client, mock_connection):
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
    result = client.batch_search(queries=queries)
    assert result == [["p1"], ["p2"]]


def test_batch_search_vectors_with_global_params(client, mock_connection):
    mock_connection.call.return_value = Mock(
        results=[
            Mock(result_point_ids=[Mock(id=Mock(value="p1"))]),
        ]
    )
    queries = [DenseVector([1, 2, 3])]
    result = client.batch_search(
        queries=queries,
        similarity=Similarity.MANHATTAN,
        limit=2,
    )
    assert result == [["p1"]]


def test_batch_search_vector_similarity_with_global_limit(client, mock_connection):
    mock_connection.call.return_value = Mock(
        results=[
            Mock(result_point_ids=[Mock(id=Mock(value="p1"))]),
        ]
    )
    queries = [
        (DenseVector([1, 2, 3]), Similarity.COSINE),
    ]
    result = client.batch_search(
        queries=queries,
        limit=2,
    )
    assert result == [["p1"]]


def test_batch_search_searchquery_objects(client, mock_connection):
    mock_connection.call.return_value = Mock(
        results=[
            Mock(result_point_ids=[Mock(id=Mock(value="p1"))]),
        ]
    )
    queries = [
        SearchQuery(DenseVector([1, 2, 3]), Similarity.COSINE, 2),
    ]
    result = client.batch_search(queries=queries)
    assert result == [["p1"]]


def test_batch_search_missing_globals_for_vector(client):
    queries = [DenseVector([1, 2, 3])]
    with pytest.raises(ValueError):
        client.batch_search(queries=queries)


def test_batch_search_missing_limit(client):
    queries = [
        (DenseVector([1, 2, 3]), Similarity.COSINE),
    ]
    with pytest.raises(ValueError):
        client.batch_search(queries=queries)


def test_batch_search_invalid_format(client):
    queries = ["invalid"]
    with pytest.raises(TypeError):
        client.batch_search(queries=queries)


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
