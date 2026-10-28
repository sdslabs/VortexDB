import math
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
class SparseVector:
    token_ids: List[int]
    weights: List[float]

    def __post_init__(self):
        if not isinstance(self.token_ids, (list, tuple)) or not isinstance(
            self.weights, (list, tuple)
        ):
            raise TypeError("SparseVector expects lists of token_ids and weights")

        if len(self.token_ids) != len(self.weights):
            raise ValueError(
                "SparseVector token_ids and weights must have equal length"
            )

        if not self.token_ids:
            raise ValueError("SparseVector cannot be empty")

        for t in self.token_ids:
            if not isinstance(t, int) or isinstance(t, bool) or t < 0:
                raise TypeError("SparseVector token_ids must be non-negative integers")

        for w in self.weights:
            if not isinstance(w, (int, float)):
                raise TypeError("SparseVector weights must be numeric (int or float)")
            if not math.isfinite(w):
                raise ValueError("SparseVector weights must be finite")
            if w < 0:
                raise ValueError("SparseVector weights must not be negative")

        if any(a >= b for a, b in zip(self.token_ids, self.token_ids[1:])):
            raise ValueError("SparseVector token_ids must be strictly increasing")

        object.__setattr__(self, "token_ids", list(self.token_ids))
        object.__setattr__(self, "weights", [float(w) for w in self.weights])

    def to_proto(self) -> vector_db_pb2.SparseVector:
        return vector_db_pb2.SparseVector(
            token_ids=self.token_ids, weights=self.weights
        )


@dataclass(frozen=True)
class PointInput:
    dense_vector: DenseVector | None = None
    sparse_vector: SparseVector | None = None
    payload: Payload | None = None
    point_id: str | None = None

    def __post_init__(self):
        if self.dense_vector is None and self.sparse_vector is None:
            raise ValueError("PointInput needs a dense_vector, a sparse_vector or both")

        if self.dense_vector is not None and not isinstance(
            self.dense_vector, DenseVector
        ):
            raise TypeError(
                "dense_vector must be a DenseVector. Use: DenseVector([1.0, 2.0, 3.0])"
            )

        if self.sparse_vector is not None and not isinstance(
            self.sparse_vector, SparseVector
        ):
            raise TypeError("sparse_vector must be a SparseVector")

        if self.payload is not None and not isinstance(self.payload, Payload):
            raise TypeError("payload must be a Payload")

    def to_proto(self) -> vector_db_pb2.PointInput:
        return vector_db_pb2.PointInput(
            id=(
                vector_db_pb2.PointID(id=vector_db_pb2.UUID(value=self.point_id))
                if self.point_id is not None
                else None
            ),
            dense_vector=self.dense_vector.to_proto() if self.dense_vector else None,
            sparse_vector=self.sparse_vector.to_proto() if self.sparse_vector else None,
            payload=self.payload.to_proto() if self.payload else None,
        )


@dataclass(frozen=True)
class Point:
    id: str
    dense_vector: DenseVector | None
    sparse_vector: SparseVector | None
    payload: Payload | None

    @staticmethod
    def from_proto(proto: vector_db_pb2.Point) -> "Point":
        dense_vector = None
        if proto.HasField("dense_vector"):
            dense_vector = DenseVector(list(proto.dense_vector.values))

        sparse_vector = None
        if proto.HasField("sparse_vector"):
            sparse_vector = SparseVector(
                list(proto.sparse_vector.token_ids),
                list(proto.sparse_vector.weights),
            )

        payload = None
        if proto.HasField("payload"):
            payload = Payload(
                content_type=ContentType.from_proto(proto.payload.content_type),
                content=proto.payload.content,
            )

        return Point(
            id=proto.id.id.value,
            dense_vector=dense_vector,
            sparse_vector=sparse_vector,
            payload=payload,
        )

    def pretty(self) -> str:
        dense = self.dense_vector.values if self.dense_vector else None
        sparse = (
            dict(zip(self.sparse_vector.token_ids, self.sparse_vector.weights))
            if self.sparse_vector
            else None
        )
        payload_type = self.payload.content_type.name if self.payload else None
        payload = self.payload.content if self.payload else None
        return (
            f"\nPoint:\n id = {self.id},\n"
            f" dense_vector = {dense},\n"
            f" sparse_vector = {sparse},\n"
            f" payload_type = {payload_type},\n"
            f" payload = {payload!r}"
        )


@dataclass(frozen=True)
class DenseQuery:
    vector: DenseVector
    similarity: Similarity
    ef: int | None = None

    def __post_init__(self):
        if not isinstance(self.vector, DenseVector):
            raise TypeError(
                "vector must be a DenseVector. Use: DenseVector([1.0, 2.0, 3.0])"
            )
        if not isinstance(self.similarity, Similarity):
            raise TypeError("similarity must be a Similarity enum")
        if self.ef is not None and not isinstance(self.ef, int):
            raise TypeError("ef must be an int")

    def to_proto(self) -> vector_db_pb2.DenseQuery:
        return vector_db_pb2.DenseQuery(
            vector=self.vector.to_proto(),
            similarity=self.similarity.to_proto(),
            ef=self.ef,
        )


# I added this because using tuples will get messy if we increase fields in a search query
@dataclass(frozen=True)
class SearchQuery:
    limit: int
    query: DenseQuery | SparseVector

    def __post_init__(self):
        if not isinstance(self.limit, int):
            raise TypeError("limit must be an int")
        if not isinstance(self.query, (DenseQuery, SparseVector)):
            raise TypeError("query must be a DenseQuery or a SparseVector")

    def to_proto(self) -> vector_db_pb2.SearchRequest:
        if isinstance(self.query, DenseQuery):
            return vector_db_pb2.SearchRequest(
                limit=self.limit, dense=self.query.to_proto()
            )
        return vector_db_pb2.SearchRequest(
            limit=self.limit, sparse=self.query.to_proto()
        )


@dataclass(frozen=True)
class SearchHit:
    id: str
    score: float

    @staticmethod
    def from_proto(proto: vector_db_pb2.SearchHit) -> "SearchHit":
        return SearchHit(id=proto.id.id.value, score=proto.score)
