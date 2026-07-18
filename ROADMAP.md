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

- [x] **E1: Framework bake-off + hello world** — the repo exists on GitHub, the framework winner is recorded in the decision log, and `cargo run` opens a native window that executes `print("hello, yoshi")` against a real local ipykernel — after an explicit kernel-ready handshake — and displays the output, on both dev platforms, before any packaging exists. Honest sizing: ~2–3 weeks of evenings. → [stories](docs/epics/E1-framework-bake-off.md)

- [ ] **E2: CI + unsigned artifacts (walking skeleton)** — the E1 hello world, unchanged, downloads from GitHub Releases and runs on a clean machine (zipped `.app` on macOS after the documented quarantine strip — or quarantine-free via the curl installer — AppImage on Linux), with CI green on both platforms. Signed installers and Homebrew arrive with v0.1 (E8). → [stories](docs/epics/E2-ci-unsigned-artifacts.md)

- [ ] **E3: Kernel runtime** — a headless integration test launches ipykernel, passes the ready handshake, executes `print("hi")`, receives the stream output, interrupts a busy kernel, and shuts down cleanly; shipped as a `yoshi-kernels` crate with CI coverage. → [stories](docs/epics/E3-kernel-runtime.md)

- [ ] **E4: Notebook document model** — every notebook in the golden corpus round-trips idempotently to canonical form: the first save may reflow once to the same output `nbformat.writes` produces, and every save thereafter is byte-stable; `yoshi-notebook` crate with golden tests and zero UI deps. → [stories](docs/epics/E4-notebook-document-model.md)

- [ ] **E5: Cell editor** — the single largest net-new UI component, built as its own epic on the E1 decision: type, select, and syntax-highlight code in a cell, with working IME and clipboard. Approach gated on E1's primitive inventory; `warp_editor` reuse only if Spike A showed extraction cheap. → [stories](docs/epics/E5-cell-editor.md)

- [ ] **E6: Notebook UI** — a user opens a real notebook, navigates cells with Jupyter's two-mode keyboard model, edits code and markdown, and runs cells against a live kernel; demo GIF in the README cut from a release build. → [stories](docs/epics/E6-notebook-ui.md)

- [ ] **E7: Output rendering, tier 1** — matplotlib inline plots, pandas text reprs, tracebacks, and streaming stdout all render correctly in the golden-notebook structure tests; the five MVP renderers ship behind a MIME-ranking dispatcher. → [stories](docs/epics/E7-output-rendering-tier-1.md)

- [ ] **E8: v0.1 hardening + distribution** — v0.1.0 is cut, signed and notarized, installable via `brew install oxmonty/tap/yoshi` on macOS and AppImage on Linux, with benchmarks published in the README. → [stories](docs/epics/E8-v01-hardening-distribution.md)

---
*MVP line — E1–E8 ship as v0.1: a native notebook editor that opens, edits, executes, and saves real-world `.ipynb` files against local Python kernels, with tier-1 outputs, undo/redo, and Jupyter keyboard parity, installable from Homebrew (macOS) and GitHub Releases.*

- [ ] **E9: Rich outputs, tier 2 (webview)** — pandas `text/html` tables and plotly figures render in sandboxed, virtualized webviews that scroll and clip correctly within the cell list; opens the feedback loop with data-science early adopters on remaining unrenderable MIME types. The largest post-MVP unknown. → [stories](docs/epics/E9-rich-outputs-tier-2.md)

- [ ] **E10: Workspace shell** — a user opens a folder, browses a project tree, and arranges notebooks side by side in draggable grid panes; opens the feedback loop with users who live in multi-notebook projects. → [stories](docs/epics/E10-workspace-shell.md)

- [ ] **E11: Terminal** — a GPU-rendered terminal pane runs a real shell in the grid next to a notebook (the agent workflow: Claude Code editing the notebook you're viewing); drag-swappable like any pane. → [stories](docs/epics/E11-terminal.md)

- [ ] **E12: Remote kernels** — connect to a running Jupyter server over WebSocket (`jupyter-websocket-client`); opens the loop with users on remote/SSH/cloud workflows. Needs slicing into stories before pickup. → [stories](docs/epics/E12-remote-kernels.md)

**Future (considered, unscheduled)**: ipywidgets via a comm bridge over the sandboxed webview channel — substantially cheaper under the hybrid model than a native reimplementation ([here](PRD.md#output-rendering)); MCP server for agent-driven notebook editing, following nteract's `runt mcp` ([here](PRD.md#competitive-landscape)); in-notebook find/replace (`⌘F`); Windows support ([here](PRD.md#distribution)); a GUI settings page (settings stay editable JSON files until then).
