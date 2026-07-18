<h1>
  <img alt="yoshi logo" src="./assets/icon/yoshi-readme.png" width="70" valign="middle">
  &nbsp;yoshi notebooks
</h1>

<a href="https://somsubhra.github.io/github-release-stats/?username=oxmonty&repository=yoshi"><img alt="Total downloads across all releases" src="https://img.shields.io/github/downloads/oxmonty/yoshi/total?label=downloads&color=2f6f4f"></a>
<a href="https://github.com/oxmonty/yoshi/releases/latest"><img alt="Latest release" src="https://img.shields.io/github/v/release/oxmonty/yoshi?include_prereleases&label=release&color=1c1c1a"></a>

Yoshi is a native, GPU-rendered Jupyter notebook desktop app for macOS and Linux, built in Rust on GPUI — the wgpu-class UI framework behind Zed.

```
yoshi analysis.ipynb   # < 500ms to an interactive window; pick a kernel once, ⇧⏎ runs
```

Status: pre-alpha (`0.1.0-alpha.N` releases; the E1 hello world, packaged).

## Install

Recommended — the installer resolves the newest release, verifies checksums, and (on macOS) skips Gatekeeper quarantine entirely, since curl downloads carry no quarantine flag:

```sh
curl -fsSL https://raw.githubusercontent.com/oxmonty/yoshi/main/install.sh | sh
```

Or download from [GitHub Releases](https://github.com/oxmonty/yoshi/releases) (binaries + checksums per platform).

Artifacts are unsigned until v0.1 — signing, notarization, Homebrew, and one-click download links arrive then.

**macOS** (zipped `.app`): browser downloads are quarantined, and modern macOS shows unsigned quarantined apps as *"yoshi.app is damaged and can't be opened"* — that's Gatekeeper talking, not a corrupt download, and right-click → Open no longer bypasses it. Clear it once after unzipping:

```sh
xattr -dr com.apple.quarantine yoshi.app
```

(or use the curl installer above and skip all of this). Signing at v0.1 removes the dance entirely.

**Linux** (AppImage): `chmod +x yoshi-*.AppImage && ./yoshi-*.AppImage`. Needs FUSE 2 (`libfuse2` on Debian/Ubuntu); without it, run with `--appimage-extract-and-run`.

Verify downloads against the `SHA256SUMS-*.txt` files on the release.

Running a cell needs a Python with [ipykernel](https://pypi.org/project/ipykernel/) (set `YOSHI_PYTHON`, or have a `.venv` in the working directory). `yoshi kernels list` shows the kernelspecs yoshi can see.

See [ROADMAP.md](ROADMAP.md) for the epic plan and [PRD.md](PRD.md) for the design spec.
