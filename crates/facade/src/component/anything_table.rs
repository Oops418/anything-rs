use gpui::{
    AnyElement, App, Context, InteractiveElement, IntoElement, ParentElement, Pixels, SharedString,
    Styled, Window, actions, div,
};
use gpui_component::{
    Icon,
    popup_menu::PopupMenu,
    table::{self, ColFixed, ColSort, Table, TableDelegate},
};
use material_icon_embed_rs::material_icon_file::MaterialIconFile;
use material_icon_embed_rs::material_icon_folder::MaterialIconFolder;
use vaultify::VAULTIFY;

use super::{
    anything_item::{Column, Something},
    custom_icon::{FileIcon, FolderIcon},
};

actions!(anything_table_action, [OpenSystemFolder, OpenSystemFile]);

pub struct AnythingTableDelegate {
    pub anything: Vec<Something>,
    columns: Vec<Column>,
    pub col_order: bool,
    pub col_sort: bool,
    pub loading: bool,
    pub indexed: bool,
}

impl AnythingTableDelegate {
    pub fn new() -> Self {
        Self {
            anything: vec![],
            columns: vec![
                Column::new("class", "Kind", None),
                Column::new("name", "Name", None),
                Column::new("path", "Path", None),
                Column::new("size", "Size", Some(ColSort::Default)),
                Column::new(
                    "last_modified_date",
                    "Last Modified",
                    Some(ColSort::Default),
                ),
            ],
            col_order: true,
            col_sort: true,
            loading: false,
            indexed: string_to_bool(VAULTIFY.get("indexed").unwrap()).unwrap(),
        }
    }

    pub fn replace_anything(&mut self, new_data: Vec<Something>) {
        self.anything = new_data;
        self.loading = false;
    }

    fn render_kind_cell(&self, kind: &SharedString, name: &SharedString) -> AnyElement {
        if kind == "folder" {
            return div()
                .flex()
                .items_center()
                .justify_center()
                .h_full()
                .child(Icon::from(FolderIcon::from(
                    MaterialIconFolder::from_folder_name(name),
                )))
                .into_any_element();
        } else {
            div()
                .flex()
                .items_center()
                .justify_center()
                .h_full()
                .child(Icon::from(FileIcon::from(
                    MaterialIconFile::from_extension(kind),
                )))
                .into_any_element()
        }
    }

    fn render_value_cell(&self, size: f64) -> AnyElement {
        let formatted_size = if size >= 1024.0 * 1024.0 * 1024.0 {
            format!("{:.1} GB", size / (1024.0 * 1024.0 * 1024.0))
        } else if size >= 1024.0 * 1024.0 {
            format!("{:.1} MB", size / (1024.0 * 1024.0))
        } else if size >= 1024.0 {
            format!("{:.1} KB", size / 1024.0)
        } else {
            format!("{:.0} B", size)
        };

        div().h_full().child(formatted_size).into_any_element()
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
        match col_ix {
            0 => 45.0.into(),
            1 => 300.0.into(),
            2 => 600.0.into(),
            3 => 80.0.into(),
            4 => 120.0.into(),
            _ => 100.0.into(),
        }
    }

    fn col_fixed(&self, col_ix: usize, _: &App) -> Option<table::ColFixed> {
        if col_ix < 2 {
            Some(ColFixed::Left)
        } else {
            None
        }
    }

    fn loading(&self, _: &App) -> bool {
        !self.indexed
    }

    fn can_load_more(&self, _cx: &App) -> bool {
        false
    }

    fn context_menu(
        &self,
        row_ix: usize,
        menu: PopupMenu,
        _window: &Window,
        _cx: &App,
    ) -> PopupMenu {
        menu.menu(
            format!("{}", self.anything.get(row_ix).unwrap().name),
            Box::new(OpenSystemFile),
        )
        .separator()
        .menu("Open", Box::new(OpenSystemFile))
        .menu("Open Folder", Box::new(OpenSystemFolder))
    }

    fn render_tr(
        &self,
        row_ix: usize,
        _: &mut Window,
        _cx: &mut Context<Table<Self>>,
    ) -> gpui::Stateful<gpui::Div> {
        div().id(row_ix)
    }

    fn render_td(
        &self,
        row_ix: usize,
        col_ix: usize,
        _: &mut Window,
        _cx: &mut Context<Table<Self>>,
    ) -> impl IntoElement {
        let something = self.anything.get(row_ix).unwrap();
        let col = self.columns.get(col_ix).unwrap();

        match col.id.as_ref() {
            "class" => self.render_kind_cell(&something.class, &something.name),
            "name" => something.name.clone().into_any_element(),
            "path" => something.path.clone().into_any_element(),
            "size" => self.render_value_cell(something.size),
            "last_modified_date" => something.last_modified_date.to_string().into_any_element(),
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
                "size" => self.anything.sort_by(|a, b| match sort {
                    ColSort::Descending => b
                        .size
                        .partial_cmp(&a.size)
                        .unwrap_or(std::cmp::Ordering::Equal),
                    _ => a
                        .size
                        .partial_cmp(&b.size)
                        .unwrap_or(std::cmp::Ordering::Equal),
                }),
                "last_modified_date" => self.anything.sort_by(|a, b| match sort {
                    ColSort::Descending => b.last_modified_date.cmp(&a.last_modified_date),
                    _ => a.last_modified_date.cmp(&b.last_modified_date),
                }),
                _ => {}
            }
        }
    }
}

pub fn string_to_bool(s: String) -> Option<bool> {
    match s.as_str() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}
