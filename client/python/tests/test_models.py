from dataclasses import FrozenInstanceError

import pytest

from vortexdb.grpc import vector_db_pb2
from vortexdb.models import (
    ContentType,
    DenseVector,
    Payload,
    Point,
    Similarity,
)

# DenseVector Tests


def test_dense_vector_valid():
    a = [1, 2.5, 3]
    v = DenseVector(a)
    assert v.values == [1.0, 2.5, 3.0]
    assert all(isinstance(value, float) for value in v.values)


def test_dense_vector_accepts_tuple():
    v = DenseVector((1, 2, 3))
    assert v.values == [1.0, 2.0, 3.0]


def test_dense_vector_accepts_numpy_array():
    numpy = pytest.importorskip("numpy")
    v = DenseVector(numpy.array([1, 2.5, 3], dtype=numpy.float32))
    assert v.values == [1.0, 2.5, 3.0]


def test_dense_vector_accepts_numpy_scalars_in_list():
    numpy = pytest.importorskip("numpy")
    v = DenseVector([numpy.float32(1.5), numpy.float32(2.5)])
    assert v.values == [1.5, 2.5]
    assert all(type(value) is float for value in v.values)


def test_dense_vector_accepts_pytorch_tensor():
    torch = pytest.importorskip("torch")
    v = DenseVector(torch.tensor([1, 2.5, 3]))
    assert v.values == [1.0, 2.5, 3.0]


def test_dense_vector_accepts_tensorflow_tensor():
    tensorflow = pytest.importorskip("tensorflow")
    v = DenseVector(tensorflow.constant([1.0, 2.5, 3.0]))
    assert v.values == [1.0, 2.5, 3.0]


def test_dense_vector_accepts_jax_array():
    jax_numpy = pytest.importorskip("jax.numpy")
    v = DenseVector(jax_numpy.array([1, 2.5, 3]))
    assert v.values == [1.0, 2.5, 3.0]


class _TensorLike:
    def tolist(self):
        return [1, 2.5, 3]


def test_dense_vector_accepts_tensor_with_tolist():
    v = DenseVector(_TensorLike())
    assert v.values == [1.0, 2.5, 3.0]


class _TensorShape:
    rank = 1


class _MaterializedTensor:
    def tolist(self):
        return [1, 2.5, 3]


class _TensorFlowLike:
    shape = _TensorShape()

    def numpy(self):
        return _MaterializedTensor()


def test_dense_vector_accepts_tensor_with_numpy_conversion():
    v = DenseVector(_TensorFlowLike())
    assert v.values == [1.0, 2.5, 3.0]


class _JaxLike:
    ndim = 1

    def __array__(self):
        numpy = pytest.importorskip("numpy")
        return numpy.array([1, 2.5, 3])


def test_dense_vector_accepts_tensor_with_array_protocol():
    v = DenseVector(_JaxLike())
    assert v.values == [1.0, 2.5, 3.0]


def test_dense_vector_rejects_empty():
    with pytest.raises(ValueError):
        DenseVector([])


def test_dense_vector_rejects_non_numeric():
    with pytest.raises(TypeError):
        DenseVector([1, "a", 3])


@pytest.mark.parametrize("values", [[[1, 2], [3, 4]], [[1, 2, 3]]])
def test_dense_vector_rejects_nested_sequences(values):
    with pytest.raises(ValueError, match="one-dimensional"):
        DenseVector(values)


def test_dense_vector_rejects_multidimensional_array():
    numpy = pytest.importorskip("numpy")
    with pytest.raises(ValueError, match="one-dimensional"):
        DenseVector(numpy.array([[1, 2], [3, 4]]))


def test_dense_vector_rejects_scalar_array():
    numpy = pytest.importorskip("numpy")
    with pytest.raises(ValueError, match="one-dimensional"):
        DenseVector(numpy.array(1.0))


class _MatrixLike:
    ndim = 2

    def tolist(self):
        return [[1, 2], [3, 4]]


def test_dense_vector_rejects_multidimensional_tensor_like_value():
    with pytest.raises(ValueError, match="one-dimensional"):
        DenseVector(_MatrixLike())


def test_dense_vector_is_frozen():
    v = DenseVector([1, 2, 3])
    with pytest.raises(FrozenInstanceError):
        v.values = [4, 5, 6]


def test_dense_vector_to_proto():
    v = DenseVector([1, 2, 3])
    proto = v.to_proto()
    assert list(proto.values) == [1.0, 2.0, 3.0]


def test_dense_vector_to_numpy():
    numpy = pytest.importorskip("numpy")
    result = DenseVector([1, 2.5, 3]).to_numpy()
    assert isinstance(result, numpy.ndarray)
    assert result.ndim == 1
    assert numpy.issubdtype(result.dtype, numpy.floating)
    assert result.tolist() == [1.0, 2.5, 3.0]


def test_dense_vector_to_torch():
    torch = pytest.importorskip("torch")
    result = DenseVector([1, 2.5, 3]).to_torch()
    assert isinstance(result, torch.Tensor)
    assert result.ndim == 1
    assert result.dtype.is_floating_point
    assert result.tolist() == [1.0, 2.5, 3.0]


def test_dense_vector_to_tensorflow():
    tensorflow = pytest.importorskip("tensorflow")
    result = DenseVector([1, 2.5, 3]).to_tensorflow()
    assert tensorflow.is_tensor(result)
    assert result.shape.rank == 1
    assert result.dtype.is_floating
    assert result.numpy().tolist() == [1.0, 2.5, 3.0]


def test_dense_vector_to_jax():
    jax_numpy = pytest.importorskip("jax.numpy")
    result = DenseVector([1, 2.5, 3]).to_jax()
    assert result.ndim == 1
    assert jax_numpy.issubdtype(result.dtype, jax_numpy.floating)
    assert result.tolist() == [1.0, 2.5, 3.0]


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
