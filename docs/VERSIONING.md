# Versioning

The root Cargo workspace version is the native application version authority.
During migration, synchronize package.json, package-lock.json, the Tauri Cargo
manifest/lockfile and configuration, and all three root Cargo package lock entries.
Run `python3 scripts/check_versions.py` to detect mismatches without downloading
dependencies. Dependency package versions are independent.

Version 0.2.0 is an unreleased native preview with additive app-window functionality.
It follows the existing v0.1.0 through v0.1.5 tag sequence; the older manifests had
lagged at 0.1.1/0.1.2. The UI migration does not reset version or release history.
SemVer minor changes during 0.x may include documented compatibility changes.

Historical changelog entries and immutable source snapshots keep their original
versions. In particular, the Arch recipe still pins commit 08414ac and version
0.1.2. It must be updated to a reviewed, fetchable 0.2.0 source commit and new
checksums before producing a 0.2.0 package. Do not relabel old source as 0.2.0.

Development integration updates dev. Publishing requires separate authorization,
release validation, a matching version tag, and verified artifacts. No release is
implied by an Unreleased changelog heading. Existing Tauri release automation is
not a native 0.2.0 publisher and must not be triggered for this integration.
