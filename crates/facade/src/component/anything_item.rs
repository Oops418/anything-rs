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

impl Something {
    pub fn random_update(&mut self) {
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
        let path: String = format!("{}/{}", dir, file_name);
        self.name = SharedString::from(file_name);
        self.path = SharedString::from(path);
        self.usage = (-300.0..999.999).fake::<f64>();
    }

    pub fn random_something(size: usize) -> Vec<Self> {
        (0..size)
            .map(|id| Something {
                id,
                name: Faker.fake::<String>().into(),
                path: Faker.fake::<String>().into(),
                usage: (-100.0..100.0).fake(),
                ..Default::default()
            })
            .collect()
    }
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
