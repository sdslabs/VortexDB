import pytest

from vortexdb.models import (
    DenseQuery,
    DenseVector,
    Payload,
    Point,
    PointInput,
    SearchQuery,
    Similarity,
    ContentType,
    SparseVector,
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


def test_sparse_vector_valid():
    v = SparseVector((1, 4), (1, 0.5))
    assert v.token_ids == [1, 4]
    assert v.weights == [1.0, 0.5]
    assert SparseVector([1], [0]).weights == [0.0]


@pytest.mark.parametrize(
    "token_ids, weights",
    [
        ([], []),
        ([1, 2], [1.0]),
        ([2, 1], [1.0, 1.0]),
        ([1, 1], [1.0, 1.0]),
        ([1], [float("inf")]),
        ([1, 2], [1.0, -0.5]),
    ],
)
def test_sparse_vector_rejects_invalid_values(token_ids, weights):
    with pytest.raises(ValueError):
        SparseVector(token_ids, weights)


@pytest.mark.parametrize(
    "token_ids, weights",
    [([1.5], [1.0]), ([-1], [1.0]), ([1], ["a"])],
)
def test_sparse_vector_rejects_invalid_types(token_ids, weights):
    with pytest.raises(TypeError):
        SparseVector(token_ids, weights)


def test_point_input_requires_a_vector():
    with pytest.raises(ValueError):
        PointInput(payload=Payload.text("hello"))


def test_point_input_to_proto_leaves_missing_fields_unset():
    proto = PointInput(sparse_vector=SparseVector([3], [1.0])).to_proto()
    assert list(proto.sparse_vector.token_ids) == [3]
    assert not proto.HasField("dense_vector")
    assert not proto.HasField("payload")
    assert not proto.HasField("id")


def test_dense_search_query_to_proto():
    query = SearchQuery(3, DenseQuery(DenseVector([1, 2]), Similarity.COSINE))
    proto = query.to_proto()
    assert proto.limit == 3
    assert proto.WhichOneof("query") == "dense"
    assert proto.dense.similarity == vector_db_pb2.Cosine
    assert not proto.dense.HasField("ef")


def test_sparse_search_query_to_proto():
    proto = SearchQuery(3, SparseVector([1, 2], [0.5, 1.0])).to_proto()
    assert proto.WhichOneof("query") == "sparse"
    assert list(proto.sparse.weights) == [0.5, 1.0]


def test_search_query_rejects_raw_vector():
    with pytest.raises(TypeError):
        SearchQuery(3, DenseVector([1, 2]))


# Point Test


def test_point_from_proto():
    proto = vector_db_pb2.Point(
        id=vector_db_pb2.PointID(id=vector_db_pb2.UUID(value="point-123")),
        dense_vector=vector_db_pb2.DenseVector(values=[1, 2, 3]),
        sparse_vector=vector_db_pb2.SparseVector(token_ids=[2], weights=[0.5]),
        payload=vector_db_pb2.Payload(content_type=vector_db_pb2.Text, content="hello"),
    )

    point = Point.from_proto(proto)

    assert point.id == "point-123"
    assert point.dense_vector.values == [1.0, 2.0, 3.0]
    assert point.sparse_vector == SparseVector([2], [0.5])
    assert point.payload.content_type == ContentType.TEXT
    assert point.payload.content == "hello"


def test_point_from_proto_with_missing_fields():
    proto = vector_db_pb2.Point(
        id=vector_db_pb2.PointID(id=vector_db_pb2.UUID(value="p1")),
        dense_vector=vector_db_pb2.DenseVector(values=[1, 2, 3]),
    )

    point = Point.from_proto(proto)
    assert point.sparse_vector is None
    assert point.payload is None
