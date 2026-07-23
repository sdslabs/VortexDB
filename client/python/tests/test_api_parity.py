# VortexDB and AsyncVortexDB should expose the same public methods
# with the same signatures
import inspect

from vortexdb.client import VortexDB
from vortexdb.async_client import AsyncVortexDB


def public_methods(cls):
    return {
        name: value
        for name, value in vars(cls).items()
        if callable(value) and not name.startswith("_")
    }


def test_sync_async_client_api_parity():
    sync_methods = public_methods(VortexDB)
    async_methods = public_methods(AsyncVortexDB)

    sync_names = set(sync_methods)
    async_names = set(async_methods)

    assert sync_names == async_names, (
        "Sync and async clients expose different methods. "
        f"Only sync: {sorted(sync_names - async_names)}. "
        f"Only async: {sorted(async_names - sync_names)}."
    )

    mismatches = [
        f"  {name}: sync{inspect.signature(sync_methods[name])}"
        f"  !=  async{inspect.signature(async_methods[name])}"
        for name in sync_names
        if inspect.signature(sync_methods[name])
        != inspect.signature(async_methods[name])
    ]

    assert not mismatches, (
        "Sync and async methods have mismatched signatures:\n" + "\n".join(mismatches)
    )
