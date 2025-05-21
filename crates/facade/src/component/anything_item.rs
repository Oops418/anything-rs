use fake::Fake;
use fake::Faker;
use fake::rand;
use fake::rand::seq::IndexedRandom;
use gpui::{
    App, ElementId, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div, px,
};
use gpui_component::label::Label;
use gpui_component::table::ColSort;
use gpui_component::{ActiveTheme, h_flex, list::ListItem, v_flex};

use fake::faker::filesystem::en::FileName;

#[derive(Clone, Default, Debug)]
pub struct Something {
    pub id: usize,
    pub path: SharedString,
    pub name: SharedString,
    pub usage: f64,
}

pub struct Column {
    pub id: SharedString,
    pub name: SharedString,
    pub sort: Option<ColSort>,
}

impl Column {
    pub fn new(
        id: impl Into<SharedString>,
        name: impl Into<SharedString>,
        sort: Option<ColSort>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            sort,
        }
    }
}
