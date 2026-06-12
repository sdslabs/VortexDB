import pytest
from unittest.mock import Mock

from vortexdb.client import VortexDB
from vortexdb.connection import GRPCConnection
from vortexdb.models import DenseVector, Payload, Similarity, ContentType, Point
from vortexdb.exceptions import InvalidArgumentError



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


def test_insert_batch_success(client, mock_connection):
    mock_connection.call.return_value = Mock(
        ids=[
            Mock(id=Mock(value="p1")),
            Mock(id=Mock(value="p2")),
        ]
    )

    point_ids = client.insert_batch(
        points=[
            (DenseVector([1, 2, 3]), Payload.text("hello")),
            (DenseVector([4, 5, 6]), Payload.text("world")),
        ]
    )

    assert point_ids == ["p1", "p2"]


def test_insert_batch_rejects_invalid_vector(client):
    with pytest.raises(TypeError):
        client.insert_batch(
            points=[
                (DenseVector([1, 2, 3]), Payload.text("hello")),
                ([4, 5, 6], Payload.text("world")),
            ]
        )


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


def test_search_invalid_vector(client):
    with pytest.raises(TypeError):
        client.search(
            vector=[1, 2, 3],
            similarity=Similarity.COSINE,
            limit=2,
        )


def test_search_batch_success(client, mock_connection):
    mock_connection.call.return_value = Mock(
        results=[
            Mock(
                result_point_ids=[
                    Mock(id=Mock(value="p1")),
                    Mock(id=Mock(value="p2")),
                ]
            ),
            Mock(
                result_point_ids=[
                    Mock(id=Mock(value="p3")),
                ]
            ),
        ]
    )

    results = client.search_batch(
        queries=[
            (DenseVector([1, 2, 3]), Similarity.COSINE, 2),
            (DenseVector([4, 5, 6]), Similarity.COSINE, 1),
        ]
    )

    assert results == [["p1", "p2"], ["p3"]]


def test_search_batch_rejects_invalid_vector(client):
    with pytest.raises(TypeError):
        client.search_batch(
            queries=[
                (DenseVector([1, 2, 3]), Similarity.COSINE, 2),
                ([4, 5, 6], Similarity.COSINE, 1),
            ]
        )


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
