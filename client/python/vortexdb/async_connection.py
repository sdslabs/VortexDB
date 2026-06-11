from typing import Any, Callable

import grpc

from vortexdb._grpc_common import build_auth_metadata, map_grpc_error
from vortexdb.config import VortexDBConfig
from vortexdb.grpc.vector_db_pb2_grpc import VectorDBStub


class AsyncGRPCConnection:
    """Async gRPC connection wrapper for VortexDB."""

    def __init__(self, config: VortexDBConfig):
        self._config = config
        self._channel = grpc.aio.insecure_channel(config.grpc_url)
        self._stub = VectorDBStub(self._channel)
        self._metadata = build_auth_metadata(config.api_key)

    @property
    def stub(self) -> VectorDBStub:
        return self._stub

    async def call(
        self,
        rpc: Callable[..., Any],
        request: Any,
    ) -> Any:
        """Execute an async gRPC call with standard error handling."""
        try:
            return await rpc(
                request,
                timeout=self._config.timeout,
                metadata=self._metadata,
            )

        except grpc.aio.AioRpcError as e:
            raise map_grpc_error(e) from e

    async def close(self) -> None:
        """Close the underlying async gRPC channel."""
        await self._channel.close()
