use gpui::{
    AppContext, ClickEvent, Context, Entity, Hsla, InteractiveElement, IntoElement, MouseButton,
    ParentElement, Render, Styled, Subscription, Window, div, prelude::FluentBuilder,
};
use gpui_component::{
    ActiveTheme, IconName, Sizable, Theme, ThemeMode, TitleBar,
    button::{Button, ButtonVariants},
    color_picker::{ColorPickerEvent, ColorPickerState},
};

pub struct FacadeTitleBar {
    theme_color: Entity<ColorPickerState>,
    _subscriptions: Vec<Subscription>,
}

impl FacadeTitleBar {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
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
            theme_color,
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
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        TitleBar::new().child(div()).child(
            div()
                .flex()
                .items_center()
                .justify_end()
                .px_2()
                .gap_2()
                .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
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
                ),
        )
    }
}
