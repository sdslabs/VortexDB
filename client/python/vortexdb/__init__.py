# vortexdb/__init__.py

from vortexdb.client import VortexDB
from vortexdb.models import (
    DenseVector,
    Payload,
    Point,
    Similarity,
    SearchQuery,
    to_dense_vectors,
)
from vortexdb.exceptions import (
    VortexDBError,
    AuthenticationError,
    NotFoundError,
    InvalidArgumentError,
    TimeoutError,
    ServiceUnavailableError,
    InternalServerError,
)

__all__ = [
    "VortexDB",
    "DenseVector",
    "Payload",
    "Point",
    "Similarity",
    "SearchQuery",
    "VortexDBError",
    "AuthenticationError",
    "NotFoundError",
    "InvalidArgumentError",
    "TimeoutError",
    "ServiceUnavailableError",
    "InternalServerError",
]
