# E1: Framework bake-off + hello world

The repo exists on GitHub, the framework winner is recorded in the decision log, and `cargo run` opens a native window that executes `print("hello, yoshi")` against a real local ipykernel — after an explicit kernel-ready handshake — and displays the output, on both dev platforms, before any packaging exists. Honest sizing: ~2–3 weeks of evenings.

Spec: [Kernel session loop](../../PRD.md#kernel-session-loop), [Project structure](../../PRD.md#project-structure)

Write-up: [E1 write-up](../write-ups/E1-framework-bake-off.md)

## Stories

- [x] Repo scaffolded and pushed: workspace layout, `script/bootstrap`, `script/run`, README with the pitch line
- [x] Spike A (timebox: 2 evenings): `warpui` + `warpui_core` as git deps at a pinned commit — build a window with a scrollable text list; capture `cargo tree` + `cargo deny`, text-input primitives, IME entry of a CJK string, clipboard round-trip, whether a native view/window handle is exposed (gates E9's overlay path), accessibility support (assume none); glance at the entanglement of Warp's AGPL `warp_editor` and `ipynb_parser` crates — reuse is a bonus discovered here, never a plan
- [x] Spike B (timebox: 2 evenings): same window and same captures on GPUI — plus skim how Zed's `crates/repl` structures kernel-channel tasks
- [x] Decide and record: criteria are (1) builds standalone with a clean license tree, (2) a usable text-input primitive path, (3) docs/examples good enough to be productive solo, (4) API stability outlook — tie-breaker to GPUI (proven as an external dependency, Apache-2.0, Zed's repl is a working reference for the riskiest integration, and the single-runtime path is validated on it); warpui must win on measured spike evidence
- [x] Hello world on the winner (timebox: 3 evenings): window renders a hardcoded cell, Run spawns ipykernel via `jupyter-zmq-client`, waits for ready (kernel_info reply + first iopub status), sends one ExecuteRequest, renders the stream output — proving the framework event loop and the kernel I/O runtime coexist (the project's single riskiest integration)
