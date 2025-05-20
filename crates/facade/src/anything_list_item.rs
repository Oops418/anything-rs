use fake::Fake;
use fake::rand;
use fake::rand::seq::IndexedRandom;
use gpui::{
    App, ElementId, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div, px,
};
use gpui_component::label::Label;
use gpui_component::{ActiveTheme, h_flex, list::ListItem, v_flex};

use fake::faker::filesystem::en::FileName;

#[derive(Clone)]
pub struct Something {
    pub name: SharedString,
    pub path: SharedString,
}

#[derive(IntoElement)]
pub struct SomethingListItem {
    base: ListItem,
    ix: usize,
    something: Something,
    selected: bool,
}

impl SomethingListItem {
    pub fn new(id: impl Into<ElementId>, something: Something, ix: usize, selected: bool) -> Self {
        SomethingListItem {
            something,
            ix,
            base: ListItem::new(id),
            selected,
        }
    }
}

impl RenderOnce for SomethingListItem {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let text_color = if self.selected {
            cx.theme().accent_foreground
        } else {
            cx.theme().foreground
        };

        let bg_color = if self.selected {
            cx.theme().list_active
        } else if self.ix % 2 == 0 {
            cx.theme().list
        } else {
            cx.theme().list_even
        };

        self.base
            .px_3()
            .py_1()
            .overflow_x_hidden()
            .bg(bg_color)
            .child(
                h_flex()
                    .items_center()
                    .gap_2()
                    .text_color(text_color)
                    .text_sm()
                    .child(
                        v_flex()
                            .gap_1()
                            .w(px(250.))
                            .overflow_x_hidden()
                            .flex_nowrap()
                            .child(Label::new(self.something.name.clone()).whitespace_nowrap()),
                    )
                    .child(
                        v_flex().overflow_x_hidden().child(
                            div()
                                .text_color(text_color)
                                .whitespace_nowrap()
                                .child(self.something.path.clone()),
                        ),
                    ),
            )
    }
}

pub fn random_something() -> Something {
    // Generate a realistic file name
    let file_name = FileName().fake::<String>();

    // Common Linux directories to make paths more realistic
    let common_dirs = [
        "/home/user/Documents",
        "/home/user/Projects",
        "/var/log",
        "/etc/config",
        "/usr/local/share",
        "/opt/applications",
        "/home/user/Downloads",
    ];

    // Choose a random base directory and combine with generated filename
    let dir = common_dirs.choose(&mut rand::thread_rng()).unwrap();
    let path = format!("{}/{}", dir, file_name);

    Something {
        name: SharedString::from(file_name),
        path: SharedString::from(path),
    }
}
