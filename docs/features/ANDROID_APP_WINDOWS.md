# Independent Android app windows

## Specification

Problem: one shared mirror cannot provide independent desktop app windows.
Goal: run compatible apps on the selected physical phone, each in a separate
scrcpy process and Android virtual display. No desktop Android runtime is used.
Scope: native Applications screen, named app inventory, configurable display,
DPI/FPS/audio/title, session monitoring, independent stop, and owned shutdown.
Desktop shortcuts and APK icon extraction are follow-up work; generic vector
icons are used when no application icon is available.

User experience: select a device, open Applications, choose Open as Desktop App.
Active App Windows lists device-bound sessions and Stop actions. Launch settings
apply to new windows. Force-stop needs explicit confirmation per launch.
Architecture: AppWindowManager in device-services owns bounded concurrent tasks;
the native worker transports commands and snapshots without blocking frames.
Safety: structured arguments, package validation, exact device targeting, no
automatic force-stop, bounded logs/startup time, and no device data in diagnostics.
Acceptance: multiple independent windows, separate stop/error states, no interference
with full-screen mirroring; failures never become Running based on spawn alone.
Test plan: fake runner concurrency/lifecycle/command/error tests; Rust/UI checks;
Arch real Spotify/Chrome/Gmail tests when installed and compatible. Windows/macOS
manual behavior must be recorded separately.

## Use

1. Install ADB and a scrcpy build with `--new-display` and `--start-app` support.
   On Arch these are android-tools and scrcpy. Authorize USB debugging, or connect
   an already configured Wi-Fi ADB device using normal platform-tools procedures.
2. Run the native app (`cargo run --locked`), explicitly select the phone, and open
   Applications. Inventory loads automatically for the selected filter.
3. Click **Open as Desktop App** beside an installed package. Labels are shown when
   scrcpy can retrieve them; otherwise the package ID is used. Android restrictions
   mean not every installed package has a launchable activity.
4. Open another app for another independent OS window. Each scrcpy window provides
   normal window-manager focus/minimize/maximize/close behavior and scrcpy mouse,
   keyboard, and clipboard defaults. Desktop grouping depends on the window manager.
5. Use **Active App Windows → Stop** to close one session. This section stays
   accessible when no device is selected. Closing DroidOps stops all its sessions.

Expand **Desktop app window settings** to set virtual resolution, desktop window
size, DPI, FPS, title override, audio, dynamic resizing, Android decorations, and
content preservation. Changes apply to new sessions. Live Android-resolution resizing requires custom mode and installed scrcpy
support for both `--flex-display` and `--render-fit`; otherwise window resizing
scales the fixed Android display. Fixed desktop dimensions are omitted in dynamic
mode because scrcpy rejects that combination. Apps may still refuse to reflow. Audio is off by
default; optional playback capture can include other apps and can be prohibited by
Android/app policy. Clipboard is shared with the phone, not isolated per window.

Force-stop is off by default and warns before each affected launch because it can
interrupt the same app on the phone. Existing task affinity can move a phone app to
its virtual display even without force-stop. The primary display is not mirrored
or switched off by this feature. Hardware resources limit concurrent encoders.

Normally removing a virtual display destroys its content. **Move content to phone
when closing** requests scrcpy's supported alternative and may bring tasks onto the
primary screen. Neither option preserves unsaved app state reliably. Keep the
phone connected. Protected/DRM content and Android security restrictions are never
bypassed. Linux desktop shortcuts remain a follow-up feature.

See [session architecture](../architecture/APP_WINDOWS.md) for ownership and limits.

## Validation

Ordinary checks require no connected device:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build --locked
cargo doc --workspace --no-deps --locked
python3 scripts/check_agent_infrastructure.py --self-test
python3 scripts/check_agent_infrastructure.py
git diff --check
```

An explicit hardware smoke check opens the supplied installed packages, verifies
virtual-display placement, stops the first independently, then stops and reaps all
owned sessions. Run only after selecting and authorizing the target phone:

```bash
cargo run -p droidops-device-services --example app_windows_smoke -- SERIAL PACKAGE_A PACKAGE_B PACKAGE_C
# Optionally include the existing full-screen mirror in the check:
# append --with-mirror after the package IDs
```

Manual acceptance also needs keyboard/mouse/clipboard, minimize/maximize/focus,
resize/reflow, audio if enabled, close via OS decoration, unplug/reconnect, and main
UI shutdown. Never use force-stop or private content for routine automated checks.

## Recorded evidence (2026-11-15)

On Arch Linux x86_64 with scrcpy 5.0.1 and a selected M2101K6P running Android 17,
the service smoke test verified Spotify, Chrome, and Gmail running simultaneously
on distinct non-primary virtual displays. Wayland's toplevel list showed three
separate app windows alongside the normal phone mirror. The primary display
retained its launcher. Stopping Spotify left Chrome and Gmail Running. The normal
mirror remained active; final shutdown stopped and reaped the owned test sessions.
The supplied screenshot also showed separate Gmail and Chrome windows after the
first session's deliberate stop. Personal screenshots/dumps are not fixtures.

An initial attempt failed; a subsequent service test passed. Testing the added
fixed window-size settings then exposed scrcpy's incompatibility between
`--flex-display` and `--window-width`/`--window-height`. The final command builder
uses these modes exclusively, and the corrected three-app-plus-mirror test passed.

Automated coverage includes command validation/capabilities, labels, exact package
and display matching, concurrent sessions, independent stop, rejected launch while
the window stays alive, startup cancellation, limits, disconnect reconciliation,
shutdown/reaping, UI rendering, and generation-bound force-stop confirmation.
Physical disconnect/reconnect, audio, clipboard/input, resize/minimize/maximize,
OS close, force-stop, and content-preservation behavior still need manual acceptance.
Windows/macOS builds and hardware acceptance were not run for this change.

Final local checks passed: rustfmt, Clippy with warnings denied across all targets,
34 workspace tests, native debug build, workspace rustdoc, 12 agent-checker
self-tests, agent infrastructure validation, and Git whitespace checks. The
workspace's existing serde dependency is reused for saved launch preferences; no
new crate was added to Cargo.lock.

## Phone resolution, resizing, and window identity

New launches default to **Match phone resolution and DPI at launch**. The worker
reads `wm size` and `wm density` from the selected phone without modifying either
setting. Android overrides take precedence over physical values. Explicit width,
height, and density are then passed to scrcpy, including in flex mode: an empty
`--new-display` has different defaults in some flex-display versions. Reading an
unrecognized display description fails visibly; disable matching to enter custom
values. The initial Android virtual display matches these dimensions, while the
host window may be fitted to the monitor. Phone mode keeps that Android resolution fixed and scales the whole frame as
the host window changes. Live Android-resolution resizing requires custom mode.

No host application can guarantee that every Android app supports live layout
changes. Disable phone matching and use **Live Android resolution (custom mode)** for
responsive apps, with DPI appropriate for a desktop-sized virtual display. Turn it off
for apps that restart, refuse to reflow, or render poorly: fixed mode keeps the
Android resolution and scales/letterboxes it inside a manually resizable window.
When supported, fixed mode removes the host aspect-ratio lock. Neither mode forces
Android developer settings or bypasses app policy. Restart the app window after
changing launch settings.

Window titles use the Android app label (package ID fallback), with an optional
custom title. Each child also receives SDL application-name and per-package
DroidOps application-ID hints, scoped to its own process. On the tested Arch
Wayland/SDL3 setup, compositor IDs changed from `scrcpy` to distinct
`io.github.bbinxx.droidops.app.PACKAGE` values. Desktop taskbar labels, grouping,
and icons still depend on the compositor and desktop metadata. Windows/macOS and
SDL2 identity behavior remain unverified. The executable remains scrcpy.

A custom egui video window is technically possible but is not part of this change.
It would require a scrcpy protocol client, video decoding, texture presentation,
and input/clipboard/audio forwarding. Reparenting a separate scrcpy window is not
a portable Wayland/Windows/macOS solution. Current app-named windows preserve
scrcpy's existing input and rendering implementation.

The updated defaults passed the three-app Arch smoke check and independent stop /
cleanup checks. Two added fake-runner tests cover targeted phone-display queries,
override precedence, custom-mode opt-out, child identity, and malformed output.
No broad claim that every installed Android application can resize is made.

References: [scrcpy virtual displays](https://github.com/Genymobile/scrcpy/blob/master/doc/virtual-display.md),
[SDL application identity](https://wiki.libsdl.org/SDL3/SDL_HINT_APP_ID), and
[SDL application name](https://wiki.libsdl.org/SDL3/SDL_HINT_APP_NAME).

## Clipped content in small flex windows

Flex mode in scrcpy defaults to unscaled rendering. A phone-resolution frame can
therefore be cropped in a small host window while Android resizes, or when an app
keeps its earlier surface size. DroidOps now explicitly requests
`--render-fit=letterbox` when supported, preserving the whole frame and its aspect
ratio in both resize modes. Flex requires both flex-display and render-fit support;
older clients fall back to fixed-resolution scaling. This changes host rendering,
not Android device resolution settings or app policy.

The regression test was observed failing without the flag, then passing with it.
A short Keep Notes launch on the selected Arch/Android 17 setup accepted the
letterbox configuration and created a 1080x2400/440 virtual display. Detailed
visual and resize acceptance should be checked with the user's affected window;
no private note content was captured as a fixture.

The Keep clipping investigation also exposed a density/layout issue: flex mode
shrunk a 1080x2400/440 display to 262x586 while retaining phone density. The
resulting Android layout can be too narrow, independently of host frame fitting.
Phone matching now forces fixed-resolution scaling even for saved preferences
that previously enabled flex. The desktop window remains resizable; Android's
pixels/DPI stay at phone values. Custom mode retains opt-in flex with explicit
DPI controls. Two regression tests were observed failing before these fixes.

The final fixed-mode Keep smoke check exited normally after 10 seconds. scrcpy
reported a 1080x2400 video texture throughout the captured output with a 450x800
resizable host window, instead of shrinking Android to desktop pixel dimensions.
All 34 workspace tests, formatting, Clippy, build, rustdoc, infrastructure, and
whitespace checks passed. FFmpeg profile warnings remained; no private note
content was captured and complete visual acceptance remains with the user.
