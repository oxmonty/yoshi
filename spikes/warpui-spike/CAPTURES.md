# Spike A: warpui + warpui_core — captures

Pinned commit: `9fd88b6fe91733183651200524b173efdcbd5108` (short form `9fd88b6fe917`) of
https://github.com/warpdotdev/warp

## 1. Build result

**Builds standalone, but only after adding a `[patch.crates-io]` table upstream doesn't
ship separately.** `cargo build` initially failed with 8 (later 20, before a lockfile
regen) `E0308` errors, all the same root cause:

```
error[E0308]: mismatched types
   --> .../font-kit-.../src/loaders/core_text.rs:607:44
    |
607 |             core_graphics_context.set_font(&self.core_text_font.copy_to_CGFont());
    |                                   -------- ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected `core_graphics::font::CGFont`, found a different `core_graphics::font::CGFont`
    |
note: there are multiple different versions of crate `core_graphics` in the dependency graph
error: could not compile `font-kit` (lib) due to 8 previous errors
```

Root cause: `warpui`'s macOS deps directly pin `core-graphics = "0.25.0"`, while `cocoa =
"=0.26.0"` and the crates.io `core-text = "21.0.0"` pull in `core-graphics = "0.24"` —
two 0.x-incompatible versions in the same graph, and a font-kit function mixes `CGFont`
values from both. Upstream warp's own root `Cargo.toml` carries a
`[patch.crates-io]` table (patching `core-foundation`, `core-graphics`, `core-text`,
`objc`, `pathfinder_simd` to a unified `servo/core-foundation-rs` fork commit) that
fixes exactly this — but `[patch]` tables are workspace-scoped and do **not** propagate
to a downstream consumer pulling `warpui` in as a plain git dependency. Copying that same
patch table into this spike's own `Cargo.toml` (see `Cargo.toml`, `[patch.crates-io]`),
then regenerating the lockfile from scratch (`cargo generate-lockfile` — reusing the
first, unpatched lockfile did **not** pick up the patch and made things worse), resolved
it. `cargo build` then finished clean:

```
Finished `dev` profile [unoptimized + debuginfo] target(s) in 22.16s
```

Ran the resulting binary for 3s in the background; it stayed alive with no panic/stderr
output (a real interactive check — window renders, scrolls smoothly, IME/clipboard — is
left for a human, per the roadmap's own note).

**Assessment**: "builds standalone with a clean license tree" (bake-off criterion 1) is
only half-true. It *does* build standalone — but only by an integrator finding,
understanding, and manually replicating an internal patch table that upstream doesn't
publish or version as part of the crate's own contract. That's a real ongoing
maintenance cost: every time warpui bumps its pin on `font-kit`/`cocoa`/`core-text`, this
patch table may need to change in lockstep, silently, with no upstream signal.

## 2. `cargo tree`

```
$ cargo tree -e normal --depth 1
warpui-spike v0.0.0 (/Users/pprunty/GitHub/oxmonty/yoshi/spikes/warpui-spike)
├── anyhow v1.0.103
├── warpui v0.0.0 (https://github.com/warpdotdev/warp?rev=9fd88b6fe917#9fd88b6f)
└── warpui_core v0.1.0 (https://github.com/warpdotdev/warp?rev=9fd88b6fe917#9fd88b6f)
```

Total unique packages in the full dependency graph (`cargo tree -e normal`, deduped):
**296**.

## 3. Licenses

`warpui` and `warpui_core` are both `license = "MIT"` explicitly in their own
`Cargo.toml` (overriding the workspace default of `AGPL-3.0-only`) — but **that doesn't
make the dependency tree MIT**. The actual build graph pulls in 6 more warp-internal
sibling crates that all inherit the workspace's `AGPL-3.0-only` default:

```
$ cargo tree -e normal --prefix none | grep 'warpdotdev/warp?rev' | sed -E 's/^([a-zA-Z0-9_-]+) .*/\1/' | sort -u
command          # AGPL-3.0-only
markdown_parser  # AGPL-3.0-only
string-offset    # AGPL-3.0-only
sum_tree         # AGPL-3.0-only
warp_errors      # AGPL-3.0-only
warp_util        # AGPL-3.0-only
warpui           # MIT
warpui_core      # MIT
```

**This is the single most decision-relevant finding of this spike.** Building nothing
more than "a window with a scrollable list" already statically links 6 AGPL-3.0-only
warp-internal crates. If yoshi ships as anything other than AGPL-3.0 itself, this
directly fails bake-off criterion 1 ("builds standalone with a clean license tree") as
currently pinned — it isn't a hypothetical risk confined to reusing `warp_editor` later
(§7), it's already true of the minimal spike.

`cargo deny check licenses` (allowlist: MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause,
MPL-2.0, AGPL-3.0, Unicode-3.0, ISC, Zlib — as specified). Two config notes up front:
this spike's own package needed an explicit `license = "MIT"` field (cargo-deny fails
unlicensed crates), and the manifests here declare `AGPL-3.0-only`, not the bare
`AGPL-3.0` in the requested allowlist — SPDX treats these as different identifiers, so
`AGPL-3.0-only` was added alongside `AGPL-3.0` in `deny.toml` to actually match. With
those two fixed, three real rejections remain against the given allowlist:

| License | Crate | Pulled in via | Note |
|---|---|---|---|
| `CC0-1.0` | `hexf-parse` | `naga` ← `wgpu` ← `warpui` | Runtime dep of wgpu's shader translator, ships in the final binary |
| `0BSD` | `enum-iterator-derive` | `enum-iterator` ← `markdown_parser`/`warpui_core` | Proc-macro, compile-time only — doesn't link into the shipped binary |
| `BSL-1.0` | `clipboard-win`, `error-code` | `arboard` ← `warpui`/`warpui_core` | Windows-only (`cfg(windows)`); irrelevant for yoshi's macOS/Linux targets |

Also one **unresolved warning**, not a hard failure: `bounded-vec-deque`
(→ `warpui_core`) declares `license = "GPL-3.0+ OR BSD-3-Clause"` using the deprecated
`GPL-3.0+` SPDX identifier, which cargo-deny can't parse (`parse-error`, not
`rejected`) — so its actual license status is unverified by this tool, not confirmed
clean. Worth a manual look before relying on the BSD-3-Clause side of that OR.

**Net**: after accounting for platform-gating and proc-macro-vs-linked distinctions, the
practically-relevant new license gap is just `CC0-1.0` via `hexf-parse` (small, unusual,
easy to allowlist or vendor around) — but the AGPL-3.0-only warp-internal crates above
are the real, unavoidable finding.

## 4. Text-input primitive inventory

Grepped `crates/warpui/src` and `crates/warpui_core/src` for `TextInput`/`Editor`/
`TextField`/input-handling types.

- **`warpui_core::ui_components::text_input::TextInput<T: View>`** (`crates/warpui_core/src/ui_components/text_input.rs`) —
  the only `TextInput`-named type in either crate, and it is **not** a text editor. It's
  a presentational wrapper: it takes an already-built child `View` (`T`) and wraps it in
  a bordered/padded/backgrounded `Container` (`ChildView` + `Clipped` + `ConstrainedBox`).
  All cursor, selection, and keystroke-handling logic is expected to live in the
  `T: View` you hand it — there is no built-in cursor/selection/undo model anywhere in
  `warpui`/`warpui_core`.
- **`warpui::platform::wasm::hidden_input::HiddenInput`** — a wasm-only helper (an
  offscreen `<input>` DOM element routing IME/composition events into the canvas); not a
  general-purpose editor widget and not available on macOS/Linux.
- No `TextField`, no standalone rope/buffer/cursor type, no syntax-highlighting hook, in
  either crate.
- **Assessment**: warpui ships no usable text-editing primitive by itself. Real text
  editing in Warp lives in the separate, AGPL-licensed `crates/editor` (`warp_editor`)
  crate — not part of `warpui`/`warpui_core`, and pulling it in drags in the entanglement
  described in §7. Directly affects E5 (cell editor): if warpui wins the bake-off, the
  cell editor's cursor/selection/rope/highlighting has to be built from scratch, since
  the AGPL cost of reusing `warp_editor` (§7) is on top of the license issue already
  present in §3.

## 5. Native view/window handle exposure

- `raw-window-handle` (0.6.2) *is* a dependency of `warpui_core` and is used
  internally — e.g. `crates/warpui_core/src/windowing/system.rs` matches on
  `RawDisplayHandle` variants (`AppKit`, `Wayland`, `Xlib`/`Xcb`, `Windows`) to detect the
  windowing system, and the non-macOS (winit) rendering backend
  (`crates/warpui/src/rendering/wgpu/*`, `crates/warpui/src/windowing/winit/window.rs`)
  uses it to hand a surface target to `wgpu`.
- On macOS specifically, `NSView` appears in the custom AppKit backend
  (`crates/warpui/src/platform/mac/window.rs`, `rendering/renderer.rs`,
  `objc/host_view.{h,m}`, `objc/window.m`) — but only as an internal implementation
  detail of warpui's own Metal renderer.
- **No public API was found** in either crate that hands the embedding application a raw
  window/view handle to attach something of its own to (no `fn ns_view(&self) -> …` or
  `fn raw_window_handle(&self) -> …` reachable from app code).
- **Assessment**: this gates E9 (webview overlay for `text/html`/plotly outputs) — as of
  this commit, warpui does not appear to expose the hook E9's overlay approach would
  need. Worth confirming with upstream or a deeper read of `core/window.rs` before ruling
  it out entirely, but nothing in the public surface suggests it today.

## 6. Async runtime (app loop)

`crates/warpui_core/src/async/native/executor.rs`:
- **Foreground/UI task queue**: `async_executor::LocalExecutor` (single-threaded,
  `!Send`/`!Sync` — pinned to the platform's main thread/event loop). This is what
  `View`/`Element` code runs on.
- **Background tasks**: spawned as `tokio::task::JoinHandle` (wrapped in a
  `BackgroundTask` type) — tokio is already present and used for background work, just
  not for the foreground UI loop itself.
- **Assessment**: good news for E1's riskiest integration (framework event loop + kernel
  I/O runtime coexisting). A tokio-based kernel client (`jupyter-zmq-client`) plausibly
  slots into warpui's existing background-task tokio integration rather than requiring a
  second bolted-on runtime — worth validating directly in the "Hello world on the
  winner" story.

## 7. AGPL entanglement estimate: `warp_editor` (`crates/editor`) + `ipynb_parser`

Read both crates' `Cargo.toml` at the pinned commit; workspace default license is
`AGPL-3.0-only`, and neither crate nor its warp-internal dependencies below override
that (only `warpui`/`warpui_core`/`warpui_extras` are explicitly `license = "MIT"`).

Direct warp-internal sibling crates (`crates/*` in the same repo) pulled in one level
deep by `warp_editor` + `ipynb_parser` combined:

| Crate | Pulled in by | License |
|---|---|---|
| `asset_cache` | `warp_editor` | AGPL-3.0-only (workspace default) |
| `ipynb_parser` | `warp_editor` (also one of the two crates in question) | AGPL-3.0-only |
| `markdown_parser` | `warp_editor`, `ipynb_parser` | AGPL-3.0-only |
| `warp_core` | `warp_editor` | AGPL-3.0-only |
| `warp_errors` | `warp_editor` | AGPL-3.0-only |
| `warp_util` | `warp_editor` | AGPL-3.0-only |
| `vim` | `warp_editor` | AGPL-3.0-only |
| `warpui_core` | `warp_editor` | MIT (already a dependency either way) |

**Count: 8 warp-internal sibling crates** (7 AGPL-3.0-only; `warpui_core` is already
pulled in regardless — and per §3, `markdown_parser`/`warp_errors`/`warp_util` are
*already* in the graph even without touching `warp_editor` at all). `warp_editor` also
depends on an external git fork (`mermaid_to_svg`) and ordinary crates.io deps
(`urlocator`, `line-ending`, `html5ever`, `imara-diff`, etc.) that carry no extraction
cost beyond the usual dependency count.

**Assessment**: reusing `warp_editor`/`ipynb_parser` means importing 7 AGPL-3.0-only
crates on top of the 6 already implied by using `warpui` at all (§3). For a project not
itself willing to ship under AGPL (network-copyleft, viral to anything statically linked
into the same binary), this is a hard no unless yoshi itself goes AGPL. Treat any reuse
of `ipynb_parser`/`warp_editor` as a licensing decision, not just an engineering one.

## 8. IME / accessibility (presence/absence only — a human should test interactively)

- **IME**: real marked-text/composition handling exists — hits in
  `crates/warpui/src/platform/mac/window.rs`,
  `crates/warpui/src/platform/mac/objc/host_view.{h,m}` (AppKit `NSTextInputClient`-style
  marked-text/insertText plumbing), `crates/warpui/src/windowing/winit/event_loop/mod.rs`
  (winit `Ime` events, non-macOS path), and `crates/warpui_core/src/event.rs` (a
  cross-platform IME event type). IME plumbing is present on both the macOS-native and
  winit backends at the event-routing layer — whether any given text widget you build
  wires it up correctly is a separate, interactive question.
- **Accessibility**: no `accesskit` crate dependency anywhere in either `Cargo.toml`.
  Instead there's a bespoke system: `crates/warpui_core/src/accessibility.rs` defines an
  `AccessibilityContent` struct (a hand-rolled "VoiceOver announcement" content model),
  documented in-file as intentionally *not* using AppKit's `NSAccessibility` APIs
  directly. Accessibility-adjacent code also touches `platform/mac/{window,delegate}.rs`,
  `windowing/winit/delegate.rs`, `platform/headless/delegate.rs`, and several `core/*`
  files. Presence confirmed; actual VoiceOver/screen-reader quality needs a human to run
  it live (as the roadmap assumes).

## 9. Upstream reference materials used

- `crates/warpui/examples/list/{main.rs,root_view.rs}` — the direct model for this
  spike; already builds exactly the "1000-item scrollable list" shape needed.
- `crates/warpui_core/src/assets/mod.rs` — `impl AssetProvider for ()`, the built-in
  no-op asset provider this spike uses (no bundled assets needed for a plain list).
- Root `Cargo.toml` — workspace member list, `[workspace.dependencies]` version pins,
  `[workspace.package] license = "AGPL-3.0-only"`, and the `[patch.crates-io]` table
  this spike had to replicate (see §1).
- `crates/warpui/Cargo.toml`, `crates/warpui_core/Cargo.toml` — dependency lists,
  `license = "MIT"` override, macOS-specific deps (objc2 family, Metal shader build via
  `build.rs`).
- `.cargo/config.toml` — sets `MACOSX_DEPLOYMENT_TARGET = "10.14"`, required by
  `warpui`'s `build.rs` (Metal shader compilation panics without it); this repo's own
  `.cargo/config.toml` mirrors that since it doesn't apply outside the upstream repo.
