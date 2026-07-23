import os
from dataclasses import dataclass
from vortexdb.exceptions import ConfigurationError

DEFAULT_GRPC_HOST = "localhost"
DEFAULT_GRPC_PORT = 50051
DEFAULT_TIMEOUT = 5.0


@dataclass(frozen=True)
class VortexDBConfig:
    """Configuration for the VortexDB Python client"""

    grpc_url: str | None = None
    api_key: str | None = None
    timeout: float = DEFAULT_TIMEOUT

    @staticmethod
    def from_env(
        *,
        grpc_url: str | None = None,
        api_key: str | None = None,
        timeout: float | None = None,
    ) -> "VortexDBConfig":
        """Load configuration from explicit arguments with environment variable fallback"""

        resolved_grpc_url = (
            grpc_url
            or os.getenv("VORTEXDB_GRPC_URL")
            or f"{DEFAULT_GRPC_HOST}:{DEFAULT_GRPC_PORT}"
        )

        resolved_api_key = api_key or os.getenv("VORTEXDB_API_KEY")
        if not resolved_api_key:
            raise ConfigurationError(
                "VortexDB API key is required. "
                "Provide api_key argument or set VORTEXDB_API_KEY."
            )

        resolved_timeout = (
            timeout
            if timeout is not None
            else float(os.getenv("VORTEXDB_TIMEOUT", DEFAULT_TIMEOUT))
        )

        return VortexDBConfig(
            grpc_url=resolved_grpc_url,
            api_key=resolved_api_key,
            timeout=resolved_timeout,
        )
