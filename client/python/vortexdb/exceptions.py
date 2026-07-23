class VortexDBError(Exception):
    """Base exception for all VortexDB client errors."""


class AuthenticationError(VortexDBError):
    """Authentication failed (invalid or missing API key)"""


class NotFoundError(VortexDBError):
    """Could not find requested resource"""


class InvalidArgumentError(VortexDBError):
    """Invalid input was provided"""


class TimeoutError(VortexDBError):
    """Request timed out while communicating with the server"""


class ServiceUnavailableError(VortexDBError):
    """The server is unavailable or unreachable"""


class InternalServerError(VortexDBError):
    """Internal error in the server"""


class ConfigurationError(VortexDBError):
    """Invalid or missing client configuration."""
