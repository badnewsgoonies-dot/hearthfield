# Pilot Settings Worker Report

## Changes made
- Updated `dlc/pilot/src/ui/settings.rs` `apply_settings` to apply real runtime settings:
  - Writes effective music/sfx levels into `VolumeSettings` (existing behavior kept).
  - Writes `GameSettings.master_volume` into Bevy `GlobalVolume` when that resource exists.
  - Applies fullscreen toggle to the primary window mode (`BorderlessFullscreen` / `Windowed`).

## Validation
- Ran: `cd dlc/pilot && cargo check`
- Result: **failed in this environment** before project type-check due Cargo/toolchain constraints:
  - lockfile v4 requires `-Znext-lockfile-bump`
  - current Cargo 1.75.0 cannot build downloaded crates requiring unstable `edition2024` support
