import pytest

from vortexdb.models import (
    DenseVector,
    Payload,
    Point,
    Similarity,
    ContentType,
)

from vortexdb.grpc import vector_db_pb2

# DenseVector Tests


def test_dense_vector_valid():
    a = [1, 2.5, 3]
    v = DenseVector(a)
    assert v.values == [1.0, 2.5, 3.0]


def test_dense_vector_accepts_tuple():
    v = DenseVector((1, 2, 3))
    assert v.values == [1.0, 2.0, 3.0]


def test_dense_vector_rejects_empty():
    with pytest.raises(ValueError):
        DenseVector([])


def test_dense_vector_rejects_non_numeric():
    with pytest.raises(TypeError):
        DenseVector([1, "a", 3])


def test_dense_vector_is_frozen():
    v = DenseVector([1, 2, 3])
    with pytest.raises(Exception):
        v.values = [4, 5, 6]


def test_dense_vector_to_proto():
    v = DenseVector([1, 2, 3])
    proto = v.to_proto()
    assert list(proto.values) == [1.0, 2.0, 3.0]


# Similarity Test


def test_similarity_to_proto():
    assert Similarity.EUCLIDEAN.to_proto() == vector_db_pb2.Euclidean
    assert Similarity.MANHATTAN.to_proto() == vector_db_pb2.Manhattan
    assert Similarity.HAMMING.to_proto() == vector_db_pb2.Hamming
    assert Similarity.COSINE.to_proto() == vector_db_pb2.Cosine


# ContentType Tests


def test_content_type_to_proto():
    assert ContentType.TEXT.to_proto() == vector_db_pb2.Text
    assert ContentType.IMAGE.to_proto() == vector_db_pb2.Image


def test_content_type_from_proto():
    assert ContentType.from_proto(vector_db_pb2.Text) == ContentType.TEXT
    assert ContentType.from_proto(vector_db_pb2.Image) == ContentType.IMAGE


def test_content_type_from_proto_invalid():
    with pytest.raises(KeyError):
        ContentType.from_proto(100)


# Payload Tests


def test_payload_text_factory():
    p = Payload.text("hello")
    assert p.content_type == ContentType.TEXT
    assert p.content == "hello"


def test_payload_image_factory():
    p = Payload.image("img_data")
    assert p.content_type == ContentType.IMAGE
    assert p.content == "img_data"


def test_payload_to_proto():
    p = Payload.text("hello")
    proto = p.to_proto()
    assert proto.content == "hello"
    assert proto.content_type == vector_db_pb2.Text


def test_payload_rejects_invalid_content_type():
    with pytest.raises(TypeError):
        Payload("text", "hello")


# Point Test


def test_point_from_proto():
    proto = vector_db_pb2.Point(
        id=vector_db_pb2.PointID(id=vector_db_pb2.UUID(value="point-123")),
        vector=vector_db_pb2.DenseVector(values=[1, 2, 3]),
        payload=vector_db_pb2.Payload(content_type=vector_db_pb2.Text, content="hello"),
    )

    point = Point.from_proto(proto)

    assert point.id == "point-123"
    assert point.vector.values == [1.0, 2.0, 3.0]
    assert point.payload.content_type == ContentType.TEXT
    assert point.payload.content == "hello"


def test_point_from_proto_without_payload():
    proto = vector_db_pb2.Point(
        id=vector_db_pb2.PointID(id=vector_db_pb2.UUID(value="p1")),
        vector=vector_db_pb2.DenseVector(values=[1, 2, 3]),
        payload=None,
    )

    point = Point.from_proto(proto)
    assert point.payload.content == ""
