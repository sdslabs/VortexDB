from dataclasses import dataclass
from enum import Enum
from numbers import Real
from typing import Any

from vortexdb.grpc import vector_db_pb2

# I found this to be a good idea, because
# 1. readability
# 2. will help in HTTP client
# 3. transport conversion at the very end, won't break if proto enum changes


class Similarity(Enum):
    EUCLIDEAN = "euclidean"
    MANHATTAN = "manhattan"
    HAMMING = "hamming"
    COSINE = "cosine"

    def to_proto(self) -> int:
        return {
            Similarity.EUCLIDEAN: vector_db_pb2.Euclidean,
            Similarity.MANHATTAN: vector_db_pb2.Manhattan,
            Similarity.HAMMING: vector_db_pb2.Hamming,
            Similarity.COSINE: vector_db_pb2.Cosine,
        }[self]


class ContentType(Enum):
    TEXT = "text"
    IMAGE = "image"

    def to_proto(self) -> int:
        return {
            ContentType.TEXT: vector_db_pb2.Text,
            ContentType.IMAGE: vector_db_pb2.Image,
        }[self]

    @staticmethod
    def from_proto(value: int) -> "ContentType":
        return {
            vector_db_pb2.Text: ContentType.TEXT,
            vector_db_pb2.Image: ContentType.IMAGE,
        }[value]


@dataclass(frozen=True)
class DenseVector:
    values: list[float]

    def __post_init__(self):
        if isinstance(self.values, (list, tuple)):
            normalized_values = list(self.values)
        else:
            normalized_values = self._array_like_to_list(self.values)

        if not normalized_values:
            raise ValueError("DenseVector cannot be empty")

        if any(isinstance(value, (list, tuple)) for value in normalized_values):
            raise ValueError("DenseVector expects a one-dimensional vector")

        for v in normalized_values:
            if not isinstance(v, Real):
                raise TypeError("DenseVector values must be numeric (int or float)")

        object.__setattr__(self, "values", [float(v) for v in normalized_values])

    @staticmethod
    def _array_like_to_list(values: Any) -> list[Any]:
        shape = getattr(values, "shape", None)
        rank = getattr(values, "ndim", None)

        if rank is None and shape is not None:
            rank = getattr(shape, "rank", None)
            if rank is None:
                try:
                    rank = len(shape)
                except (TypeError, ValueError):
                    pass

        if rank is not None and rank != 1:
            raise ValueError("DenseVector expects a one-dimensional vector")

        to_list = getattr(values, "tolist", None)
        if callable(to_list):
            converted = to_list()
        else:
            to_numpy = getattr(values, "numpy", None)
            if callable(to_numpy):
                converted = to_numpy()
            else:
                to_array = getattr(values, "__array__", None)
                if not callable(to_array):
                    raise TypeError("DenseVector could not convert the array or tensor")
                converted = to_array()

            converted_to_list = getattr(converted, "tolist", None)
            if not callable(converted_to_list):
                raise TypeError("DenseVector could not convert the array or tensor")
            converted = converted_to_list()

        if not isinstance(converted, list):
            raise TypeError("DenseVector could not convert the array or tensor")
        return converted

    def to_proto(self) -> vector_db_pb2.DenseVector:
        return vector_db_pb2.DenseVector(values=self.values)

    def to_list(self) -> list[float]:
        return list(self.values)

    # Keep framework dependencies optional by importing only when conversion is requested.
    def to_numpy(self) -> Any:
        try:
            import numpy
        except ImportError as error:
            raise ImportError(
                "NumPy is required to convert DenseVector to an array"
            ) from error
        return numpy.asarray(self.values, dtype=float)

    def to_torch(self) -> Any:
        try:
            import torch
        except ImportError as error:
            raise ImportError(
                "PyTorch is required to convert DenseVector to a tensor"
            ) from error
        return torch.tensor(self.values)

    def to_tensorflow(self) -> Any:
        try:
            import tensorflow
        except ImportError as error:
            raise ImportError(
                "TensorFlow is required to convert DenseVector to a tensor"
            ) from error
        return tensorflow.convert_to_tensor(self.values)

    def to_jax(self) -> Any:
        try:
            import jax.numpy
        except ImportError as error:
            raise ImportError(
                "JAX is required to convert DenseVector to an array"
            ) from error
        return jax.numpy.asarray(self.values)


# & Helper Function for Batch of DenseVectors
def to_dense_vectors(arr):
    return [DenseVector(x) for x in arr]


@dataclass(frozen=True)
class Payload:
    content_type: ContentType
    content: str

    @staticmethod
    def text(content: str) -> "Payload":
        return Payload(ContentType.TEXT, content)

    @staticmethod
    def image(content: str) -> "Payload":
        return Payload(ContentType.IMAGE, content)

    def __post_init__(self):
        if not isinstance(self.content_type, ContentType):
            raise TypeError("content_type must be ContentType enum")

    def to_proto(self) -> vector_db_pb2.Payload:
        return vector_db_pb2.Payload(
            content_type=self.content_type.to_proto(),
            content=self.content,
        )


@dataclass(frozen=True)
class Point:
    id: str
    vector: DenseVector
    payload: Payload

    @staticmethod
    def from_proto(proto: vector_db_pb2.Point) -> "Point":
        payload = proto.payload
        if payload is None:
            payload_obj = Payload.text("")
        else:
            payload_obj = Payload(
                content_type=ContentType.from_proto(payload.content_type),
                content=payload.content,
            )

        return Point(
            id=proto.id.id.value,
            vector=DenseVector(list(proto.vector.values)),
            payload=payload_obj,
        )

    def pretty(self) -> str:
        return (
            f"\nPoint:\n id = {self.id},\n"
            f" vector_dim = {len(self.vector.values)},\n"
            f" vector = {self.vector},\n"
            f" payload_type = {self.payload.content_type.name},\n"
            f" payload = '{self.payload.content}'"
        )


# I added this because using tuples will get messy if we increase fields in a search query
@dataclass(frozen=True)
class SearchQuery:
    vector: DenseVector
    similarity: Similarity
    limit: int

    def to_proto(self) -> vector_db_pb2.SearchRequest:
        return vector_db_pb2.SearchRequest(
            query_vector=self.vector.to_proto(),
            similarity=self.similarity.to_proto(),
            limit=self.limit,
        )
