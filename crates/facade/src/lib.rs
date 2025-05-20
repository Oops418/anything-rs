use anything_list::AnythingList;
use asset::Assets;
use gpui::{
    AnyView, App, AppContext, Application, Bounds, KeyBinding, Menu, MenuItem, SharedString,
    Window, WindowBounds, WindowKind, WindowOptions, actions, px, size,
};
use gpui_component::{
    Root, TitleBar,
    input::{Copy, Cut, Paste, Redo, Undo},
};
use root::FacadeRoot;

mod anything_list;
mod anything_list_item;
mod asset;
mod demo;
mod root;
mod title_bar;

actions!(facade, [Quit, Hide]);

struct Facade {}

impl Facade {
    fn init(cx: &mut App) {
        gpui_component::init(cx);
        Self::shortcut_binding_init(cx);
        Self::menu_init(cx);
        cx.activate(true);
    }

    fn create_new_window<F, E>(title: &str, crate_view_fn: F, cx: &mut App)
    where
        E: Into<AnyView>,
        F: FnOnce(&mut Window, &mut App) -> E + Send + 'static,
    {
        let options = Self::window_options_init(cx);
        let title = SharedString::from(title.to_string());
        cx.spawn(async move |cx| {
            let window = cx
                .open_window(options, |window, cx: &mut App| {
                    let view = crate_view_fn(window, cx);
                    let root = cx.new(|cx| FacadeRoot::new(view, window, cx));

                    cx.new(|cx| Root::new(root.into(), window, cx))
                })
                .expect("failed to open window");

            window
                .update(cx, |_, window, _| {
                    window.activate_window();
                    window.set_window_title(&title);
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

pub fn setup() {
    let app = Application::new().with_assets(Assets);

    app.run(|cx: &mut App| {
        Facade::init(cx);

        Facade::create_new_window("Anything", AnythingList::view, cx);
    });
}
