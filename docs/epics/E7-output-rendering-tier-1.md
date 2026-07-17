# E7: Output rendering, tier 1

Matplotlib inline plots, pandas text reprs, tracebacks, and streaming stdout all render correctly in the golden-notebook structure tests; the five MVP renderers ship behind a MIME-ranking dispatcher.

Spec: [Output rendering](../../PRD.md#output-rendering)

## Stories

- [ ] MIME bundle ranking (richest-renderable-first, Zed's model), with `text/html` explicitly ranked below `text/plain` in MVP so the sibling fallback is unambiguous
- [ ] Renderers: ANSI/plain text streaming (adjacent-stream merge, `\r` handling), error/traceback, PNG/JPEG, markdown, latex-as-plain (deferred math)
- [ ] `clear_output(wait)` as a first-class output-store operation, plus `update_display_data`/`display_id` mutation — both mechanisms power progress bars
- [ ] Output cap + "show more" for large streams; clear-cell-outputs and clear-all commands
- [ ] Golden tests assert renderer choice, MIME dispatch, and output structure — not pixels (GPU rasterization differs across platforms)
