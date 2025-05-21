use anything_view::AnythingView;
use asset::Assets;
use component::anything_item::Something;
use crossbeam_channel::{Receiver, Sender};
use gpui::{
    App, AppContext, Application, Bounds, KeyBinding, Menu, MenuItem, Window, WindowBounds,
    WindowKind, WindowOptions, actions, px, size,
};
use gpui_component::{
    Root, TitleBar,
    input::{Copy, Cut, Paste, Redo, Undo},
};
use root::FacadeRoot;

mod anything_table_view;
mod anything_view;
mod asset;
pub mod component;
mod root;
mod title_bar;

actions!(facade, [Quit, Hide]);

pub fn setup(request_sender: Sender<String>, data_reciver: Receiver<Vec<Something>>) {
    let app = Application::new().with_assets(Assets);
    app.run(|cx: &mut App| {
        gpui_component::init(cx);
        Facade::shortcut_binding_init(cx);
        Facade::menu_init(cx);

        cx.activate(true);
        let window_options = Facade::window_options_init(cx);
        Facade::windows_async_init(cx, window_options, request_sender, data_reciver);
    });
}

struct Facade();

impl Facade {
    fn windows_async_init(
        cx: &mut App,
        windows_options: WindowOptions,
        request_sender: Sender<String>,
        data_reciver: Receiver<Vec<Something>>,
    ) {
        cx.spawn(async move |cx| {
            let window = cx
                .open_window(windows_options, |window, cx: &mut App| {
                    let view = AnythingView::create(window, cx, request_sender, data_reciver);
                    let root = cx.new(|cx| FacadeRoot::create(view, window, cx));

                    cx.new(|cx| Root::new(root.into(), window, cx))
                })
                .expect("failed to open window");

            window
                .update(cx, |_, window: &mut Window, _| {
                    window.activate_window();
                    window.set_window_title("Anything");
                })
                .expect("failed to update window");

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    }

    fn shortcut_binding_init(cx: &mut App) {
        cx.bind_keys([
            KeyBinding::new("cmd-q", Quit, None),
            KeyBinding::new("cmd-w", Hide, None),
        ]);
        cx.on_action(|_: &Quit, cx: &mut App| {
            cx.quit();
        });
        cx.on_action(|_: &Hide, cx: &mut App| {
            cx.hide();
        });
    }

    fn menu_init(cx: &mut App) {
        cx.set_menus(vec![
            Menu {
                name: "Anas".into(),
                items: vec![MenuItem::action("Quit", Quit)],
            },
            Menu {
                name: "Edit".into(),
                items: vec![
                    MenuItem::os_action("Undo", Undo, gpui::OsAction::Undo),
                    MenuItem::os_action("Redo", Redo, gpui::OsAction::Redo),
                    MenuItem::separator(),
                    MenuItem::os_action("Cut", Cut, gpui::OsAction::Cut),
                    MenuItem::os_action("Copy", Copy, gpui::OsAction::Copy),
                    MenuItem::os_action("Paste", Paste, gpui::OsAction::Paste),
                ],
            },
            Menu {
                name: "Window".into(),
                items: vec![],
            },
        ]);
    }

    fn window_options_init(cx: &mut App) -> WindowOptions {
        let mut window_size = size(px(1600.0), px(1200.0));
        if let Some(display) = cx.primary_display() {
            let display_size = display.bounds().size;
            window_size.width = window_size.width.min(display_size.width * 0.5);
            window_size.height = window_size.height.min(display_size.height * 0.5);
        }
        let window_bounds = Bounds::centered(None, window_size, cx);

        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(window_bounds)),
            titlebar: Some(TitleBar::title_bar_options()),
            window_min_size: Some(gpui::Size {
                width: px(640.),
                height: px(480.),
            }),
            kind: WindowKind::Normal,
            ..Default::default()
        }
    }
}
