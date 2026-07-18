# yoshi PRD

Design specification for [ROADMAP.md](ROADMAP.md). Roadmap epics link into the sections below; each section is the spec for the stories that reference it.

## Workflow

### Notebook editing loop

```
open .ipynb ──▶ nbformat parse ──▶ NotebookModel ──▶ cell list renders
                                        │
   ⇧⏎ on cell ──▶ ExecuteRequest ──▶ shell socket ──▶ kernel
                                        │
   iopub stream (status/stream/display_data/execute_result/error/clear_output)
        │  keyed by parent_header.msg_id
        ▼
   OutputStore mutates cell outputs ──▶ MIME ranking ──▶ renderer ──▶ re-render
                                        │
   ⌘S ──▶ canonical nbformat serialize ──▶ atomic write
```

Implementation traps:
- **iopub/shell ordering is not guaranteed relative to each other.** `execute_reply` on shell can arrive before trailing iopub outputs. Never treat `execute_reply` as "done" — completion is iopub `status: idle` with a matching `parent_header.msg_id`. Key every output to its parent msg_id, not "the currently running cell". Do consume `execute_reply` too: it carries `execution_count` and the ok/error/aborted verdict (on run-all, an error makes the kernel reply `aborted` for queued cells — show them as aborted, not running).
- **Progress bars use two mutation mechanisms.** Outputs carrying a `display_id` must be stored addressably — a later `update_display_data` targets the same id across *all* cells that displayed it. Separately, plain tqdm and hand-rolled loops use `clear_output(wait=True)` then reprint; `wait=True` means don't clear until the next output arrives. Both are first-class output-store operations.
- **nbformat round-trip churn.** Reordering JSON keys or dropping unknown metadata produces massive git diffs and destroys downstream tooling trust. The guarantee yoshi makes is *idempotent canonical round-trip* (see [Document model](#document-model)) — golden tests assert save-after-save byte stability, and unknown fields, cell ids, and the source minor version are always preserved.
- **Two undo domains must compose.** Intra-cell text undo (editor history) and structural cell undo (model-level stack) are separate systems that must share one user-visible history — `Esc`, `Z` restores the last deleted cell, not a character.
- **The pure-Rust `zeromq` crate has had kernel-specific quirks** — R and Rust kernels reported heartbeat/unresponsiveness issues against it (runtimed #73, since closed). MVP claims Python kernel support only; other kernels are "may work".
- **GPUI is a fast-moving dependency.** Pin the exact version, and budget for API breakage on every bump — crates.io publishes lag the Zed repo by months, so each upgrade can carry a large API delta. Run `cargo deny` in CI with the license allowlist (MIT/Apache/BSD/MPL/AGPL-3.0) so nothing unknown or proprietary sneaks into the tree.

### Kernel session loop

```
kernelspec discovery ──▶ pick spec ──▶ write connection file (0600) ──▶ spawn kernel
      │                                          (own process group)      │
      ▼                                                                   ▼
 jupyter-zmq-client connect (shell / iopub / control / stdin / heartbeat)
      │
 ready gate: kernel_info_request reply + first iopub status
      │
 session actor owns sockets; UI talks to it via channels only
      │
 interrupt ──▶ per interrupt_mode     restart ──▶ shutdown_request + respawn + ready gate
```

Implementation traps:
- **iopub SUB is a slow joiner.** ZeroMQ SUB sockets drop everything published before the subscription is established — execute immediately after spawn and the kernel's early messages (including your output) can vanish. The failure is intermittent and looks like a rendering bug. After connecting, poll `kernel_info_request` on shell until the reply arrives *and* at least one iopub `status` message has been observed, before sending any execution (this is what `jupyter_client.wait_for_ready` does). The ready gate applies after every launch and every restart.
- **Interrupt honors the kernelspec `interrupt_mode`.** The default — and ipykernel's — is `signal`: send SIGINT to the kernel's process group (spawn with its own group via setsid/`process_group(0)`, or the kernel's subprocesses won't be interrupted). Only `interrupt_mode: "message"` kernels take `interrupt_request` on the control channel.
- **Zombie kernels.** Spawn with `kill_on_drop(true)` and reap on window close, crash, and restart; clean up stale connection files.
- **Heartbeat vs. busy.** A kernel executing a long computation is heartbeat-alive but shell-unresponsive; the status UI must distinguish busy from dead or users will force-restart mid-computation.
- **stdin: fail loud, not silently wrong.** MVP sets `allow_stdin: false` on every execute — `input()` then raises `StdinNotImplementedError`, which renders as a normal traceback. Auto-replying an empty string would let programs continue on bad input; a native prompt is a tier-2 nicety.
- **Connection-file security.** The file contains the HMAC key: write it 0600 in the Jupyter runtime dir. The transport signs outgoing and verifies incoming messages (`jupyter-zmq-client` handles this; it matters most for E12 remote).

### Managed default kernel

Zero-setup first run without bundling Python: on first launch yoshi provisions a **managed kernel** via `uv` (the uv binary ships inside the app; the first provision needs network) — `uv python install` + a managed env with ipykernel under yoshi's data dir (`~/.local/share/yoshi/managed-kernel/`), outside the app bundle so app updates never wipe user-installed packages. Users install packages into it with `uv pip install` (a UI affordance for this can come later). The kernel picker lists the managed kernel first, registered kernelspecs below — casual use gets instant execution, project users keep their own environments. Known limit, stated openly: a shared managed env accumulates conflicting packages over time; it is the onboarding path, not a substitute for per-project environments. Auto-detection of unregistered venvs/conda envs in the picker is future work.

### Future: remote kernels

The session actor's public API (execute, interrupt, status watch, output stream) is transport-agnostic from day one — `jupyter-protocol` types are shared between the ZeroMQ and WebSocket clients, so E12 swaps the transport behind the same trait rather than reworking the UI. The discipline now: no ZeroMQ types leak above `yoshi-kernels`.

---

## Surfaces

| Surface | Primary user | What's on it |
|---|---|---|
| Desktop app | Data scientist / notebook author | Notebook view (cell list, editor, outputs, run controls); kernel status bar (state, interrupt, restart, picker); file open/save dialogs; settings via `settings.json` + `keymap.json` |
| CLI | Same human + CI smoke tests | `yoshi <file>` (launch app on file), `yoshi kernels list` (print discovered kernelspecs), `yoshi --version` |

Views (desktop app, MVP):
- **Notebook view** — the cell list; primary actions: run cell, edit cell. Everything else hangs off it.
- **Kernel status bar** — kernel name + state dot; primary actions: interrupt, restart.
- **Kernel picker** — the managed kernel first, registered kernelspecs below; primary action: select (persisted into the notebook's `kernelspec` metadata on save). This is the common path, not an edge case: notebook metadata usually names bare `python3`, which resolves to whatever interpreter that is (frequently the wrong env), and venvs without `ipykernel install` don't appear as kernelspecs at all — the managed kernel is the reliable default.

**Surface-specific (deliberate non-parity):** headless execution (`yoshi run notebook.ipynb`) is *not* in the CLI — `jupyter nbconvert --execute` and papermill own that space, and shipping a worse version dilutes the pitch. The CLI exists to launch the app and smoke-test installs, nothing more.

---

## Validation strategy

Two models: **golden/integration tests** (notebooks are a round-trip format — corruption is the trust-killer) and a **competitor benchmark** (the performance pitch must be a measured number, not vibes). Naive metrics fail here in a specific way: "renders the notebook" is meaningless without structure assertions, and startup time is meaningless without a defined stopwatch.

1. **Golden round-trip + execution corpus** (primary/cheap/objective) — a corpus curated for producer and encoding diversity (see [Document model](#document-model)): load→save must be idempotently byte-stable; execute-all against pinned ipykernel must produce outputs matching committed expectations (normalized for timestamps/ids). The corpus includes named protocol fixtures — interrupt of a busy kernel, `allow_stdin` error, `update_display_data`, `clear_output(wait)`, abort-after-error — so protocol conformance is exercised end-to-end against a real kernel rather than tracked as a separate metric. Runs on every PR.
2. **Startup + scroll benchmarks vs incumbents** (the metric that matters) — measured by a committed harness on the same machine class against nteract Desktop (the maintained comparator) and JupyterLab Desktop (the abandoned incumbent). Defined stopwatches: **cold start** = process launch → UI interactive (first executable cell selectable), warm caches, tier-1 outputs; **kernel-ready** is reported separately, because launching ipykernel costs 1–3s of CPython startup yoshi doesn't control. Frame-time target: 60fps scrolling tier-1 native outputs (tier-2 webview outputs are outside the 60fps guarantee; the 10k-line bench scrolls the capped view E7 ships).

Shipped as: `cargo xtask validate` → `target/validation/report.md` (published in release notes; bench numbers in README).

---

## Features

### Cell model & keyboard interaction

Jupyter's two-mode model, kept deliberately: **command mode** (cell-level: `A`/`B` insert, `DD` delete, `M`/`Y` type toggle, `C`/`X`/`V` cell copy/cut/paste, `Z`/`⇧Z` structural undo/redo, `↑↓` navigate, `⏎` to edit) and **edit mode** (text-level, `Esc` out). `⇧⏎` run-and-advance, `⌃⏎` run-in-place, `⌥⏎` run-and-insert. Run All and Restart-and-Run-All from the command palette/menu. Scope: code and markdown cells; raw cells load and save intact but render as plain text. The cell clipboard is separate from the editor's text clipboard.

Stated v0.1 gaps: no in-notebook find/replace (deferred to v0.2); no math rendering in markdown (`$…$` shows raw source); no screen-reader accessibility — neither candidate framework provides an accessible text editor, and this is recorded as a known limitation, not silently dropped.

**Deviation to explore:** Warp-style *block* affordances on cells — click-to-select whole cell+output as a unit, copy-as-markdown, jump-to-cell palette (`⌘P` over cell headings). Measured by whether early users mention navigation in feedback; it's the visible "this came from Warp DNA" differentiator.

### Cell editor

The code cell editor is the single largest net-new UI component in the MVP and is budgeted as its own epic (E5). The path: **assemble from permissive parts** — helix-core (MPL-2.0) for rope/selection, cosmic-text (MIT/Apache) for shaping/layout — wired into GPUI's draw and input paths, with cursor/selection painting, intra-cell scrolling, syntax highlighting (tree-sitter or syntect), IME (CJK, dead keys), and clipboard built on top. The E1 spike settled the make-vs-take question: GPUI ships real input *primitives* (`EntityInputHandler`, `ElementInputHandler`, marked-text IME) but no editor widget — its upstream input example hand-rolls ~780 lines of cursor/selection/IME on those primitives, which is the shape of work E5 budgets for.

Editor reuse is off the table, priced during E1: Warp's AGPL `warp_editor` is terminal-shaped (no cell semantics), coupled to warpui, and drags in seven more warp-internal AGPL crates one level deep; Zed's editor is GPL and monorepo-entangled on the same extraction-cost grounds. Intra-cell text undo comes from the editor's history; structural cell undo lives in the document model — the two must present one coherent history to the user.

### Output rendering

MIME ranking dispatcher (Zed's model): given a bundle, pick the richest renderable type. MVP order: `image/png` > `image/jpeg` > `text/markdown` > `application/vnd.jupyter.stderr`-styled tracebacks > `text/plain` (ANSI-aware) > `text/html` (unrenderable in MVP — ranking it below `text/plain` makes the sibling fallback automatic).

| MIME / message | MVP behavior |
|---|---|
| `stream` (stdout/stderr) | Streaming ANSI text, appended live; adjacent same-name streams merged; `\r` handled; capped with expand |
| `error` | Traceback through ANSI renderer, colors preserved |
| `clear_output` | Clears the cell's outputs; `wait=True` defers until the next output arrives |
| `image/png`, `image/jpeg` | Native image render, retina-aware, sized to line grid |
| `text/markdown` | Native markdown (no raw HTML pass-through) |
| `execute_result` `text/plain` | ANSI text (covers pandas/polars reprs — pandas emits a `text/plain` sibling alongside `text/html`, so plain `df` display renders a readable table in MVP) |
| `text/html` | Sibling fallback via ranking; placeholder when HTML-only (`Styler`, folium, `IPython.display.HTML`) — tier 2 renders these in a sandboxed per-output webview |
| `application/vnd.plotly.*` | Placeholder in MVP; tier 2 webview with locally bundled plotly.js |
| `video/mp4` | Placeholder in MVP; tier 2 sandboxed webview `<video>` (system codecs) |
| ipywidgets comms | Placeholder with type name; Future (comm bridge over the sandboxed webview channel) |

**Hybrid rendering — native-first, webview-per-output.** Shell, cells, editor, and tier-1 outputs are GPU-native; `text/html`, plotly, and (future) ipywidgets render in embedded per-output webviews. Constraints that make this work instead of quietly becoming Electron: (1) **virtualized** — webviews exist only for rich outputs currently on screen, recycled from a small pool on scroll; (2) **sandboxed** — notebook outputs are untrusted code, so no filesystem or network bridge, plotly.js bundled locally, comm access only through an explicit channel (nteract's iframe-isolation model is the reference); (3) **contained** — a webview never hosts app chrome, only output content.

The escalation ladder, in order:
1. **wry overlay** (system webview as a child view: WKWebView on macOS — viable in principle, since GPUI's window exposes a real `NSView` through raw-window-handle, verified in the E1 spike; on Linux, `new_as_child` is X11-only — Wayland requires the GTK-host path, which the E9 spike probes first). Known costs even when it works: child views composite *above* the wgpu surface, so app chrome (palettes, menus, drag previews) cannot overlap a rich output — an accepted limitation — and scroll-sync can visibly swim during momentum scroll.
2. **`wgpu-scry`** — composites system webviews into wgpu textures (no bundled browser; Linux caveats around WebKitGTK GPU import and synthesized input). Solves clipping, occlusion, and scroll-sync in one move; the primary escalation.
3. **Static-image fallback** — plotly via static export, HTML tables re-rendered natively from underlying data, with "open in browser" for interactivity. Sidesteps compositing entirely; the most solo-realistic floor.
4. **CEF offscreen rendering** — perfect compositing and a consistent engine, but a project-sized subsystem: ~170MB bundle, manual input plumbing (mouse/keyboard/IME/hit-testing), multi-process supervision, and per-helper codesign + notarization. Last resort.

Measurable claim: memory scales with *visible rich outputs*, not app size — benchmarked against the Electron incumbents from tier 2.

### Document model

The `nbformat` v3 crate parses; serialization goes through yoshi's canonical writer, because the crate preserves unknown fields (flattened maps) but not key order. The in-memory `NotebookModel` is the single source of truth; the UI subscribes to it. It carries the model-level undo/redo stack for structural cell operations.

The round-trip guarantee is **idempotent canonical round-trip**: yoshi normalizes to the same output `nbformat.writes` produces — 1-space indent, `ensure_ascii=False`, trailing newline, source as line-arrays, adjacent same-name streams merged — so the *first* save may reflow a notebook once, to the form any Jupyter tool emits, and every save thereafter is byte-stable (`write(read(write(read(x)))) == write(read(x))`). Unknown metadata is preserved at notebook, cell, and output level; v4.0 notebooks keep their minor version and do not get cell ids added on open (that would rewrite every cell). This is what actually protects git diffs and ecosystem trust; byte-identity against arbitrary producers is unattainable (Python's `ensure_ascii=True` escapes alone break it).

Autosave to a sidecar recovery file every 30s when dirty; never autosave over the user's file.

### Workspace shell

Post-MVP (E10): the single-notebook window grows into a workspace — a **pane grid** (split, resize, close, and Warp-style drag-a-header-to-swap; Zed's pane system is the architecture reference, GPL, read-only) and a **project tree** sidebar built on the `ignore` crate's gitignore-aware walker with `notify` file watching, rendered on the same virtualized list machinery as cells. Each pane hosts an independent notebook view with its own kernel session. Tree selections preview through the existing output pipeline: images via the native renderers, `video/mp4` via the sandboxed webview (system codecs — no VLC/ffmpeg dependency, which would add a ~100MB runtime for what `<video>` already does). Workspace layout persists across restarts.

Boundary: the workspace exists to open, arrange, and run notebooks — it is not a step toward a general editor/IDE. The tree opens `.ipynb` into panes and previews data files; it does not grow code-editing of arbitrary files.

### Terminal

Post-MVP (E11): a GPU-rendered terminal pane in the grid, sitting beside a notebook — the agent workflow (Claude Code operating on the notebook you're viewing) is the motivating use, and it pairs with the future MCP server. Engine: **`alacritty_terminal`** (Apache-2.0) for PTY and grid state, rendered as a native GPUI view; Zed's `crates/terminal` is the architecture reference (GPL, read-only). Warp is the deliberate UX reference — its drag-swap grid affordances and block-style polish are catalogued and ported by hand; warpui's MIT components are fair styling references, but no Warp code is extracted (E1's spike priced that path out: AGPL sibling webs, warpui coupling).

### Scope

**In scope (v0.1):** local Python kernels (ipykernel), single notebook window, tier-1 outputs, macOS + Linux, Jupyter keyboard parity, undo/redo.
**Scheduled post-MVP:** tier-2 webview outputs (E9), workspace shell — pane grid + project tree (E10), terminal (E11), remote kernels (E12, cheap due to transport abstraction).
**Future work (considered, not scheduled):** ipywidgets (needs comm protocol + a widget component library — the single largest deferred cost); find/replace; math in markdown; Windows (signing + CI cost defers it); a GUI settings page; picker auto-detection of unregistered venvs/conda envs (the managed kernel covers onboarding, so this waits for demand); session restore (reopen the last-open notebook on launch — E10 persists workspace layout, but single-window session restore is unscheduled).
**Out of scope:** being a general editor/IDE (Zed exists); headless execution (papermill exists); JupyterLab extension compatibility (structurally impossible without the web runtime — this is the price of native, stated openly).

---

## Project structure

```
yoshi/
├── crates/
│   ├── yoshi-app/        # application shell, views, keyboard, main binary
│   ├── yoshi-editor/     # cell text editor: buffer, selection, highlight, IME
│   ├── yoshi-ui-kit/     # shared widget library (status dot, pickers, renderer views)
│   ├── yoshi-notebook/   # document model, cell ops, canonical round-trip, undo stack
│   ├── yoshi-kernels/    # session actor, kernel lifecycle, transport trait (zmq now, ws later)
│   ├── yoshi-outputs/    # MIME ranking + renderers
│   └── yoshi-cli/        # arg parsing, `kernels list`
├── xtask/                # validate, bench, release helpers
├── golden/               # corpus notebooks + expected outputs
├── docs/write-ups/       # dated epic write-ups (append-only history)
└── script/               # bootstrap, run
```

Principles: the framework choice is load-bearing — view code is framework code, a migration would be a rewrite, and no façade can change that; churn is mitigated by pinning the dependency and bumping it on a schedule in isolated PRs, not by an abstraction layer. `yoshi-ui-kit` is a shared-widget component library, not an isolation boundary. The real seams are data/protocol boundaries and cheap to enforce: no ZeroMQ types above `yoshi-kernels`; `yoshi-notebook` has zero UI deps and is fully testable headless. Golden files are the test suite of record; `cargo deny` (allowlist: MIT/Apache/BSD/MPL/AGPL-3.0) plus the golden round-trip suite are the CI checks that matter most.

### Pipeline / runtime model

One UI thread (GPUI's app loop) + kernel I/O on a single async runtime: **no tokio in-process** — kernel I/O runs on GPUI's own executor (`BackgroundExecutor`/`ForegroundExecutor`) via `jupyter-zmq-client`'s `async-dispatcher-runtime` feature (verified present at runtimed HEAD during the E1 spike). Zed's repl demonstrates the production pattern: one spawned task per ZMQ socket, mpsc channels to the UI. tokio-alongside with channel hops is the fallback if the feature misbehaves.

The session actor owns all sockets; UI ↔ actor communication is channels only. Invariant that removes synchronization: cell outputs are only ever mutated by the single iopub consumer task, then published as immutable snapshots to the UI. Determinism rule: golden execution tests normalize msg ids, timestamps, and memory addresses in reprs before comparison.

---

## Distribution

macOS: signed + notarized `.dmg` via GitHub Releases + Homebrew **cask** in `oxmonty/homebrew-tap` (`brew install oxmonty/tap/yoshi` — macOS-only, since Homebrew on Linux doesn't support casks; the `yoshi` CLI is exposed via the cask's `binary` stanza). Linux: AppImage on GitHub Releases (one format for MVP; `.deb` is future). Signing and notarization land at v0.1 (E8); before that, releases ship unsigned artifacts with the Gatekeeper bypass documented — early adopters of an OSS dev tool tolerate right-click-Open. Release automation cuts all artifacts from one tag. No crates.io publication — yoshi is an app, and the name is taken there anyway.

**Naming note:** `yoshi` on crates.io is an existing error-handling framework (unavailable, but not needed); Wix's `yoshi` is an established JS build toolkit on npm/GitHub. Homebrew core has no `yoshi` formula or cask; a personal tap sidesteps collisions regardless. Real flag: **Yoshi is a Nintendo trademark** — fine for a personal OSS project, a liability if this grows a brand; decide before any public launch push whether to keep it (precedent: plenty of OSS uses game names quietly) or rename while cheap.

---

## CI/CD

**Quality gates (every PR, from E2):** `cargo fmt --check`, `clippy -D warnings`, `cargo deny check licenses`, nextest unit suite, golden round-trip suite, headless kernel integration test (CI installs python3 + ipykernel). Required: all. The license check and golden round-trip are the two that protect the project's core promises.

**Versioning:** semver via release-please (active from E2), conventional commits. Pre-MVP releases are a `0.1.0-alpha.N` prerelease series, one release-please PR merged per epic exit, until E8 cuts `v0.1.0`. Breaking = anything that changes on-disk notebook output or settings schema.

```
merge to main ──▶ release-please PR ──▶ tag vX.Y.Z
   ──▶ build matrix (macos-14 arm64+x86, ubuntu-24.04)
   ──▶ macOS: codesign + notarize (Developer ID via GH environment secrets; from E8)
   ──▶ GitHub Release (artifacts + checksums)
   ──▶ bump Homebrew tap cask (repo-dispatch; from E8)
```

**Secrets & signing:** Apple Developer ID cert + notarization keys in a protected GitHub environment scoped to the release workflow only; Linux artifacts checksummed and (future) sigstore-signed; no long-lived tokens where OIDC works (the tap bump uses a fine-grained PAT scoped to the tap repo — the one exception, documented).

---

## Additional design considerations

- **Crash isolation**: a kernel crash must never take the app down — session actor failures surface as a "kernel died" state with restart affordance; app panics trigger the autosave recovery path.
- **Startup discipline** (protects the <500ms claim): kernelspecs are read directly from disk, never by shelling out to `jupyter`; kernel launch is async and never blocks first paint; startup avoids system-font enumeration (bundle the font, load lazily); wgpu pipeline and font-atlas warmup happen off the critical path. First-ever launch (cold shader/fontconfig caches) is slower than the benchmarked warm-cache number — the benchmark states its cache assumptions.
- **Telemetry stance**: none in MVP; tier-2 adds *opt-in, local-only* logging of unrenderable MIME types to guide renderer priorities. Stated in README — it's a differentiator against Warp's own telemetry reputation.
- **Exit contract**: CLI exits 0/1/2 (ok / file error / kernel error); the app never blocks quit on a busy kernel — it interrupts, waits 2s, kills.
- **App identity & menus**: yoshi ships as a real macOS app bundle — Info.plist with bundle id `com.oxmonty.yoshi`, version synced from Cargo, `.icns` icon (placeholder until the E8 branding pass) — and a native menu bar via GPUI's `cx.set_menus` (Zed's mechanism): File/Edit/Window menus route the same actions as the keyboard shortcuts, because macOS users discover features through the menu bar. Linux gets a `.desktop` entry + icon inside the AppImage. The logo asset itself is an open need before E8's branding story.
- **Settings & themes**: plain JSON files, Zed's pattern — `~/.config/yoshi/settings.json` (editor settings, font, default kernel, active theme) and `keymap.json` (keybinding overrides), defaults written on first run, opened via a menu command for hand-editing; no settings GUI until post-MVP. Themes ship as three built-ins persisted in settings: **Gruvbox Dark Soft** (default), Gruvbox Light, One Dark; the terminal (E11) and ANSI renderer draw from the active theme's palette.
- **Font/rendering**: bundle one good monospace (JetBrains Mono or Geist Mono) so golden output is deterministic across machines.
- **Upstream hygiene**: framework bumps happen on a schedule (monthly), each in an isolated PR with the full validation suite — never alongside feature work.

## Competitive landscape

**JupyterLab Desktop** (the incumbent everyone has — now abandoned): Electron wrapping the full JupyterLab web app, and per its own README unmaintained since August 2025 with no security fixes. Seconds-long cold start, heavy memory; its strength (full compatibility, extensions, ipywidgets) is exactly what yoshi trades away for speed — and its abandonment is the market opening. **nteract Desktop** (the maintained comparator and the real benchmark target): Tauri + React on the same runtimed crates yoshi uses (`jupyter-protocol`, the `runtimed` daemon), with realtime sync, Automerge docs, and an MCP server for agents (`runt mcp`) — genuinely good, but still a webview rendering React. **Zed's notebooks**: native and fast, but notebooks are a side feature of an editor, not a product. The gap: nobody ships a *notebook-first* desktop app with a fully native shell — where the browser engine appears only inside the outputs that are actually web content, instead of hosting the entire application. The claim to defend against nteract: cold start and scroll on notebooks with few or no rich outputs, where yoshi runs zero webviews and nteract runs one for the whole app. The pitch line: **"Your notebook opens before JupyterLab's splash screen renders."** Migration is free — it's the same `.ipynb` on disk and the same kernels; the candidate command is just `yoshi existing-notebook.ipynb`.

## Tech stack

- **Rust** — the entire viable stack (frameworks, jupyter crates) is Rust; matches the performance claim.
- **GPUI 0.2.2** (Apache-2.0, crates.io; its API tracks Zed HEAD closely, and Zed's repl proves the kernel integration) — chosen in the E1 bake-off, 2026-07-16 (see Open questions for the evidence).
- **jupyter-zmq-client** v1 (kernel transport; the renamed runtimelib), **jupyter-protocol** v2 (message types, transport-agnostic), **jupyter-websocket-client** v2 (E12), **nbformat** v3 (parse; yoshi owns canonical serialization) — all BSD-3-Clause, runtimed org.
- **helix-core** (MPL-2.0) + **cosmic-text** (MIT/Apache) — cell-editor baseline; **Warp `warp_editor` / `ipynb_parser`** (AGPL) — extraction candidates evaluated as spike-time bonuses only.
- **wry** (per-output webviews, tier 2; `wgpu-scry` → static-image → CEF-OSR escalation ladder), **async-dispatcher** (kernel I/O on GPUI's executor, no tokio in-process — validated in E1), **zeromq** pure-Rust (transitive), **syntect or tree-sitter** (highlighting), **resvg** (tier-2 SVG), **clap** (CLI).
- **uv** (bundled binary, MIT/Apache — provisions the managed default kernel on first launch).
- Post-MVP: **ignore** + **notify** (E10 project tree), **alacritty_terminal** (Apache-2.0, E11 terminal engine).

## Reference codebases

| Project | Lesson |
|---|---|
| `zed-industries/zed` → `crates/repl`, `crates/terminal`, pane system | The blueprint: MIME ranking, per-type output views, kernel-channel task architecture, `RunningKernel` trait (E1/E3/E7); terminal-on-GPUI and pane/dock architecture (E10/E11). GPL — architecture reference only. |
| `warpdotdev/warp` → `crates/ipynb_parser`, block crates, terminal UX | The AGPL `ipynb_parser` is still an E4 reuse candidate (framework-independent); the block-UI crates and the drag-swap grid are the deliberate UX references for E10/E11 (warpui's MIT components are fair styling references, ported by hand). The warpui framework and `warp_editor` were priced out in the E1 spike (`spikes/warpui-spike/CAPTURES.md`). |
| `nteract/desktop` | The maintained competitor: daemon design, MCP tool surface, iframe isolation for untrusted outputs, Automerge sync. Consult for tier-2/Future scoping. |
| `runtimed/runtimed` (+ `sidecar` example) | Canonical `jupyter-zmq-client` usage patterns; kernel-compat quirks live in its issue tracker. |
| `jupyterlab/jupyterlab-desktop` | Gap analysis + what users expect from menus/session management; also what *not* to rebuild. |

## License

**AGPL-3.0** (decided 2026-07-16, before any outside contributions; the repo's LICENSE file carries the full text). Rationale: yoshi is an open-source app, not a library — AGPL costs its users nothing and guarantees improvements flow back even from hosted forks. It also keeps open the option of reusing Warp's AGPL crates (`warp_editor`, `ipynb_parser`, block UI) should extraction ever prove cheap — a bonus, not a plan. Compatibility posture: dependencies may be MIT/Apache/BSD/MPL/AGPL-3.0; GPL-3.0 code (e.g. Zed's editor) is legally combinable with AGPL-3.0 but stays off the table for extraction-cost reasons. `cargo deny` enforces the allowlist. Contributors license under AGPL-3.0 (inbound = outbound, no CLA). Outputs (notebooks users create) are theirs; no trademark claim on "yoshi" is possible (see naming note). If a genuinely standalone library ever falls out of this (e.g. the cell-editor crate), consider dual-licensing *that crate* permissively at extraction time — decided per-crate, not now.

## Open questions

- **warpui vs GPUI — which framework?** (**Resolved 2026-07-16: GPUI**, by head-to-head spike, decisively on all four criteria. GPUI 0.2.2 from crates.io built and ran first try with no AGPL/GPL in its 422-package tree, ships real input primitives + IME + AccessKit, its API tracks Zed HEAD, and it exposes the native `NSView` handle E9's overlay path needs. warpui failed to build standalone without hand-replicating Warp's internal `[patch.crates-io]` table, pulled six AGPL warp-internal crates for a bare window+list, offers no editing model in its `TextInput`, and exposes no raw-window-handle. Evidence: `spikes/*/CAPTURES.md`.)
- **What async runtime does GPUI's event loop expect?** (**Resolved 2026-07-16**: single runtime — kernel I/O on GPUI's executor via `jupyter-zmq-client`'s `async-dispatcher-runtime` feature, verified present at runtimed HEAD during the spike; Zed's repl demonstrates the pattern in production. tokio-alongside remains the fallback. Folded into [Pipeline / runtime model](#pipeline--runtime-model).)
- **Is `warpui_extras` MIT or AGPL?** (**Resolved 2026-07-16: AGPL** — its Cargo.toml inherits the workspace license and it is not in the MIT exception. Moot for yoshi since GPUI won the bake-off.)
- **Where does the code editor widget come from?** (**Resolved 2026-07-16**: assemble from helix-core + cosmic-text on GPUI's input primitives, budgeted as epic E5. The E1 spike closed the alternatives: GPUI ships primitives but no editor widget, and Warp's `warp_editor` is terminal-shaped, warpui-coupled, and drags seven AGPL siblings; Zed's GPL editor stays out on extraction cost. Residual: which of helix-core's layers to take vs. reimplement — priced during E5's first story.)
- **License stance.** (**Resolved 2026-07-16: AGPL-3.0**, chosen before any outside contributions; LICENSE file replaced the repo's initial MIT text the same day. Dependency allowlist MIT/Apache/BSD/MPL/AGPL-3.0 via `cargo deny`. Folded into [License](#license).)
- **Webview policy.** (**Revised 2026-07-16: hybrid — native-first shell with sandboxed, virtualized per-output webviews**, with the escalation ladder wry-overlay → `wgpu-scry` → static-image fallback → CEF-OSR. Research finding that shaped it: wry child views are X11-only on Linux, so the E9 spike probes Wayland/GTK embedding first, and `wgpu-scry` displaced CEF as the primary escalation. Supersedes the earlier "never" decision same-day. Folded into [Output rendering](#output-rendering).)
- **Is "native shell, webview only inside outputs" enough differentiation against nteract Desktop?** (Blocks the positioning paragraph staying honest — sharper now that JupyterLab Desktop is unmaintained and nteract shares yoshi's runtimed stack. Resolves: E8 benchmark numbers vs nteract, not just JupyterLab Desktop — the claim to defend is cold start and scroll on notebooks with few or no rich outputs.)
- **How do users get a working kernel without env pain?** (**Resolved 2026-07-17: managed default kernel via bundled `uv`** — provisioned outside the app bundle on first launch, listed first in the picker, kernelspecs below. Literally bundling CPython was rejected for ~100MB of per-dylib signing and update-wipe risk; kernelspec-only was rejected because first-run would require an existing Jupyter setup. Env auto-detection deferred to Future — the managed kernel covers onboarding. Folded into [Managed default kernel](#managed-default-kernel).)
- **Keep the name yoshi?** (Blocks nothing until public launch; decide before v0.2 marketing. Collisions on record: crates.io error framework, Wix's JS build toolkit, and the Nintendo trademark. Resolves: a 30-minute trademark-risk read + gut check; rename cost is near-zero pre-launch.)
