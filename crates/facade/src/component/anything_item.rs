use gpui::SharedString;

use gpui_component::table::ColSort;

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
