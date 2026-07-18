mod kernel;
mod text_input;

use std::sync::Arc;
use std::time::Duration;

use futures::StreamExt as _;
use futures::channel::mpsc::UnboundedSender;
use gpui::{
    App, Application, Bounds, Context, Entity, Focusable as _, KeyBinding, PlatformDispatcher,
    Subscription, Window, WindowBounds, WindowOptions, actions, div, prelude::*, px, rgb, size,
};

use text_input::TextInput;

const HELLO_CODE: &str = "print(\"hello, yoshi\")";

actions!(yoshi, [RunCell]);

fn main() {
    match yoshi_cli::parse() {
        yoshi_cli::Invocation::Headless => match run_headless() {
            Ok(()) => std::process::exit(0),
            Err(e) => {
                eprintln!("headless kernel round-trip failed: {e:#}");
                std::process::exit(2);
            }
        },
        yoshi_cli::Invocation::KernelsList => yoshi_cli::print_kernels_list(),
        // notebook opening lands with the real document model (E6)
        yoshi_cli::Invocation::Gui(_notebook) => run_gui(),
    }
}

fn run_headless() -> anyhow::Result<()> {
    async_dispatcher::set_dispatcher(async_dispatcher::thread_dispatcher());
    let python = kernel::python_path();
    let (connection_info, guard) = kernel::spawn_kernel(&python)?;

    let (cmd_tx, cmd_rx) = futures::channel::mpsc::unbounded();
    let (tx, mut rx) = futures::channel::mpsc::unbounded();
    let _session = async_dispatcher::spawn(kernel::run_session(connection_info, cmd_rx, tx));

    let output = async_dispatcher::block_on(async move {
        let mut collected = String::new();
        while let Some(event) = rx.next().await {
            match event {
                kernel::Event::Status(s) => eprintln!("status: {s}"),
                kernel::Event::Ready => {
                    eprintln!("status: ready");
                    cmd_tx.unbounded_send(HELLO_CODE.to_string())?;
                }
                kernel::Event::Output(text) => collected.push_str(&text),
                kernel::Event::Done => return Ok(collected),
                kernel::Event::Failed(e) => anyhow::bail!(e),
            }
        }
        anyhow::bail!("event stream ended before completion")
    })?;
    drop(guard);

    anyhow::ensure!(
        output == "hello, yoshi\n",
        "unexpected output: {output:?} (expected \"hello, yoshi\\n\")"
    );
    println!("headless OK: kernel echoed {output:?}");
    Ok(())
}

struct GpuiDispatcher {
    dispatcher: Arc<dyn PlatformDispatcher>,
}

impl async_dispatcher::Dispatcher for GpuiDispatcher {
    fn dispatch(&self, runnable: async_dispatcher::Runnable) {
        self.dispatcher.dispatch(runnable, None);
    }

    fn dispatch_after(&self, duration: Duration, runnable: async_dispatcher::Runnable) {
        self.dispatcher.dispatch_after(duration, runnable);
    }
}

fn run_gui() {
    let app = Application::new();
    // macOS: clicking the Dock icon after the window was closed recreates it
    app.on_reopen(|cx| {
        if cx.windows().is_empty() {
            open_hello_window(cx);
        }
    });
    app.run(|cx: &mut App| {
        // kernel I/O runs on gpui's executor: single runtime, no tokio (PRD, runtime model)
        async_dispatcher::set_dispatcher(GpuiDispatcher {
            dispatcher: cx.background_executor().dispatcher.clone(),
        });
        text_input::bind_keys(cx);
        cx.bind_keys([KeyBinding::new("shift-enter", RunCell, None)]);
        #[cfg(not(target_os = "macos"))]
        cx.on_window_closed(|cx| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();

        open_hello_window(cx);
    });
}

fn open_hello_window(cx: &mut App) {
    let bounds = Bounds::centered(None, size(px(680.0), px(420.0)), cx);
    let window = cx
        .open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| {
                let input = cx.new(|cx| TextInput::new(cx, HELLO_CODE));
                cx.new(|cx| HelloCell::new(input, cx))
            },
        )
        .unwrap();
    window
        .update(cx, |view, window, cx| {
            window.focus(&view.input.focus_handle(cx));
            cx.activate(true);
        })
        .unwrap();
}

struct HelloCell {
    input: Entity<TextInput>,
    status: String,
    output: String,
    ready: bool,
    busy: bool,
    commands: Option<UnboundedSender<String>>,
    kernel: Option<kernel::KernelGuard>,
    _quit_hook: Subscription,
}

impl HelloCell {
    fn new(input: Entity<TextInput>, cx: &mut Context<Self>) -> Self {
        let quit_hook = cx.on_app_quit(|this: &mut Self, _cx| {
            this.kernel.take();
            async {}
        });
        let mut this = Self {
            input,
            status: "starting".to_string(),
            output: String::new(),
            ready: false,
            busy: false,
            commands: None,
            kernel: None,
            _quit_hook: quit_hook,
        };
        // prewarm: the kernel boots while the user reads the window, so the first
        // Run pays a protocol round-trip, not a CPython start
        this.start_kernel(cx);
        this
    }

    fn start_kernel(&mut self, cx: &mut Context<Self>) {
        self.status = "spawning ipykernel".to_string();
        let python = kernel::python_path();
        match kernel::spawn_kernel(&python) {
            Ok((connection_info, guard)) => {
                self.kernel = Some(guard);
                let (cmd_tx, cmd_rx) = futures::channel::mpsc::unbounded();
                let (tx, mut rx) = futures::channel::mpsc::unbounded();
                self.commands = Some(cmd_tx);
                cx.background_executor()
                    .spawn(kernel::run_session(connection_info, cmd_rx, tx))
                    .detach();
                cx.spawn(async move |this, cx| {
                    while let Some(event) = rx.next().await {
                        if this
                            .update(cx, |this, cx| {
                                this.apply(event);
                                cx.notify();
                            })
                            .is_err()
                        {
                            break;
                        }
                    }
                })
                .detach();
            }
            Err(e) => {
                self.status = format!("failed: {e:#}");
            }
        }
    }

    fn run(&mut self, cx: &mut Context<Self>) {
        if self.busy || !self.ready {
            return;
        }
        let code = self.input.read(cx).text();
        if code.trim().is_empty() {
            return;
        }
        if let Some(commands) = &self.commands {
            self.busy = true;
            self.output.clear();
            self.status = "running".to_string();
            let _ = commands.unbounded_send(code);
        }
        cx.notify();
    }

    fn apply(&mut self, event: kernel::Event) {
        match event {
            kernel::Event::Status(s) => self.status = s,
            kernel::Event::Ready => {
                self.ready = true;
                self.status = "ready — ⇧⏎ runs".to_string();
            }
            kernel::Event::Output(text) => self.output.push_str(&text),
            kernel::Event::Done => {
                self.status = "ready — ⇧⏎ runs".to_string();
                self.busy = false;
            }
            kernel::Event::Failed(e) => {
                self.status = format!("kernel error: {e}");
                self.ready = false;
                self.busy = false;
                self.kernel.take();
                self.commands.take();
            }
        }
    }
}

impl Render for HelloCell {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let run_enabled = self.ready && !self.busy;
        div()
            .size_full()
            .flex()
            .flex_col()
            .gap_3()
            .p_4()
            .bg(rgb(0x1e1e1e))
            .font_family("Menlo")
            .text_color(rgb(0xdcdcdc))
            .key_context("HelloCell")
            .on_action(cx.listener(|this, _: &RunCell, _, cx| this.run(cx)))
            .child(div().text_lg().child("yoshi — E1 hello world"))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(div().text_color(rgb(0x9a9a9a)).child("In [ ]:"))
                    .child(div().flex_1().child(self.input.clone()))
                    .child(
                        div()
                            .id("run")
                            .px_4()
                            .py_2()
                            .bg(if run_enabled {
                                rgb(0x2f6f4f)
                            } else {
                                rgb(0x333333)
                            })
                            .rounded_md()
                            .cursor_pointer()
                            .hover(|s| s.bg(rgb(0x3a8a62)))
                            .on_click(cx.listener(|this, _, _, cx| this.run(cx)))
                            .child("Run ▶"),
                    ),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(0x9a9a9a))
                    .child(format!("kernel: {}", self.status)),
            )
            .child(
                div()
                    .flex_1()
                    .px_3()
                    .py_2()
                    .bg(rgb(0x161616))
                    .rounded_md()
                    .child(if self.output.is_empty() {
                        "(output)".to_string()
                    } else {
                        self.output.clone()
                    }),
            )
    }
}
