# yoshi

Yoshi is a native, GPU-rendered Jupyter notebook desktop app for macOS and Linux, built in Rust on GPUI — the wgpu-class UI framework behind Zed.

```
yoshi analysis.ipynb   # < 500ms to an interactive window; pick a kernel once, ⇧⏎ runs
```

Status: pre-alpha (`0.1.0-alpha.N` releases; the E1 hello world, packaged).

## Install

Download from [GitHub Releases](https://github.com/oxmonty/yoshi/releases). Artifacts are unsigned until v0.1 — signing, notarization, and Homebrew arrive then.

**macOS** (zipped `.app`): unzip, move `yoshi.app` wherever you like, then bypass Gatekeeper on first launch — right-click the app → Open → Open (or `xattr -dr com.apple.quarantine yoshi.app`). Needed once; unsigned apps can't be opened by double-click.

**Linux** (AppImage): `chmod +x yoshi-*.AppImage && ./yoshi-*.AppImage`. Needs FUSE 2 (`libfuse2` on Debian/Ubuntu); without it, run with `--appimage-extract-and-run`.

Verify downloads against the `SHA256SUMS-*.txt` files on the release.

Running a cell needs a Python with [ipykernel](https://pypi.org/project/ipykernel/) (set `YOSHI_PYTHON`, or have a `.venv` in the working directory). `yoshi kernels list` shows the kernelspecs yoshi can see.

See [ROADMAP.md](ROADMAP.md) for the epic plan and [PRD.md](PRD.md) for the design spec.
