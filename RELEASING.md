# Releasing VortexDB

This document describes the release process. It is intended for maintainers with push access to `main`.

## Branch model

- `main` is the only long-lived branch.
- Direct pushes to `main` are disabled; all changes go through a PR.
- PR titles must follow [Conventional Commits](https://www.conventionalcommits.org/), enforced by the `semantic-pull-request` check.

## Releasing the server (Rust)

Merging a PR into `main` does not trigger a release. Releases are triggered by pushing a version tag.

1. Bump `version` under `[workspace.package]` in the root `Cargo.toml`, and merge via PR.
2. From `main`, tag and push:
   ```
   git tag v0.1.0
   git push origin v0.1.0
   ```
3. The tag push triggers `.github/workflows/release.yml`, which:
   - Verifies the tag matches the version in `Cargo.toml`.
   - Verifies the tagged commit is on `main`.
   - Builds and runs the test suite.
   - Creates the GitHub Release.

A failing test suite stops the workflow before the release step runs.

## Releasing the Python client

The Python client (`client/python/`) is versioned and released independently of the server, using its own tag prefix.

1. Bump `version` in `client/python/pyproject.toml`, and merge via PR.
2. Tag and push with the `python-v` prefix:
   ```
   git tag python-v0.2.0
   git push origin python-v0.2.0
   ```
3. The tag push triggers `.github/workflows/publish-python.yml`, which builds the package and publishes to PyPI via [Trusted Publishing](https://docs.pypi.org/trusted-publishers/) (OIDC, no stored API token).

### One-time PyPI setup

1. On PyPI: project page → **Publishing** → add a GitHub publisher with owner `sdslabs`, repo `vector-db`, workflow `publish-python.yml`, environment `pypi`.
2. On GitHub: Settings → Environments → create an environment named `pypi`, matching the `environment: pypi` value in the workflow.

## Secrets

Neither workflow requires manually configured secrets. `release.yml` uses the default `GITHUB_TOKEN`. `publish-python.yml` uses OIDC trusted publishing.

