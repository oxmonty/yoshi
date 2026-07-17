# E5: Cell editor

The single largest net-new UI component, built as its own epic on the E1 decision: type, select, and syntax-highlight code in a cell, with working IME and clipboard. Approach gated on E1's primitive inventory; `warp_editor` reuse only if Spike A showed extraction cheap.

Spec: [Cell editor](../../PRD.md#cell-editor)

## Stories

- [ ] Text buffer + selection model: helix-core ropes on GPUI's input primitives (E1 confirmed GPUI ships primitives but no editor widget)
- [ ] Rendering: cosmic-text shaping/layout wired into the framework's draw path; cursor + selection painting; intra-cell scroll
- [ ] Syntax highlighting (tree-sitter or syntect) for Python and markdown source
- [ ] IME + clipboard correctness: CJK entry, dead keys, text copy/paste in both directions
- [ ] Intra-cell text undo/redo from the editor history (structural cell undo lives in the model, E4)
