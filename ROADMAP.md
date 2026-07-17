# yoshi

> Repo: https://github.com/oxmonty/yoshi

Yoshi is a native, GPU-rendered Jupyter notebook desktop app for macOS and Linux, built in Rust on GPUI — the wgpu-class UI framework behind Zed. It opens, edits, and executes `.ipynb` files against local Jupyter kernels with sub-second cold start to an interactive window and 60fps scrolling through large native outputs — where JupyterLab Desktop is Electron wrapping a web app and nteract Desktop is Tauri wrapping React, yoshi's shell, editor, and common outputs are fully native, with sandboxed webviews appearing only *inside* individual rich outputs: the browser engine renders web content, never the app.

```
yoshi analysis.ipynb   # < 500ms to an interactive window; pick a kernel once, ⇧⏎ runs
```

Usable as:
- **Desktop app**: the notebook editor — open/edit/run/save `.ipynb`, kernel lifecycle management, native output rendering.
- **CLI**: `yoshi <file>` launcher + `yoshi kernels list` (kernelspec discovery, doubles as an install smoke test).

---

## Roadmap

- [ ] **E1: Framework bake-off + hello world** — the repo exists on GitHub, the framework winner is recorded in the decision log, and `cargo run` opens a native window that executes `print("hello, yoshi")` against a real local ipykernel — after an explicit kernel-ready handshake — and displays the output, on both dev platforms, before any packaging exists. Honest sizing: ~2–3 weeks of evenings. → [Kernel session loop](PRD.md#kernel-session-loop), [Project structure](PRD.md#project-structure)
    - [x] Repo scaffolded and pushed: workspace layout, `script/bootstrap`, `script/run`, README with the pitch line
    - [x] Spike A (timebox: 2 evenings): `warpui` + `warpui_core` as git deps at a pinned commit — build a window with a scrollable text list; capture `cargo tree` + `cargo deny`, text-input primitives, IME entry of a CJK string, clipboard round-trip, whether a native view/window handle is exposed (gates E9's overlay path), accessibility support (assume none); glance at the entanglement of Warp's AGPL `warp_editor` and `ipynb_parser` crates — reuse is a bonus discovered here, never a plan
    - [x] Spike B (timebox: 2 evenings): same window and same captures on GPUI — plus skim how Zed's `crates/repl` structures kernel-channel tasks
    - [x] Decide and record: criteria are (1) builds standalone with a clean license tree, (2) a usable text-input primitive path, (3) docs/examples good enough to be productive solo, (4) API stability outlook — tie-breaker to GPUI (proven as an external dependency, Apache-2.0, Zed's repl is a working reference for the riskiest integration, and the single-runtime path is validated on it); warpui must win on measured spike evidence
    - [x] Hello world on the winner (timebox: 3 evenings): window renders a hardcoded cell, Run spawns ipykernel via `jupyter-zmq-client`, waits for ready (kernel_info reply + first iopub status), sends one ExecuteRequest, renders the stream output — proving the framework event loop and the kernel I/O runtime coexist (the project's single riskiest integration)

- [ ] **E2: CI + unsigned artifacts (walking skeleton)** — the E1 hello world, unchanged, downloads from GitHub Releases and runs on a clean machine (zipped `.app` on macOS via right-click-Open, AppImage on Linux), with CI green on both platforms. Signed installers and Homebrew arrive with v0.1 (E8). → [Distribution](PRD.md#distribution), [CI/CD](PRD.md#cicd)
    - [ ] GitHub Actions matrix (macos-14, ubuntu-24.04): fmt, clippy, `cargo deny`, nextest
    - [ ] Tag → build → GitHub Release: zipped `.app` + AppImage + checksums, unsigned (Gatekeeper bypass documented in the README)
    - [ ] Minimal app identity: macOS bundle with Info.plist (bundle id `com.oxmonty.yoshi`, version from Cargo) + placeholder `.icns`; AppImage carries a `.desktop` file + icon
    - [ ] `yoshi --version` and `yoshi kernels list` work from a downloaded artifact (kernelspec-discovery smoke test)
    - [ ] Every later epic stays green on CI from here; tag per epic

- [ ] **E3: Kernel runtime** — a headless integration test launches ipykernel, passes the ready handshake, executes `print("hi")`, receives the stream output, interrupts a busy kernel, and shuts down cleanly; shipped as a `yoshi-kernels` crate with CI coverage. → [Kernel session loop](PRD.md#kernel-session-loop)
    - [ ] Kernelspec discovery reads kernelspec JSON directly from disk (all standard dirs, including `~/Library/Jupyter/kernels` on macOS; never shells out to `jupyter`)
    - [ ] Launch in its own process group, connection file written 0600 to the Jupyter runtime dir, `kill_on_drop`, stale-file cleanup; shutdown/restart lifecycle
    - [ ] Ready gate after every launch and restart: `kernel_info_request` reply + first observed iopub `status` before accepting work (iopub SUB is a slow joiner)
    - [ ] Session actor: shell + iopub + control routing tasks; outputs keyed by `parent_header.msg_id`; consumes `execute_reply` for `execution_count` and the ok/error/aborted verdict; `allow_stdin: false` on every execute
    - [ ] Interrupt honors the kernelspec `interrupt_mode`: SIGINT to the process group (ipykernel's default), `interrupt_request` on control for message-mode kernels
    - [ ] Execution state machine (starting/idle/busy/dead) exposed as a watch channel; CI installs python + ipykernel and runs the round-trip headlessly

- [ ] **E4: Notebook document model** — every notebook in the golden corpus round-trips idempotently to canonical form: the first save may reflow once to the same output `nbformat.writes` produces, and every save thereafter is byte-stable; `yoshi-notebook` crate with golden tests and zero UI deps. → [Document model](PRD.md#document-model)
    - [ ] Fidelity spike (story 1, gates the rest): confirm what the `nbformat` v3 crate preserves — unknown fields survive via flattened maps, but key order does not; wrap with a `preserve_order` `serde_json::Value` layer or a custom serializer, and evaluate Warp's AGPL `ipynb_parser` as an alternative; decision recorded
    - [ ] Canonical writer matching `nbformat.writes`: 1-space indent, `ensure_ascii=False`, trailing newline, source as line-arrays, adjacent same-name streams merged; v4.0 notebooks are not upconverted (no cell ids added) on open
    - [ ] Cell CRUD (insert, delete, move, change type) on the in-memory model
    - [ ] Model-level undo/redo stack over cell operations (min depth 100)
    - [ ] Dirty tracking + atomic save (write-temp-rename)
    - [ ] Golden corpus curated for producer and encoding diversity: classic Notebook, JupyterLab, VS Code, and papermill outputs; v4.0 and v4.5; non-ASCII/emoji content; widget and unknown metadata

- [ ] **E5: Cell editor** — the single largest net-new UI component, built as its own epic on the E1 decision: type, select, and syntax-highlight code in a cell, with working IME and clipboard. Approach gated on E1's primitive inventory; `warp_editor` reuse only if Spike A showed extraction cheap. → [Cell editor](PRD.md#cell-editor)
    - [ ] Text buffer + selection model: helix-core ropes on GPUI's input primitives (E1 confirmed GPUI ships primitives but no editor widget)
    - [ ] Rendering: cosmic-text shaping/layout wired into the framework's draw path; cursor + selection painting; intra-cell scroll
    - [ ] Syntax highlighting (tree-sitter or syntect) for Python and markdown source
    - [ ] IME + clipboard correctness: CJK entry, dead keys, text copy/paste in both directions
    - [ ] Intra-cell text undo/redo from the editor history (structural cell undo lives in the model, E4)

- [ ] **E6: Notebook UI** — a user opens a real notebook, navigates cells with Jupyter's two-mode keyboard model, edits code and markdown, and runs cells against a live kernel; demo GIF in the README cut from a release build. → [Notebook editing loop](PRD.md#notebook-editing-loop), [Surfaces](PRD.md#surfaces)
    - [ ] Scrollable cell list with selection and command/edit modes
    - [ ] Command-mode keyboard parity: `A`/`B` insert, `DD` delete, `M`/`Y` type toggle, `C`/`X`/`V` cell clipboard, `Z`/`⇧Z` structural undo/redo wired to the E4 stack, `↑↓`/`⏎`/`Esc`; `⇧⏎`/`⌃⏎`/`⌥⏎` run variants
    - [ ] Markdown cells toggle rendered↔source: rendered when unselected, raw source in edit mode, re-render on run
    - [ ] Run All and Restart-and-Run-All (cells aborted after an error show as aborted, not running)
    - [ ] Kernel status indicator + kernel picker — the picker is the common path, not a fallback: notebook metadata usually names bare `python3`, which mispicks environments
    - [ ] File open/save/save-as with native dialogs; New Notebook (`⌘N`); native macOS menu bar (`cx.set_menus`) with File/Edit/Window routing the same actions as the shortcuts

- [ ] **E7: Output rendering, tier 1** — matplotlib inline plots, pandas text reprs, tracebacks, and streaming stdout all render correctly in the golden-notebook structure tests; the five MVP renderers ship behind a MIME-ranking dispatcher. → [Output rendering](PRD.md#output-rendering)
    - [ ] MIME bundle ranking (richest-renderable-first, Zed's model), with `text/html` explicitly ranked below `text/plain` in MVP so the sibling fallback is unambiguous
    - [ ] Renderers: ANSI/plain text streaming (adjacent-stream merge, `\r` handling), error/traceback, PNG/JPEG, markdown, latex-as-plain (deferred math)
    - [ ] `clear_output(wait)` as a first-class output-store operation, plus `update_display_data`/`display_id` mutation — both mechanisms power progress bars
    - [ ] Output cap + "show more" for large streams; clear-cell-outputs and clear-all commands
    - [ ] Golden tests assert renderer choice, MIME dispatch, and output structure — not pixels (GPU rasterization differs across platforms)

- [ ] **E8: v0.1 hardening + distribution** — v0.1.0 is cut, signed and notarized, installable via `brew install oxmonty/tap/yoshi` on macOS and AppImage on Linux, with benchmarks published in the README. → [Validation strategy](PRD.md#validation-strategy), [Distribution](PRD.md#distribution)
    - [ ] Bench harness committed: cold start = process launch → UI interactive (warm caches); kernel-ready reported separately (bounded by CPython startup); scroll FPS on a tier-1 capped-output notebook; measured against nteract Desktop and JupyterLab Desktop
    - [ ] Crash-safe autosave / sidecar recovery file
    - [ ] Settings: `~/.config/yoshi/settings.json` + `keymap.json` — defaults written on first run, a menu command opens them for manual editing (editor settings, keybinding overrides); no settings GUI in v0.1
    - [ ] Themes, persisted in settings.json: Gruvbox Dark Soft (default), Gruvbox Light, One Dark
    - [ ] macOS signing: Developer ID + notarytool + stapling + hardened-runtime entitlements; `.dmg` artifact
    - [ ] Homebrew cask in `oxmonty/homebrew-tap` (macOS-only; CLI exposed via the cask `binary` stanza) + release-please tag pipeline
    - [ ] Branding pass: real app icon (`.icns` + Linux icon), About panel, `.dmg` background — logo asset needed before this story

---
*MVP line — E1–E8 ship as v0.1: a native notebook editor that opens, edits, executes, and saves real-world `.ipynb` files against local Python kernels, with tier-1 outputs, undo/redo, and Jupyter keyboard parity, installable from Homebrew (macOS) and GitHub Releases.*

- [ ] **E9: Rich outputs, tier 2 (webview)** — pandas `text/html` tables and plotly figures render in sandboxed, virtualized webviews that scroll and clip correctly within the cell list; opens the feedback loop with data-science early adopters on remaining unrenderable MIME types. The largest post-MVP unknown. → [Output rendering](PRD.md#output-rendering)
    - [ ] Spike, ordered by kill-risk: (1) Linux embedding — wry child views are X11-only, so probe the GTK-host path under Wayland first; (2) attach to the native view handle captured in E1; (3) scroll-sync/clipping quality during momentum scroll. Go/no-go recorded
    - [ ] Escalation ladder recorded: wry overlay → `wgpu-scry` (system webview composited into a wgpu texture) → static-image fallback (plotly static export + "open in browser") → CEF offscreen rendering as last resort (input plumbing, process supervision, and per-helper notarization — not just bundle size)
    - [ ] Webview pool: create-on-visible, recycle-on-scroll, hard cap on live instances
    - [ ] Sandbox policy: no fs/network bridge, plotly.js bundled locally, CSP locked down (nteract's iframe-isolation model is the reference)
    - [ ] Native table view for `application/vnd.dataresource+json`; SVG via resvg (both stay native); `video/mp4` outputs play in the sandboxed webview (system codecs, no media dependencies)
    - [ ] Instrument: opt-in, local-only logging of unrenderable MIME types to prioritize tier 3

- [ ] **E10: Workspace shell** — a user opens a folder, browses a project tree, and arranges notebooks side by side in draggable grid panes; opens the feedback loop with users who live in multi-notebook projects. → [Workspace shell](PRD.md#workspace-shell)
    - [ ] Pane grid: horizontal/vertical split, resize, close; drag a pane header to swap positions (Warp's drag-to-swap behavior is the UX reference; Zed's pane system the architecture reference, GPL, read-only)
    - [ ] Project tree sidebar: `ignore`-crate walker + `notify` file watching, rendered on the same virtualized list machinery as cells; toggleable
    - [ ] Open notebooks from the tree into panes — each pane hosts an independent notebook view with its own kernel session
    - [ ] Tree preview for images (native renderers) and `video/mp4` (sandboxed webview), reusing the output pipeline
    - [ ] Workspace state (open panes, layout, tree visibility) persisted and restored across restarts

- [ ] **E11: Terminal** — a GPU-rendered terminal pane runs a real shell in the grid next to a notebook (the agent workflow: Claude Code editing the notebook you're viewing); drag-swappable like any pane. → [Terminal](PRD.md#terminal)
    - [ ] Terminal engine: `alacritty_terminal` (Apache-2.0) for PTY + grid state, rendered as a native GPUI view (Zed's `crates/terminal` is the architecture reference, GPL, read-only)
    - [ ] Terminal pane type in the E10 grid: shell spawned in the workspace directory, drag-swap with notebook panes
    - [ ] Keyboard/IME passthrough, scrollback, selection + copy/paste, ANSI colors from the active theme
    - [ ] Warp UX pass: catalogue the interaction details worth porting by hand (drag-swap affordances, block-style polish — warpui's MIT components are fair styling references; no code extraction)

- [ ] **E12: Remote kernels** — connect to a running Jupyter server over WebSocket (`jupyter-websocket-client`); opens the loop with users on remote/SSH/cloud workflows. Needs slicing into stories before pickup. → [Future: remote kernels](PRD.md#future-remote-kernels)

**Future (considered, unscheduled)**: ipywidgets via a comm bridge over the sandboxed webview channel — substantially cheaper under the hybrid model than a native reimplementation ([here](PRD.md#output-rendering)); MCP server for agent-driven notebook editing, following nteract's `runt mcp` ([here](PRD.md#competitive-landscape)); in-notebook find/replace (`⌘F`); Windows support ([here](PRD.md#distribution)); a GUI settings page (settings stay editable JSON files until then).
