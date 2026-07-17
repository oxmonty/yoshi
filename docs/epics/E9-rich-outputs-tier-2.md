# E9: Rich outputs, tier 2 (webview)

Pandas `text/html` tables and plotly figures render in sandboxed, virtualized webviews that scroll and clip correctly within the cell list; opens the feedback loop with data-science early adopters on remaining unrenderable MIME types. The largest post-MVP unknown.

Spec: [Output rendering](../../PRD.md#output-rendering)

## Stories

- [ ] Spike, ordered by kill-risk: (1) Linux embedding — wry child views are X11-only, so probe the GTK-host path under Wayland first; (2) attach to the native view handle captured in E1; (3) scroll-sync/clipping quality during momentum scroll. Go/no-go recorded
- [ ] Escalation ladder recorded: wry overlay → `wgpu-scry` (system webview composited into a wgpu texture) → static-image fallback (plotly static export + "open in browser") → CEF offscreen rendering as last resort (input plumbing, process supervision, and per-helper notarization — not just bundle size)
- [ ] Webview pool: create-on-visible, recycle-on-scroll, hard cap on live instances
- [ ] Sandbox policy: no fs/network bridge, plotly.js bundled locally, CSP locked down (nteract's iframe-isolation model is the reference)
- [ ] Native table view for `application/vnd.dataresource+json`; SVG via resvg (both stay native); `video/mp4` outputs play in the sandboxed webview (system codecs, no media dependencies)
- [ ] Instrument: opt-in, local-only logging of unrenderable MIME types to prioritize tier 3
