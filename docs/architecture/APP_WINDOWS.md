# App window ownership and execution

`droidops-device-services::app_windows::AppWindowManager<R>` owns independent
scrcpy sessions using the existing injectable `Runner` boundary. The production
runner observes stdout/stderr while bounding capture to 4 MiB per process,
executes separate host arguments, enforces cancellation/timeouts, kills and reaps
children, and suppresses console windows on Windows. It never invokes a host shell.

Each session has a monotonic ID, immutable device/package/title, its own cancellation
token, task, and state. Eight concurrent sessions are allowed. A duplicate active
package on the same device is rejected because Android may move its existing task.
Finished sessions remain visible until the next launch, which clears finished history.

Launch validates settings and consent, probes installed scrcpy help, and requests
one new virtual display per process. Optional flex/decorations flags are capability
gated. Unsupported preserve-content requests fail explicitly. This checks client
capabilities; the Android ROM, codec, permissions, and app determine actual support.

Starting becomes Running only after the scrcpy server identifies a non-primary
display for the exact package and a bounded ADB `dumpsys activity activities`
probe finds that package resumed on that display. Unsupported dump/log formats
fail closed after 30 seconds, stopping and reaping the process. A missing launch
intent fails even when the scrcpy window stays alive. Running means initial launch
was verified and the process remains alive; it does not guarantee app rendering,
continuous foreground status, playback, or login success. Processes are limited to
24 hours per session. Session failures expose bounded, redacted messages; raw app
inventories and output are not written to diagnostics.

The native Worker runs a separate bounded app-command channel and actor alongside
ordinary device jobs and full-screen mirroring. It publishes changed snapshots and
requests repaint. Spawning, discovery, label queries, verification, cancellation,
and process cleanup happen off the egui thread. Completed package inventories use
the existing selection generation gate. App-window results belong to immutable
session IDs across device selection changes, rather than changing the current
selected-device snapshot. Discovery cancels windows for disconnected/unauthorized
devices. Selecting another still-online device leaves earlier app windows running.

Normal DroidOps shutdown cancels and awaits all manager tasks before dropping its
runtime. App windows do not intentionally outlive DroidOps. Forceful host termination
cannot guarantee graceful cleanup; scrcpy's connection-loss cleanup still applies.
Each Stop action affects its session alone. Full-screen mirroring retains its own
worker/task/token and is not implemented through AppWindowManager.

Labels are retrieved using `scrcpy --list-apps` alongside ADB package inventory.
Missing scrcpy or unknown label formats retain package IDs and generic vector
icons. No APK/icon extraction or third-party metadata service is introduced.
Preferences store display/audio options, but no devices, app inventories, window
session identities, or title overrides. Force-stop is never restored and requires
fresh confirmation bound to selection generation, package, title, and settings.

Phone-matching launch preferences default on. The background service resolves
selected-device main-display size and density with read-only ADB queries, preferring
Android overrides, before constructing an explicit virtual-display configuration.
The process specification supports child-local environment overrides for SDL app
name/identity; the host environment is never mutated. Flex and fixed-resolution
scaling remain separate modes. See [the feature guide](../features/ANDROID_APP_WINDOWS.md#phone-resolution-resizing-and-window-identity).

Phone-matched sessions use fixed-resolution host scaling, overriding any saved
flex preference. This prevents desktop window pixels from becoming a very small
Android display at the phone's high DPI. Custom mode may use flex when both
flex-display and render-fit are advertised. Explicit letterboxing prevents full
frames from being cropped in small windows during display/surface resize.
