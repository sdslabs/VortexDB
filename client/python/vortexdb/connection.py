import grpc
from typing import Any, Callable

from vortexdb._grpc_common import build_auth_metadata, map_grpc_error
from vortexdb.config import VortexDBConfig

from vortexdb.grpc.vector_db_pb2_grpc import VectorDBStub


class GRPCConnection:
    """ gRPC connection wrapper for VortexDB"""

    def __init__(self, config: VortexDBConfig):
        self._config = config
        self._channel = grpc.insecure_channel(config.grpc_url)
        self._stub = VectorDBStub(self._channel)
        # Because this is required in every request
        self._metadata = build_auth_metadata(config.api_key)

    @property
    def stub(self) -> VectorDBStub:
        return self._stub

    def call(
        self,
        rpc: Callable[..., Any],
        request: Any,
    ) -> Any:
        """ Execute a gRPC call with standard error handling """
        try:
            return rpc(
                request,
                timeout=self._config.timeout,
                metadata=self._metadata,
            )

        except grpc.RpcError as e:
            raise map_grpc_error(e) from e

    def close(self) -> None:
        """ Close the underlying gRPC channel """
        self._channel.close()
