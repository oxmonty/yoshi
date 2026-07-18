# E2: CI + unsigned artifacts (walking skeleton)

**Date:** 2026-07-18 (single session)
**Status:** stories 1–5 done; epic box held open pending the human right-click-Open demo on a real Mac (the one claim CI cannot make)

## What shipped

- Quality-gate CI matrix (macos-14, ubuntu-24.04): fmt, clippy `-D warnings`, `cargo deny`, nextest, and the headless kernel round-trip — green on the first ubuntu run, which was also the Linux evidence that closed E1's epic box.
- **`v0.1.0-alpha.1` is live on GitHub Releases** with five assets: zipped `.app` for arm64 and x86_64, an AppImage, and per-platform SHA256SUMS. release-please owns versioning from here — one release PR merged per epic exit, `Release-As` footer forced the first version.
- App identity: `yoshi.app` with `com.oxmonty.yoshi`, version synced from Cargo (numeric `CFBundleVersion`, full prerelease string in `CFBundleShortVersionString`), user-provided halo icon; AppImage carries `.desktop` + icon.
- CLI surface: `yoshi --version`, `yoshi kernels list` (venv + user + system kernelspec discovery, reusing `jupyter-zmq-client`'s dir resolution and `KernelspecDir`), `yoshi <file>` accepted for E6.
- Every release artifact is smoke-tested *as an artifact* before upload: unpacked on the runner, `--version` run, and `kernels list` must find a freshly registered kernelspec.
- Upgrade design recorded for E8 (biscuit's two-channel stable/next model, checksum-verified self-swap, brew delegation) after a comparative read of `../biscuit`.

## Evidence

- CI matrix green: https://github.com/oxmonty/yoshi/actions/runs/29640874283
- Release run green (both artifact jobs + smoke tests): https://github.com/oxmonty/yoshi/actions/runs/29639688162
- Release: https://github.com/oxmonty/yoshi/releases/tag/v0.1.0-alpha.1
- Fresh public download, `shasum -a 256 -c` against published checksums, binary runs: `yoshi 0.1.0-alpha.1`.

## Decisions made along the way

- **release-please from E2, not manual tags until E8** (user call, mid-kickoff): the walking skeleton should exercise the real pipeline. First version pinned with a `Release-As: 0.1.0-alpha.1` footer; `versioning: prerelease` carries the alpha series.
- **Artifact builds live in the same workflow as release-please** because tags created with `GITHUB_TOKEN` never trigger other workflows — a separate tag-triggered release.yml would silently never run.
- **Generic-annotation version bump over release-please's rust type**: deterministic against our workspace-inherited version, at the documented cost that `Cargo.lock`'s member versions lag one release behind (harmless while nothing builds `--locked`; upgrade path is the cargo-workspace plugin).
- **Gate commands live in the Makefile, workflows call `make`**; apt build-deps and the artifact smoke test each live in one script shared by both workflows. Drift bit once during the session (`--no-tests=warn` added in two places); the dedup came from the review, not foresight.

## What surprised

- **The `cargo deny` licenses gate had never actually passed on the final workspace.** The Makefile arrived at the end of E1, nobody ran `make ci` end-to-end, and the allowlist said deprecated `AGPL-3.0` while the crates declare `AGPL-3.0-only` (plus `win_uds`'s Unlicense via zeromq). E2's first local gate run caught a rot that predated it.
- **ubuntu CI was green on the first attempt.** Pre-reading gpui 0.2.2's manifests from the local registry (wayland/fontconfig are dlopen'd; only xkbcommon, freetype, and xcb link at build time) turned the feared CI ping-pong into a single correct apt line.
- **The 8-angle review earned its cost**: it caught `yoshi <file>` being rejected by the new clap parser — breaking the README's headline command and the `.desktop` `Exec=yoshi %f` association while every test stayed green (nothing exercised a positional arg) — plus invalid prerelease `CFBundleVersion` and venv kernelspecs being invisible to `kernels list` under the exact workflow the README documents.
- **jupyter-zmq-client's kernelspec listing is tokio-gated** while yoshi runs the async-dispatcher feature; its sync `dirs::data_dirs()` and `KernelspecDir` struct are not, so discovery reuses those with a ~30-line sync lister rather than a parallel implementation.

## Left open

- The human Gatekeeper demo (download in a browser, right-click → Open) — the epic box's remaining evidence.
- PR #2 (`0.1.0-alpha.2`, the icon iterations) accumulates until E3's exit; its bot-PR CI runs need a one-click approval in the Actions UI.
- Kernelspec discovery migrates behind `yoshi-kernels` in E3 (noted in E3's first story).
