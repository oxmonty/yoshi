# E6: Notebook UI

A user opens a real notebook, navigates cells with Jupyter's two-mode keyboard model, edits code and markdown, and runs cells against a live kernel; demo GIF in the README cut from a release build.

Spec: [Notebook editing loop](../../PRD.md#notebook-editing-loop), [Surfaces](../../PRD.md#surfaces)

## Stories

- [ ] Scrollable cell list with selection and command/edit modes
- [ ] Command-mode keyboard parity: `A`/`B` insert, `DD` delete, `M`/`Y` type toggle, `C`/`X`/`V` cell clipboard, `Z`/`⇧Z` structural undo/redo wired to the E4 stack, `↑↓`/`⏎`/`Esc`; `⇧⏎`/`⌃⏎`/`⌥⏎` run variants
- [ ] Markdown cells toggle rendered↔source: rendered when unselected, raw source in edit mode, re-render on run
- [ ] Run All and Restart-and-Run-All (cells aborted after an error show as aborted, not running)
- [ ] Kernel status indicator + kernel picker — managed kernel first, registered kernelspecs below; selection persisted into the notebook's `kernelspec` metadata on save. The picker is the common path, not a fallback: notebook metadata usually names bare `python3`, which mispicks environments
- [ ] File open/save/save-as with native dialogs; New Notebook (`⌘N`); native macOS menu bar (`cx.set_menus`) with File/Edit/Window routing the same actions as the shortcuts
