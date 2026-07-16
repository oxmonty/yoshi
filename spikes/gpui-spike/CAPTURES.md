# Spike B: GPUI — captures

Standalone crate at `spikes/gpui-spike/` (own `[workspace]`, not part of the root workspace).
App: a native window with a `uniform_list` of 1000 rows (`src/main.rs`).

## Build result

`gpui = "0.2.2"` from crates.io built and ran cleanly on the first try. No fallback to a
git dependency on zed-industries/zed was needed.

- `cargo build`: clean, 2m09s cold (pulls in wgpu/naga/blade, image codecs, resvg/usvg, etc.)
- `cargo run`: opens an 800x600 window, renders 1000 rows via `uniform_list`, no panics.
  Confirmed via a 3s background run (log shows AppKit's IMKClient/IMKInputSession
  initializing — the window did acquire real keyboard/IME focus, not a headless stub).
- The published crate's public API (`Application::new().run(...)`, `cx.open_window`,
  `cx.processor`, `uniform_list`) matches current zed HEAD almost exactly — only needed
  one fix (an explicit `Range<usize>` type on the `cx.processor` closure param, which HEAD
  infers or has a slightly different signature for). This is a good sign for API stability
  even across the "9 months stale" gap: the 0.2.2 API is not a fossil, it tracks HEAD closely.

## Dependency tree

`cargo tree -e normal --depth 1`:
```
gpui-spike v0.0.0 (/Users/pprunty/GitHub/oxmonty/yoshi/spikes/gpui-spike)
└── gpui v0.2.2
```

Full graph size (this platform, normal deps only, de-duplicated): **422 unique packages**.
(`cargo metadata` reports 703 packages total across all platforms/features in the lock file;
422 is what's actually compiled on macOS/aarch64 for a normal, non-dev, non-test build.)
This is a heavy dependency tree — GPUI bundles its own renderer (wgpu/naga/blade), image
decoding (image, resvg/usvg, ravif, tiff, png, gif, exr, qoi...), font shaping, and more.
gpui itself is Apache-2.0.

## Licensing (`cargo deny check licenses`)

cargo-deny 0.20.2 installed (`cargo install cargo-deny`). Allowlist: MIT, Apache-2.0,
BSD-2-Clause, BSD-3-Clause, MPL-2.0, AGPL-3.0-only, Unicode-3.0, ISC, Zlib.

Result: **FAILED** — 4 transitive licenses rejected, all several hops deep, none of them
gpui's own or a direct dependency:

| Crate | License | Path |
|---|---|---|
| `ar_archive_writer` 0.5.2 | Apache-2.0 WITH LLVM-exception | (build) `psm` → `stacker` → `stacksafe` → `gpui` |
| `hexf-parse` 0.2.1 | CC0-1.0 | `naga` → `blade-graphics`/`blade-util` → `gpui` |
| `libfuzzer-sys` 0.4.13 | (MIT OR Apache-2.0) AND NCSA | `rav1e` → `ravif` → `image` → `gpui` |
| `tiny-keccak` 2.0.2 | CC0-1.0 | `const-random` → `ahash` → `zed-xim` → `gpui` |

All four are permissive-adjacent (CC0-1.0 public domain dedication, an Apache-2.0 variant
with an LLVM linking exception, and an OSI-approved NCSA clause) — none are copyleft or
otherwise concerning, they just weren't on the literal allowlist string match. Practical
fix would be adding those three exact SPDX strings to the allowlist; flagging here rather
than silently doing it since it's a policy call, not a build blocker. No AGPL, no GPL, no
SSPL anywhere in the tree.

## Text-input primitive inventory

GPUI ships **no ready-made `TextInput` widget**. What it does provide, per
`crates/gpui/examples/input.rs` (778 lines) and `crates/gpui/src/platform.rs`:

- `EntityInputHandler` trait — the hook an app implements (`replace_text_in_range`,
  `selected_text_range`, `marked_text_range`, `bounds_for_range`, etc.) so GPUI's platform
  layer can drive IME composition and OS text services against your own buffer.
- `ElementInputHandler` — bridges an `Element`'s paint/layout pass to the above.
- `FocusHandle` / `Focusable` — focus routing so a text field can receive key events.
- `ShapedLine` — precomputed text shaping/layout the app uses to paint the caret and
  selection itself.
- `marked_range: Option<Range<usize>>` on the example's own struct — IME preedit state is
  application-owned, not framework-owned.

In short: GPUI gives you the *plumbing* (focus, input-method callbacks, text shaping
primitives, clipboard, key bindings via `actions!`), but the entire text buffer, caret
math, selection model, and rendering in `input.rs` (Unicode-segmentation-aware cursor
movement, mouse-drag selection, etc.) is ~700 lines of example code you'd otherwise have
to write yourself. This directly bears on E5 (cell editor): expect to build the editor
on these primitives, not on a reusable widget.

Zed's real code editor (`crates/editor`, GPL) is a separate crate built on these same
primitives — not assessed here, per scope.

## Native view/window handle (for E9's overlay path)

Confirmed present and real, not aspirational:

- `crates/gpui/src/window.rs`: `Window` implements `raw_window_handle::HasWindowHandle` /
  `HasDisplayHandle` — `fn window_handle(&self) -> Result<raw_window_handle::WindowHandle<'_>, HandleError>`.
- `crates/gpui/src/platform.rs`: `pub trait PlatformWindow: HasWindowHandle + HasDisplayHandle`.
- `crates/gpui_macos/src/window.rs`: the macOS backend's raw handle is literally
  "a wrapper around a pointer to an NSView" (comment at the unsafe conversion site); the
  window's content view (`GPUIView`) is a real `NSView` subclass.

This gates E9 cleanly: a child `NSView`/`HWND`/X11 window for an embedded webview can be
attached via the standard `raw-window-handle` crate, the same mechanism `wry`/`wgpu`
consumers already expect.

## Async runtime model

- GPUI's own executor (`crates/gpui/src/executor.rs`) is built on `async-task` (its
  `Cargo.toml` depends on `async-task = "4.7"`), exposing `BackgroundExecutor` and
  `ForegroundExecutor` types. It does not itself depend on tokio, async-std, or smol.
- `jupyter-zmq-client`'s `Cargo.toml` (runtimed/runtimed @ HEAD) confirms an
  `async-dispatcher-runtime` feature exists: `["zeromq/async-dispatcher-runtime",
  "async-dispatcher", "async-std", "smol"]`, as an alternative to its `tokio-runtime`
  feature.
- `async-dispatcher` (crates.io, published by the Zed org) is exactly what its
  description says: "async runtime based on a pluggable dispatcher" — a thin abstraction
  that lets `async`/`await` code run on whatever executor is plugged in behind it,
  instead of hard-requiring tokio.
- Net finding: the single-runtime path is real, not hypothetical. GPUI's executor plus
  `jupyter-zmq-client`'s `async-dispatcher-runtime` feature is the intended way to drive
  kernel I/O on GPUI's own event loop without pulling in a second runtime (tokio) — you
  implement/plug a `Dispatcher` that forwards onto GPUI's `BackgroundExecutor`. This is
  the validation the roadmap's tie-breaker rationale assumes.

## Zed `crates/repl` at HEAD — kernel-channel tasks on gpui's executor

(`crates/repl` ships under `LICENSE-GPL` — reference only, not reusable.)

`kernels/native_kernel.rs` opens four ZMQ sockets (iopub, shell, control, stdin) via
`runtimelib`, then calls `start_kernel_tasks(...)` which hands them to `cx.spawn` tasks —
inbound iopub/shell/control messages get routed into the session state via `cx` update
callbacks, outbound requests go through `mpsc::Sender<JupyterMessage>` (`request_tx`,
`stdin_tx`) that the UI holds. Separate `cx.spawn` tasks (detached) tail the kernel
subprocess's stdout/stderr into the logger and watch `process.status()` for kernel death.
Everything is `cx.spawn`/`cx.spawn_in(window, ...)` — no tokio, no separate thread pool;
it's all on GPUI's own async executor, which is the pattern to copy for `yoshi-kernels`.

## IME / accessibility (presence only — human to test interactively)

- **IME**: real, not stubbed. `gpui_macos/src/window.rs`'s `GPUIView` implements
  `NSTextInputClient`-style selectors directly: `has_marked_text`, `markedRange`,
  `setMarkedText:selectedRange:replacementRange:`. `marked_range` also appears in
  `gpui/src/platform.rs` and the `input.rs` example (`marked_range: Option<Range<usize>>`
  on the app-owned text-input struct) — preedit plumbing exists end to end.
- **Accessibility**: present as a first-class subsystem, not bolted on: `gpui/src/
  _accessibility.rs`, `gpui/src/window/a11y.rs`, `gpui/src/window/a11y/debug.rs`, plus
  hooks in `element.rs`, `app.rs`, `div.rs`, and the platform backends
  (`gpui_windows/src/window.rs`, `gpui_windows/src/events.rs`). "AccessKit" appears
  30 times across the tree, including a dedicated debug view. No claim about actual
  screen-reader UX here — that needs a human running VoiceOver/NVDA against a real
  window, which this spike didn't do.

## Three most decision-relevant findings

1. **crates.io `gpui = "0.2.2"` just works** — no git-dependency fallback needed, and its
   API already matches zed HEAD closely (one signature nit). The "9 months stale" framing
   in the spike brief overstates the risk for now, though it's still a distribution risk
   to watch (a solo maintainer pinned to crates.io is at the mercy of the Zed team's
   release cadence for API changes made after 0.2.2).
2. **Text input is BYO, not a widget** — no `TextInput` component ships; GPUI hands you
   `EntityInputHandler`/`ElementInputHandler`/`FocusHandle`/`ShapedLine` and you write the
   ~700-line editor yourself (as `examples/input.rs` does). E5 should budget for that as
   real, first-party work, not integration of an existing component.
3. **The single-runtime kernel path is concretely validated**: GPUI's `async-task`-based
   executor + `jupyter-zmq-client`'s `async-dispatcher-runtime` feature + the
   `async-dispatcher` pluggable-runtime crate is a real, coherent path to run kernel I/O
   without a second async runtime in the process, and Zed's own `crates/repl` proves the
   `cx.spawn`-per-socket pattern works in production.

## Interactive checks (human, 2026-07-16)

- Scroll feel, 1000-row uniform_list, macOS trackpad: fast momentum scrolling smooth, no stutter or tearing. PASS.
- IME (CJK) + clipboard round-trip: pending — requires the `input` binary (src/bin/input.rs).
