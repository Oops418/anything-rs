use std::rc::Rc;

use gpui::{
    AnyElement, App, AppContext, ClickEvent, Context, Entity, Hsla, InteractiveElement,
    IntoElement, MouseButton, ParentElement, Render, SharedString, Styled, Subscription, Window,
    div, prelude::FluentBuilder,
};
use gpui_component::{
    ActiveTheme, ContextModal, IconName, Sizable, Theme, ThemeMode, TitleBar,
    badge::Badge,
    button::{Button, ButtonVariants},
    color_picker::{ColorPickerEvent, ColorPickerState},
};

pub struct FacadeTitleBar {
    title: SharedString,
    theme_color: Entity<ColorPickerState>,
    child: Rc<dyn Fn(&mut Window, &mut App) -> AnyElement>,
    _subscriptions: Vec<Subscription>,
}

impl FacadeTitleBar {
    pub fn new(
        title: impl Into<SharedString>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let theme_color =
            cx.new(|cx| ColorPickerState::new(window, cx).default_value(cx.theme().primary));
        let _subscriptions = vec![cx.subscribe_in(
            &theme_color,
            window,
            |this, _, ev: &ColorPickerEvent, window, cx| match ev {
                ColorPickerEvent::Change(color) => {
                    this.set_theme_color(*color, window, cx);
                }
            },
        )];

        Self {
            title: title.into(),
            theme_color,
            child: Rc::new(|_, _| div().into_any_element()),
            _subscriptions,
        }
    }

    fn set_theme_color(
        &mut self,
        color: Option<Hsla>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(color) = color {
            let theme = cx.global_mut::<Theme>();
            theme.apply_color(color);
            self.theme_color.update(cx, |state, cx| {
                state.set_value(color, window, cx);
            });
            window.refresh();
        }
    }

    fn change_theme_mode(&mut self, _: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        let mode = match cx.theme().mode.is_dark() {
            true => ThemeMode::Light,
            false => ThemeMode::Dark,
        };

        Theme::change(mode, None, cx);
        self.set_theme_color(self.theme_color.read(cx).value(), window, cx);
    }
}

impl Render for FacadeTitleBar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let notifications_count = window.notifications(cx).len();

        TitleBar::new()
            // left side
            .child(div().flex().items_center().child(self.title.clone()))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_end()
                    .px_2()
                    .gap_2()
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .child((self.child.clone())(window, cx))
                    .child(
                        Button::new("theme-mode")
                            .map(|this| {
                                if cx.theme().mode.is_dark() {
                                    this.icon(IconName::Sun)
                                } else {
                                    this.icon(IconName::Moon)
                                }
                            })
                            .small()
                            .ghost()
                            .on_click(cx.listener(Self::change_theme_mode)),
                    )
                    .child(
                        Button::new("github")
                            .icon(IconName::GitHub)
                            .small()
                            .ghost()
                            .on_click(|_, _, cx| cx.open_url("https://github.com/Oops418")),
                    )
                    .child(
                        div().relative().child(
                            Badge::new().count(notifications_count).max(99).child(
                                Button::new("bell")
                                    .small()
                                    .ghost()
                                    .compact()
                                    .icon(IconName::Bell),
                            ),
                        ),
                    ),
            )
    }
}
