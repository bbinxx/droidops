# Arch Linux application package

The Arch recipe is in `packaging/arch/PKGBUILD`. It builds the native egui
application from pinned public source, runs its tests, and produces a pacman
package. No React/Tauri frontend or Node dependency is built or installed.

## Build and install

Use Arch Linux x86_64 with an updated system. From the repository root,
install the build tools once, then build as your normal user:

```bash
sudo pacman -S --needed base-devel
cd packaging/arch
makepkg -si
```

Do not run makepkg or the application as root. `makepkg -si` builds the
package, runs tests, and requests pacman authorization for dependencies and
installation. The resulting `.pkg.tar.zst` file stays in `packaging/arch`.

makepkg resolves the declared dependencies with normal pacman authorization.
Cargo, librsvg, and desktop-file-utils are needed to build. Runtime dependencies
include android-tools, Linux graphics libraries, and xdg-desktop-portal. A desktop
portal backend is needed for file dialogs. scrcpy is optional for mirroring.

To inspect a package before installation:

```bash
makepkg
pacman -Qip ./droidops-0.1.2-1-x86_64.pkg.tar.zst
pacman -Qlp ./droidops-0.1.2-1-x86_64.pkg.tar.zst
namcap ./droidops-0.1.2-1-x86_64.pkg.tar.zst
sudo pacman -U ./droidops-0.1.2-1-x86_64.pkg.tar.zst
```

namcap is optional inspection tooling. The package has dynamically loaded graphics
and portal dependencies that static dependency analysis may report as unused;
review each warning instead of removing runtime dependencies automatically.

## Installed files

- `/usr/bin/droidops`
- `/usr/share/applications/DroidOps.desktop`
- `/usr/share/icons/hicolor/scalable/apps/io.github.bbinxx.droidops.svg`
- `/usr/share/icons/hicolor/256x256/apps/io.github.bbinxx.droidops.png`
- `/usr/share/licenses/droidops/LICENSE_STATUS`

Open DroidOps from the application menu or run `droidops`. The launcher does not
open a terminal and does not start privileged operations. Pacman owns installed
files and desktop/icon cache hooks. Uninstall with `sudo pacman -R droidops`.

## Reproducibility and scope

The current recipe pins commit `08414ac823609d544876bd59621bac31a9462f79`, validates
source/integration files with SHA-256, and preserves workspace version 0.1.2 as
development metadata. It does not publish a new upstream release or change tags.
Only Arch x86_64 is declared; no other platform build is implied.

Cargo fetches dependencies during prepare(), then builds/tests use frozen/offline
resolution. Default build concurrency is two jobs; override CARGO_BUILD_JOBS if
appropriate. Rust compilation uses normal Arch build flags and the project's
release profile. A local build is not a claim of bit-for-bit reproducibility.

The project has no confirmed license text. LicenseRef-Unknown and LICENSE_STATUS
describe that metadata gap without assigning new ownership or license terms.

See [the recipe guide](../../packaging/arch/README.md) for maintenance details and
[the native guide](NATIVE.md) for application acceptance checks. Package tests use
mock devices and disposable host fixtures; installation never modifies USB rules,
flashes devices, or runs device operations as root.

## Verify, upgrade, and remove

After installation:

```bash
pacman -Q droidops
pacman -Qkk droidops
droidops
```

The expected version for this recipe is `0.1.2-1`. The integrity check should
report zero altered files. If the application menu has not refreshed, reopen
the launcher or log in again; `droidops` starts the same installed executable.

For a packaging update, check out the reviewed packaging changes, rebuild in
`packaging/arch` with `makepkg -sf`, then install the resulting package with
`sudo pacman -U ./droidops-0.1.2-1-x86_64.pkg.tar.zst`. Adjust the filename when
the version changes. Rebuilding this pinned recipe alone does not fetch newer
application code; updating the source pin and checksums is a maintainer step.

Remove the application with:

```bash
sudo pacman -R droidops
```

## Recorded local validation

On 2026-11-15 the pinned source built on Arch Linux x86_64 with Rust 1.100.0.
All 22 release-profile workspace tests passed, source checksums and desktop-file
validation passed, and pacman installed version 0.1.2-1. The installed package
integrity check reported 17 entries and zero altered files. Static package lint
reported dynamic runtime dependency warnings described above. This does not
establish physical-device acceptance or support on other platforms.
