# Repository Guidelines

## Project Structure & Module Organization
`src/` contains the main `hearthfield` crate, split into domain plugins. Shared cross-domain types live in `src/shared/mod.rs`; domains should import `crate::shared` instead of depending on each other directly. Root integration tests live in `tests/`. DLC workspace crates live in `dlc/pilot/` and `dlc/city/`. Assets are in `assets/`; web packaging uses `web_template/`, `web/`, and `build_wasm.sh`. Specs and execution artifacts live in `docs/`, `objectives/`, and `status/`.

## Build, Test, and Development Commands
`cargo run` launches the main game. `cargo check` is the fastest compile gate. `cargo test --test headless` runs Bevy integration tests without a GPU, and `cargo test --test keybinding_duplicates` covers input regressions. `cargo clippy -- -D warnings` is required to stay warning-free. `bash scripts/run-gates.sh` runs the full gate set: contract checksum, `cargo check`, headless tests, clippy, and connectivity checks. Use `cargo test -p skywarden --test headless` or `cargo test -p city_office_worker_dlc` for DLC work. `./build_wasm.sh` builds the browser bundle.

## Coding Style, Naming & Contract
Use Rust 2021 defaults with four-space indentation and format with `cargo fmt --all`. Keep files and modules in `snake_case`; types, plugins, resources, and events belong in `UpperCamelCase`. Wire new plugins and resources in `src/main.rs` carefully because registration order matters. If shared data must cross domains, add it to `src/shared/mod.rs` instead of creating direct domain-to-domain imports. When that contract file changes, update the checksum with `shasum -a 256 src/shared/mod.rs > .contract.sha256`.

## Testing & Agent Workflow
Prefer deterministic, headless coverage. Follow the pattern in `tests/headless.rs`: build a minimal Bevy app, add only the systems under test, call `app.update()`, then assert state changes. Every gameplay fix should ship with a targeted regression test. For sub-agent work, keep the shared contract frozen during parallel implementation, define scope in `docs/domains/*.md`, clamp worker edits with `bash scripts/clamp-scope.sh src/<domain>/`, and record outcomes in `status/workers/`. If clamping breaks a fix, route it to integration instead of widening scope ad hoc.

When a lane discovers something unexpected, type it before acting:

- If it is a current-base reproducible runtime bug inside the lane's owned surface, fix it and add or strengthen the regression.
- If it is a fragile seam with missing direct coverage, add the regression first. Only change gameplay code if that regression proves a real failure.
- If it is a fidelity gap, escalate or backlog it unless the tranche explicitly owns that behavior.
- If it is UX friction, record it unless the tranche explicitly owns usability or polish for that surface.
- If it is comment drift, naming drift, or doc mismatch, report it unless it directly impedes the tranche.
- If it is out of scope, record it and do not widen the lane ad hoc.

`Reproducible` means a named command, test, or exact observed path. After any in-scope fix, the bounded lane flow still applies: clamp, rerun validation, and review only the clamped result.

## Bounded Lane Flow
For any bounded worker or foreman lane, the post-run flow is mandatory:

1. Run the lane.
2. Clamp scope mechanically with `scripts/clamp-scope.sh` using the lane allowlist.
3. Re-run the lane's required validation commands on the clamped result.
4. Review only the post-clamp result.
5. Accept or relaunch.

If a lane produces useful work plus out-of-scope edits, clamp first and judge the clamped result.
If a lane drifts badly or leaves no usable in-scope result, discard it and relaunch rather than salvaging by hand.

Do not review or merge an unclamped bounded-lane result.

## Commit & Pull Request Guidelines
Recent history uses concise conventional subjects such as `feat:`, `fix:`, `chore:`, and scoped forms like `feat(city):`. Keep commits narrow, imperative, and reviewable; repo planning docs call out a practical target of about 20 files and 1,200 insertions per slice. Do not use `WIP` commits. PRs should summarize gameplay impact, list the commands you ran, link the related issue or objective, and call out any `src/shared/mod.rs` changes. Include a screenshot or short capture for UI updates.
