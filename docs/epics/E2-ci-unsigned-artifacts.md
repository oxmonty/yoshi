# E2: CI + unsigned artifacts (walking skeleton)

The E1 hello world, unchanged, downloads from GitHub Releases and runs on a clean machine (zipped `.app` on macOS after the documented quarantine strip — or quarantine-free via the curl installer — AppImage on Linux), with CI green on both platforms. Signed installers and Homebrew arrive with v0.1 (E8).

Spec: [Distribution](../../PRD.md#distribution), [CI/CD](../../PRD.md#cicd)

## Stories

- [x] GitHub Actions matrix (macos-14, ubuntu-24.04): fmt, clippy, `cargo deny`, nextest
- [x] release-please → tag → build → GitHub Release: zipped `.app` + AppImage + checksums, unsigned (Gatekeeper bypass documented in the README); first release is `v0.1.0-alpha.1`
- [x] Minimal app identity: macOS bundle with Info.plist (bundle id `com.oxmonty.yoshi`, version from Cargo) + placeholder `.icns`; AppImage carries a `.desktop` file + icon
- [x] `yoshi --version` and `yoshi kernels list` work from a downloaded artifact (kernelspec-discovery smoke test)
- [x] Every later epic stays green on CI from here; one release-please PR merged per epic exit (`0.1.0-alpha.N` series until E8)
