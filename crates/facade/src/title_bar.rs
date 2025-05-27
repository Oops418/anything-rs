use std::f32;

use gpui::{
    AppContext, ClickEvent, Context, Entity, Hsla, InteractiveElement, IntoElement, MouseButton,
    ParentElement, Render, SharedString, Styled, Subscription, Timer, Window, div,
    prelude::FluentBuilder,
};
use gpui_component::{
    ActiveTheme, IconName, Sizable, Theme, ThemeMode, TitleBar,
    button::{Button, ButtonVariants},
    color_picker::{ColorPickerEvent, ColorPickerState},
    progress::Progress,
};
use tracing::debug;
use vaultify::VAULTIFY;

pub struct FacadeTitleBar {
    theme_color: Entity<ColorPickerState>,
    progress_value: f32,
    index_files_count: SharedString,
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

        // cx.spawn(async move |this, cx| {
        //     loop {
        //         this.update(cx, |this, cx| {
        //             debug!(
        //                 "the value of indexed files accessed by ui: {}",
        //                 this.index_files_count
        //             );
        //             this.index_files_count = VAULTIFY.get("indexed_files").unwrap().into();
        //             this.progress_value = VAULTIFY
        //                 .get("indexed_progress")
        //                 .unwrap()
        //                 .parse::<f32>()
        //                 .unwrap();
        //             debug!("indexed files: {}", this.index_files_count);
        //             cx.notify();
        //         })
        //         .unwrap();

        //         Timer::after(std::time::Duration::from_secs(2)).await;
        //     }
        // })
        // .detach();

        debug!("title bar crated");
        Self {
            theme_color,
            progress_value: 65.0,
            index_files_count: "".into(),
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
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .mr_4()
                        .child(div().text_sm().child("Indexed Files: "))
                        .child(div().text_sm().child(format!("{}", self.index_files_count)))
                        .child(
                            div()
                                .w_24()
                                .ml_2()
                                .child(Progress::new().value(self.progress_value)),
                        ),
                )
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
                        .on_click(|_, _, cx| cx.open_url("https://github.com/Oops418/anything-rs")),
                ),
        )
    }
}
