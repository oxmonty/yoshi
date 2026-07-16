// Spike A (yoshi E1): a native window with a scrollable list of ~1000 text rows,
// using warpui + warpui_core as pinned git dependencies. Adapted from warpui's
// own `examples/list` at the pinned commit.

use std::sync::{Arc, Mutex};

use anyhow::Result;
use warpui::color::ColorU;
use warpui::elements::{
    Border, ConstrainedBox, Container, Fill, List, ListState, ParentElement, Rect,
    ScrollStateHandle, Scrollable, ScrollableElement, ScrollbarWidth, Stack, Text,
};
use warpui::fonts::FamilyId;
use warpui::{
    platform, AddWindowOptions, AppContext, Element, Entity, SingletonEntity as _,
    TypedActionView, View, ViewContext,
};

struct RootView {
    list_state: ListState<()>,
    scroll_state: ScrollStateHandle,
}

impl RootView {
    fn new(ctx: &mut ViewContext<Self>) -> Self {
        let font_family = warpui::fonts::Cache::handle(ctx)
            .update(ctx, |cache, _| cache.load_system_font("Arial").unwrap());

        let list_state = ListState::new(move |i, _scroll_offset, _app| {
            Self::make_row(i, font_family).finish()
        });
        for _ in 0..1000 {
            list_state.add_item();
        }

        RootView {
            list_state,
            scroll_state: Arc::new(Mutex::new(Default::default())),
        }
    }

    fn make_row(index: usize, font_family: FamilyId) -> Container {
        let bg_color = if index % 2 == 0 {
            ColorU::new(240, 240, 240, 255)
        } else {
            ColorU::white()
        };

        Container::new(
            ConstrainedBox::new(
                Text::new_inline(format!("Row #{index}"), font_family, 16.)
                    .with_color(ColorU::black())
                    .finish(),
            )
            .with_width(600.)
            .with_height(32.)
            .finish(),
        )
        .with_background_color(bg_color)
        .with_border(Border::all(1.).with_border_color(ColorU::new(200, 200, 200, 255)))
    }
}

impl Entity for RootView {
    type Event = ();
}

impl View for RootView {
    fn ui_name() -> &'static str {
        "WarpuiSpikeRootView"
    }

    fn render(&self, _: &AppContext) -> Box<dyn Element> {
        Stack::new()
            .with_child(Rect::new().with_background_color(ColorU::white()).finish())
            .with_child(
                Scrollable::vertical(
                    self.scroll_state.clone(),
                    List::new(self.list_state.clone()).finish_scrollable(),
                    ScrollbarWidth::Auto,
                    Fill::Solid(ColorU::new(150, 150, 150, 255)),
                    Fill::Solid(ColorU::new(100, 100, 100, 255)),
                    Fill::Solid(ColorU::new(240, 240, 240, 255)),
                )
                .finish(),
            )
            .finish()
    }
}

impl TypedActionView for RootView {
    type Action = ();
}

fn main() -> Result<()> {
    // `()` is warpui_core's built-in no-op AssetProvider; this spike loads no
    // bundled assets (fonts come from the system via font-kit).
    let app_builder = platform::AppBuilder::new(platform::AppCallbacks::default(), Box::new(()), None);
    let _ = app_builder.run(move |ctx| {
        ctx.add_window(AddWindowOptions::default(), RootView::new);
    });

    Ok(())
}
