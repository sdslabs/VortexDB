from dataclasses import dataclass
from enum import Enum
from typing import List
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


# TODO Extend support to other data types than lists or tuples (numpy arrays probably)
# TODO Further compatibility to allow conversions directly to numpy arrays (similar to .to_list())
@dataclass(frozen=True)
class DenseVector:
    values: List[float]

    def __post_init__(self):
        if not isinstance(self.values, (list, tuple)):
            raise TypeError("DenseVector expects a list or tuple of floats")

        if not self.values:
            raise ValueError("DenseVector cannot be empty")

        for v in self.values:
            if not isinstance(v, (int, float)):
                raise TypeError("DenseVector values must be numeric (int or float)")

        # force float normalization
        object.__setattr__(self, "values", [float(v) for v in self.values])

    def to_proto(self) -> vector_db_pb2.DenseVector:
        return vector_db_pb2.DenseVector(values=self.values)

    def to_list(self) -> list[float]:
        return list(self.values)


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
