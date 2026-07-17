# E11: Terminal

A GPU-rendered terminal pane runs a real shell in the grid next to a notebook (the agent workflow: Claude Code editing the notebook you're viewing); drag-swappable like any pane.

Spec: [Terminal](../../PRD.md#terminal)

## Stories

- [ ] Terminal engine: `alacritty_terminal` (Apache-2.0) for PTY + grid state, rendered as a native GPUI view (Zed's `crates/terminal` is the architecture reference, GPL, read-only)
- [ ] Terminal pane type in the E10 grid: shell spawned in the workspace directory, drag-swap with notebook panes
- [ ] Keyboard/IME passthrough, scrollback, selection + copy/paste, ANSI colors from the active theme
- [ ] Warp UX pass: catalogue the interaction details worth porting by hand (drag-swap affordances, block-style polish — warpui's MIT components are fair styling references; no code extraction)
