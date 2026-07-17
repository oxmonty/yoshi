# E4: Notebook document model

Every notebook in the golden corpus round-trips idempotently to canonical form: the first save may reflow once to the same output `nbformat.writes` produces, and every save thereafter is byte-stable; `yoshi-notebook` crate with golden tests and zero UI deps.

Spec: [Document model](../../PRD.md#document-model)

## Stories

- [ ] Fidelity spike (story 1, gates the rest): confirm what the `nbformat` v3 crate preserves — unknown fields survive via flattened maps, but key order does not; wrap with a `preserve_order` `serde_json::Value` layer or a custom serializer, and evaluate Warp's AGPL `ipynb_parser` as an alternative; decision recorded
- [ ] Canonical writer matching `nbformat.writes`: 1-space indent, `ensure_ascii=False`, trailing newline, source as line-arrays, adjacent same-name streams merged; v4.0 notebooks are not upconverted (no cell ids added) on open
- [ ] Cell CRUD (insert, delete, move, change type) on the in-memory model
- [ ] Model-level undo/redo stack over cell operations (min depth 100)
- [ ] Dirty tracking + atomic save (write-temp-rename)
- [ ] Golden corpus curated for producer and encoding diversity: classic Notebook, JupyterLab, VS Code, and papermill outputs; v4.0 and v4.5; non-ASCII/emoji content; widget and unknown metadata
