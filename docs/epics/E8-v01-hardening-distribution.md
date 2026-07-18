# E8: v0.1 hardening + distribution

V0.1.0 is cut, signed and notarized, installable via `brew install oxmonty/tap/yoshi` on macOS and AppImage on Linux, with benchmarks published in the README.

Spec: [Validation strategy](../../PRD.md#validation-strategy), [Distribution](../../PRD.md#distribution)

## Stories

- [ ] Bench harness committed: cold start = process launch → UI interactive (warm caches); kernel-ready reported separately (bounded by CPython startup); scroll FPS on a tier-1 capped-output notebook; measured against nteract Desktop and JupyterLab Desktop; download-size chart regenerated (`script/bench-download-size`, built in E2) and embedded in the README with the startup numbers. Further chart candidates for the same README section, added as the harness measures them: cold start (ms, defined stopwatch), idle memory RSS with one notebook open (the anti-Electron number), scroll frame time (p95 ms/frame); every chart carries its measurement date
- [ ] Crash-safe autosave / sidecar recovery file
- [ ] Settings: `~/.config/yoshi/settings.json` + `keymap.json` — defaults written on first run, a menu command opens them for manual editing (editor settings, keybinding overrides); no settings GUI in v0.1
- [ ] Themes, persisted in settings.json: Gruvbox Dark Soft (default), Gruvbox Light, One Dark
- [ ] macOS signing: Developer ID + notarytool + stapling + hardened-runtime entitlements (covers the bundled `uv` binary too); `.dmg` artifact
- [ ] Homebrew cask in `oxmonty/homebrew-tap` (macOS-only; CLI exposed via the cask `binary` stanza) + release-please tag pipeline
- [ ] README one-click download links return, per platform, on evergreen `/releases/latest/download/` URLs — the stable-named alias assets release.yml already uploads make these zero-maintenance once v0.1.0 is a full (non-prerelease) release; sensible only once artifacts are signed and open without the quarantine dance
- [ ] Branding pass: real app icon (`.icns` + Linux icon), About panel, `.dmg` background — logo asset needed before this story (halo-gradient reference committed in E2: `assets/icon/`)
- [ ] `yoshi upgrade` (alias `update`): stable/next channels per the PRD's [Upgrade](../../PRD.md#distribution) design — GitHub Releases discovery, checksum-verified self-swap for AppImage/bare `.app`, `brew upgrade --cask` delegation on macOS
- [ ] Download metrics, VLC's model: count server-side from the GitHub Releases API (`assets[].download_count`) plus Homebrew analytics — no in-app phone-home, so no disclaimer or opt-out needed; an opt-in update-ping is considered only if `yoshi upgrade` ever grows an automatic check
