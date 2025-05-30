use gpui::SharedString;

use gpui_component::table::ColSort;

#[derive(Debug)]
pub struct Something {
    pub class: SharedString,
    pub path: SharedString,
    pub last_modified_date: time::Date,
    pub name: SharedString,
    pub size: f64,
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
