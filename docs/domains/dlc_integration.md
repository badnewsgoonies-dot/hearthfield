# DLC Integration Spec — Start Screen Launcher

## Goal
Add "Skywarden (Pilot DLC)" and "City Office Worker" options to the Hearthfield
main menu. When selected, launch the respective DLC binary. Return to the main
menu when the DLC process exits.

## Architecture Decision
Both DLCs are **standalone Bevy applications** with their own type contracts,
plugin registrations, and window setup. Merging them into a single binary would
require unifying 3 separate type systems (~6,300 lines of contract code) — not
viable. Instead: **subprocess launch** via `std::process::Command`.

## Files to Modify

### `src/ui/main_menu.rs`
1. Add two new options to `MAIN_MENU_OPTIONS`:
   - Native: `["New Game", "Load Game", "Skywarden", "City Office", "Quit"]`
   - WASM: `["New Game", "Load Game"]` (no subprocess launch possible)

2. In `main_menu_navigation`, handle cursor indices:
   - 0 = New Game (existing)
   - 1 = Load Game (existing)
   - 2 = Launch Skywarden
   - 3 = Launch City Office Worker
   - 4 = Quit (native only)

3. For DLC launch (indices 2, 3):
   ```rust
   // Determine the binary path relative to the current executable
   let current_exe = std::env::current_exe().unwrap_or_default();
   let base_dir = current_exe.parent().unwrap_or(std::path::Path::new("."));
   let dlc_binary = base_dir.join("skywarden"); // or "city_office_worker"

   // Spawn the process
   std::process::Command::new(&dlc_binary)
       .spawn()
       .ok();
   ```

4. Update `MAIN_MENU_MAX_ITEMS` to accommodate the new options.

5. Set status message if DLC binary not found:
   `"Skywarden not installed. Build with: cargo build -p skywarden"`

### `Cargo.toml` (root)
Add workspace members so both DLCs build together:
```toml
[workspace]
members = [".", "dlc/pilot", "city_office_worker_dlc"]
```

### DLC Cargo.toml files
Ensure `[[bin]]` section specifies the binary name:
- `dlc/pilot/Cargo.toml`: binary name `skywarden`
- `city_office_worker_dlc/Cargo.toml`: binary name `city_office_worker`

## Constants
- `MAIN_MENU_OPTIONS` (native): 5 items
- `MAIN_MENU_OPTIONS` (wasm): 2 items (DLCs not available on web)
- DLC binary names: `skywarden`, `city_office_worker`

## Does NOT Handle
- Merging DLC plugins into the base game binary
- Shared save files between base game and DLCs
- In-process DLC loading (would require unified type contract)
- WASM DLC support (subprocesses not available)

## Validation
```bash
cargo build --workspace        # All 3 binaries compile
cargo test --test headless     # Base game tests pass
cargo clippy -p hearthfield -- -D warnings  # Base game clippy clean
```
