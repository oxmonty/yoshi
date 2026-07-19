# E10: Workspace shell

A user opens a folder, browses a project tree, and arranges notebooks side by side in draggable grid panes; opens the feedback loop with users who live in multi-notebook projects.

Spec: [Workspace shell](../../PRD.md#workspace-shell)

## Stories

- [ ] Pane grid: horizontal/vertical split, resize, close; drag a pane header to swap positions (Warp's drag-to-swap behavior is the UX reference; Zed's pane system the architecture reference, GPL, read-only)
- [ ] Project tree sidebar: `ignore`-crate walker + `notify` file watching, rendered on the same virtualized list machinery as cells; toggleable — the toggle lives leading in the E6 titlebar (Warp's placement as UX reference)
- [ ] Titlebar grows workspace controls, spiked at epic start — candidates: tree toggle leading, per-pane identity moving into pane headers, and notebook tabs with right-click rename and color coding ([Warp's context menu](../reference/warp-sidebar-tab-context-menu.png) as the interaction reference). Tab placement is user-selectable, both modes supported: a sidebar list ([Warp's current sidebar](../reference/warp-sidebar-tabs.png) — view-as Panes/Tabs, density toggle, per-item metadata) or inline with the traffic lights ([Warp's earlier browser-style strip](../reference/warp-titlebar-app-tabs.png)). The spike also settles how tabs-in-sidebar coexists with the project tree (stacked sidebar sections vs a mode swap vs a second rail). Global search/command palette stays out (Future, with in-notebook `⌘F`)
- [ ] Open notebooks from the tree into panes — each pane hosts an independent notebook view with its own kernel session
- [ ] Tree preview for images (native renderers) and `video/mp4` (sandboxed webview), reusing the output pipeline
- [ ] Workspace state (open panes, layout, tree visibility) persisted and restored across restarts
