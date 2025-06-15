use std::{env, f32, path::PathBuf};

use gpui::{
    AppContext, ClickEvent, Context, Corner, Element, Entity, Hsla, InteractiveElement,
    IntoElement, MouseButton, ParentElement, Pixels, Render, SharedString, Styled, Subscription,
    Timer, Window, div, prelude::FluentBuilder, px,
};
use gpui_component::{
    ActiveTheme, ColorName, Icon, IconName, Sizable, Size, Theme, ThemeMode, TitleBar,
    button::{Button, ButtonVariants},
    color_picker::{ColorPickerEvent, ColorPickerState},
    divider::Divider,
    indicator::Indicator,
    popover::{Popover, PopoverContent},
    tag::Tag,
    v_flex,
};
use tracing::{debug, trace};
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

        cx.spawn(async move |this, cx| {
            loop {
                this.update(cx, |this, cx| {
                    trace!(
                        "the value of indexed files accessed by ui: {}",
                        this.index_files_count
                    );
                    this.index_files_count = VAULTIFY.get("indexed_files").unwrap().into();
                    this.progress_value = VAULTIFY
                        .get("indexed_progress")
                        .unwrap()
                        .parse::<f32>()
                        .unwrap();
                    trace!("indexed files: {}", this.index_files_count);
                    cx.notify();
                })
                .unwrap();

                Timer::after(std::time::Duration::from_secs(2)).await;
            }
        })
        .detach();

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
                                .child(div().text_sm().child(format!(
                                    "• {}%",
                                    if self.progress_value == 100.0 {
                                        "100".to_string()
                                    } else {
                                        format!("{:.2}", self.progress_value)
                                    }
                                )))
                                .mr(Pixels(-5.)),
                        )
                        .child(if self.progress_value == 100.0 {
                            div()
                                .child(Icon::new(IconName::Eye).with_size(Size::Small))
                                .into_any_element()
                        } else {
                            Indicator::new().small().into_any_element()
                        }),
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
                    Popover::new("refresh_popover")
                        .anchor(Corner::TopRight)
                        .trigger(
                            Button::new("refresh")
                                .icon(IconName::EyeOff)
                                .small()
                                .ghost(),
                        )
                        .content(|window, cx| {
                            cx.new(|cx| {
                                PopoverContent::new(window, cx, |_, _| {
                                    v_flex()
                                        .gap_4()
                                        .w_80()
                                        .child("Are you sure you want to refresh index now?")
                                        .child(Divider::horizontal())
                                        .child(
                                            div().flex().justify_center().w_full().child(
                                                Button::new("refresh_info")
                                                    .label("Yes")
                                                    .w(px(80.))
                                                    .on_click(|_, _, cx| {
                                                        VAULTIFY
                                                            .set("refresh", "true".to_string())
                                                            .unwrap();
                                                        if let Ok(current_exe) = env::current_exe()
                                                        {
                                                            cx.restart(Some(current_exe));
                                                        } else {
                                                            cx.restart(Some(PathBuf::from(
                                                                "/Applications/Anything.app",
                                                            )));
                                                        }
                                                    })
                                                    .small(),
                                            ),
                                        )
                                        .into_any()
                                })
                            })
                        }),
                )
                // .child(
                //     Button::new("Custom Path")
                //         .icon(IconName::Eye)
                //         .small()
                //         .ghost()
                //         .on_click(cx.listener(Self::change_theme_mode)),
                // )
                .child(
                    Button::new("github")
                        .small()
                        .ghost()
                        .text()
                        .on_click(|_, _, cx| cx.open_url("https://github.com/Oops418/anything-rs"))
                        .child(Tag::color(ColorName::Cyan).child("Beta")),
                ),
        )
    }
}
