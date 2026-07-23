from typing import Any

import grpc

from vortexdb.exceptions import (
    AuthenticationError,
    InternalServerError,
    InvalidArgumentError,
    NotFoundError,
    ServiceUnavailableError,
    TimeoutError,
    VortexDBError,
)


def build_auth_metadata(api_key: str) -> tuple[tuple[str, str], ...]:
    return (("authorization", f"Bearer {api_key}"),)


def map_grpc_error(error: Any) -> VortexDBError:
    code = error.code()

    if code == grpc.StatusCode.UNAUTHENTICATED:
        return AuthenticationError(error.details())

    if code == grpc.StatusCode.NOT_FOUND:
        return NotFoundError(error.details())

    if code == grpc.StatusCode.INVALID_ARGUMENT:
        return InvalidArgumentError(error.details())

    if code == grpc.StatusCode.DEADLINE_EXCEEDED:
        return TimeoutError(error.details())

    if code == grpc.StatusCode.UNAVAILABLE:
        return ServiceUnavailableError(error.details())

    return InternalServerError(error.details())
