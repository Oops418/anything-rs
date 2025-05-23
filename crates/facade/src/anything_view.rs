use core::str;
use crossbeam_channel::{Receiver, Sender};
use gpui::{
    App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, actions,
    div,
};

use crate::{anything_table_view::TableView, component::anything_item::Something};

actions!(anything_list, [SelectedItem]);

pub struct AnythingView {
    root: Entity<TableView>,
}

impl AnythingView {
    pub fn create(
        window: &mut Window,
        cx: &mut App,
        request_sender: Sender<String>,
        data_reciver: Receiver<Vec<Something>>,
    ) -> Entity<AnythingView> {
        cx.new(|cx| {
            let root = TableView::create(window, cx, request_sender, data_reciver);
            AnythingView { root }
        })
    }
}

impl Render for AnythingView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().p_4().size_full().child(self.root.clone())
    }
}
