# E1: Framework bake-off + hello world

**Date:** 2026-07-16 (single day, one long session)
**Status:** stories 1–5 done; epic box held open pending Linux verification (arrives with E2's CI matrix)

## What shipped

- Cargo workspace (7 crates + xtask), bootstrap/run scripts, README — `7a41bbb`.
- Both bake-off spikes with full CAPTURES: warpui at Warp commit `9fd88b6fe917`, GPUI 0.2.2 from crates.io — `0c449cb`, plus an IME/clipboard test binary adapted from gpui's input example — `28f8176`.
- **Framework decision: GPUI**, recorded in PRD Open Questions.
- The hello world, beyond spec: an editable code cell (the spike's IME-tested input, lifted into `yoshi-app`), a kernel that **prewarms at app startup** and stays alive across runs, ⇧⏎ to run, results and tracebacks rendered alongside streams. Headless mode (`--headless`) runs the same session logic and asserts `"hello, yoshi\n"`, exit 0 — the CI hook for E3.

## Evidence

- `cargo run -p yoshi-app -- --headless` → exit 0, output asserted byte-equal.
- Human checks on GPUI: 1000-row scroll momentum smooth; clipboard round-trip both directions; accent-picker IME composition commits correctly.
- `spikes/*/CAPTURES.md` hold the raw bake-off data.

## Decisions made along the way

- **GPUI over warpui, on all four criteria** — not the tie-breaker. warpui failed to build standalone without hand-replicating Warp's internal `[patch.crates-io]` table, pulled six AGPL warp-internal crates for a bare window+list, has no text-editing model (`TextInput` is a styled container), and exposes no raw-window-handle (which would have killed E9's overlay path outright).
- **Single-runtime validated in practice**: kernel ZMQ I/O runs on GPUI's executor via a 12-line `async_dispatcher::Dispatcher` bridge; headless uses `thread_dispatcher()`. No tokio in the process.
- **deny.toml grew five documented transitive exceptions** (CC0-1.0, NCSA, 0BSD, BSL-1.0, LLVM-exception) — all surfaced by the gpui tree, all permissive, each annotated with the crate that needs it.
- **Kernel prewarm became the architecture** after the first GUI felt slow: the original hello world spawned a fresh kernel per click (~2–3s of CPython each time). Restructured to boot once at startup and hold the session; repeat runs are a bare protocol round-trip. This is the PRD's "kernel launch never blocks first paint" made concrete a full epic early.

## What surprised

- **The pre-session staff review + fact-check paid for itself immediately.** The researcher found the `nbformat` crate doesn't preserve key order (killing the byte-identical round-trip promise before E4 could trip on it), and the staff reviewers' iopub slow-joiner warning was wired into the hello world's ready-gate from the first line — the intermittent lost-output bug never got a chance to exist.
- **warpui's "MIT framework" is real but not consumable in isolation** — the license is MIT, the dependency graph is Warp. The bake-off criteria caught what the README framing hides.
- **The adapted IME example crashed on first human contact**: a one-token transcription bug (`range.end` for `range.start`) in the marked-text selection offset, surfaced by macOS's press-and-hold accent picker, aborting inside a non-unwinding Objective-C callback. Fixed in `28f8176`. Lesson: IME paths need hands on keys, not just code review.
- **A delegated implementation agent died mid-story on a usage limit**; the story was finished in the main loop instead. The precise upfront API research (exact crate versions, function signatures pulled from local registry sources) made the takeover cheap.

## Left open

- Linux run of the same hello world (E2's CI matrix is the proof).
- The `yoshi` GUI's editable cell is a single-line input — the real multi-line cell editor is E5, as planned.
