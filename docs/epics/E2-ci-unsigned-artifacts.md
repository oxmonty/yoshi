# E2: CI + unsigned artifacts (walking skeleton)

The E1 hello world, unchanged, downloads from GitHub Releases and runs on a clean machine (zipped `.app` on macOS via right-click-Open, AppImage on Linux), with CI green on both platforms. Signed installers and Homebrew arrive with v0.1 (E8).

Spec: [Distribution](../../PRD.md#distribution), [CI/CD](../../PRD.md#cicd)

## Stories

- [ ] GitHub Actions matrix (macos-14, ubuntu-24.04): fmt, clippy, `cargo deny`, nextest
- [ ] Tag → build → GitHub Release: zipped `.app` + AppImage + checksums, unsigned (Gatekeeper bypass documented in the README)
- [ ] Minimal app identity: macOS bundle with Info.plist (bundle id `com.oxmonty.yoshi`, version from Cargo) + placeholder `.icns`; AppImage carries a `.desktop` file + icon
- [ ] `yoshi --version` and `yoshi kernels list` work from a downloaded artifact (kernelspec-discovery smoke test)
- [ ] Every later epic stays green on CI from here; tag per epic
