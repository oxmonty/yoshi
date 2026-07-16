use std::ops::Range;

use gpui::{
    App, Application, Bounds, Context, Window, WindowBounds, WindowOptions, div, prelude::*, px,
    rgb, size, uniform_list,
};

const ROW_COUNT: usize = 1000;

struct RowList {}

impl Render for RowList {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full().bg(rgb(0x1e1e1e)).child(
            uniform_list(
                "rows",
                ROW_COUNT,
                cx.processor(|_this, range: Range<usize>, _window, _cx| {
                    range
                        .map(|ix| {
                            div()
                                .id(ix)
                                .px_2()
                                .py_1()
                                .text_color(rgb(0xdcdcdc))
                                .child(format!("Row {ix}: the quick brown fox jumps over the lazy dog"))
                        })
                        .collect::<Vec<_>>()
                }),
            )
            .h_full(),
        )
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(800.0), px(600.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(|_| RowList {}),
        )
        .unwrap();
        cx.activate(true);
    });
}
