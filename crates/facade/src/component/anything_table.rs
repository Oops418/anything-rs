use core::str;
use gpui::{
    AnyElement, App, ClickEvent, Context, Edges, InteractiveElement, IntoElement, ParentElement,
    Pixels, SharedString, StatefulInteractiveElement, Styled, Task, Timer, Window, div,
    impl_internal_actions, px,
};
use gpui_component::{
    ActiveTheme, Size, StyleSized, green,
    list::{List, ListDelegate},
    popup_menu::PopupMenu,
    red,
    table::{self, ColFixed, ColSort, Table, TableDelegate},
};
use serde::Deserialize;
use std::{any::Any, ops::Range, time::Duration};

use super::anything_item::{Column, Something};

impl_internal_actions!(table_story, [OpenDetail, ChangeSize]);

#[derive(Clone, PartialEq, Eq, Deserialize)]
struct OpenDetail(usize);

#[derive(Clone, PartialEq, Eq, Deserialize)]
pub struct ChangeSize(pub Size);

pub struct AnythingTableDelegate {
    pub anything: Vec<Something>,
    columns: Vec<Column>,
    pub col_order: bool,
    pub col_sort: bool,
    pub loading: bool,
}

impl AnythingTableDelegate {
    pub fn new(size: usize) -> Self {
        Self {
            anything: Something::random_something(size),
            columns: vec![
                Column::new("id", "ID", None),
                Column::new("name", "Name", None),
                Column::new("path", "Path", Some(ColSort::Default)),
                Column::new("usage", "Usage", Some(ColSort::Default)),
            ],
            col_order: true,
            col_sort: true,
            loading: false,
        }
    }

    pub fn replace_anything(&mut self, new_data: Vec<Something>) {
        self.anything = new_data;
        self.loading = false;
    }

    fn render_value_cell(&self, val: f64, cx: &mut Context<Table<Self>>) -> AnyElement {
        let (fg_scale, bg_scale, opacity) = match cx.theme().mode.is_dark() {
            true => (200, 950, 0.3),
            false => (600, 50, 0.6),
        };

        let this = div().h_full().child(format!("{:.3}", val));
        // Val is a 0.0 .. n.0
        // 30% to red, 30% to green, others to default
        let right_num = ((val - val.floor()) * 1000.).floor() as i32;

        let this = if right_num % 3 == 0 {
            this.text_color(red(fg_scale))
                .bg(red(bg_scale).opacity(opacity))
        } else if right_num % 3 == 1 {
            this.text_color(green(fg_scale))
                .bg(green(bg_scale).opacity(opacity))
        } else {
            this
        };

        this.into_any_element()
    }
}

impl TableDelegate for AnythingTableDelegate {
    fn cols_count(&self, _: &App) -> usize {
        self.columns.len()
    }

    fn rows_count(&self, _: &App) -> usize {
        self.anything.len()
    }

    fn col_name(&self, col_ix: usize, _: &App) -> SharedString {
        if let Some(col) = self.columns.get(col_ix) {
            col.name.clone()
        } else {
            "--".into()
        }
    }

    fn col_width(&self, col_ix: usize, _: &App) -> Pixels {
        if col_ix == 0 {
            40.0.into()
        } else {
            200.0.into()
        }
    }

    fn col_fixed(&self, col_ix: usize, _: &App) -> Option<table::ColFixed> {
        if col_ix < 2 {
            Some(ColFixed::Left)
        } else {
            None
        }
    }

    fn render_td(
        &self,
        row_ix: usize,
        col_ix: usize,
        _: &mut Window,
        cx: &mut Context<Table<Self>>,
    ) -> impl IntoElement {
        let something = self.anything.get(row_ix).unwrap();
        let col = self.columns.get(col_ix).unwrap();

        match col.id.as_ref() {
            // "id" => something.id.to_string().into_any_element(),
            "name" => something.name.clone().into_any_element(),
            "path" => something.path.clone().into_any_element(),
            "usage" => self.render_value_cell(something.usage, cx),

            _ => "--".to_string().into_any_element(),
        }
    }

    fn can_move_col(&self, _: usize, _: &App) -> bool {
        self.col_order
    }

    fn move_col(
        &mut self,
        col_ix: usize,
        to_ix: usize,
        _: &mut Window,
        _: &mut Context<Table<Self>>,
    ) {
        let col = self.columns.remove(col_ix);
        self.columns.insert(to_ix, col);
    }

    fn col_sort(&self, col_ix: usize, _: &App) -> Option<ColSort> {
        if !self.col_sort {
            return None;
        }

        self.columns.get(col_ix).and_then(|c| c.sort)
    }

    fn perform_sort(
        &mut self,
        col_ix: usize,
        sort: ColSort,
        _: &mut Window,
        _: &mut Context<Table<Self>>,
    ) {
        if !self.col_sort {
            return;
        }

        if let Some(col) = self.columns.get_mut(col_ix) {
            match col.id.as_ref() {
                "path" => self.anything.sort_by(|a, b| match sort {
                    ColSort::Descending => b.path.cmp(&a.path),
                    _ => a.id.cmp(&b.id),
                }),
                "usage" => self.anything.sort_by(|a, b| match sort {
                    ColSort::Descending => b
                        .usage
                        .partial_cmp(&a.usage)
                        .unwrap_or(std::cmp::Ordering::Equal),
                    _ => a.id.cmp(&b.id),
                }),
                _ => {}
            }
        }
    }

    fn load_more_threshold(&self) -> usize {
        150
    }

    fn load_more(&mut self, _: &mut Window, cx: &mut Context<Table<Self>>) {
        self.loading = true;

        cx.spawn(async move |view, cx| {
            // Simulate network request, delay 1s to load data.
            Timer::after(Duration::from_secs(1)).await;

            cx.update(|cx| {
                let _ = view.update(cx, |view, _| {
                    view.delegate_mut()
                        .anything
                        .extend(Something::random_something(200));
                    view.delegate_mut().loading = false;
                });
            })
        })
        .detach();
    }
}
