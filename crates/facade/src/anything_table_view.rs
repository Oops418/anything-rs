use std::{path::PathBuf, process::Command};

use gpui::{
    App, AppContext, Context, Entity, Focusable, InteractiveElement, IntoElement, KeyDownEvent,
    ParentElement, Render, Styled, Timer, Window,
};
use gpui_component::{
    h_flex,
    input::{InputEvent, InputState, TextInput},
    table::{Table, TableEvent},
    v_flex,
};
use smol::channel::{Receiver, Sender};

use tracing::{debug, trace};
use vaultify::VAULTIFY;

use crate::component::{
    anything_item::Something,
    anything_table::{AnythingTableDelegate, OpenSystemFile, OpenSystemFolder, string_to_bool},
};

pub struct TableView {
    table: Entity<Table<AnythingTableDelegate>>,
    query_input: Entity<InputState>,
    // stripe: bool,
    // refresh_data: bool,
    // size: Size,
    request_sender: Sender<String>,
}

impl TableView {
    pub fn create(
        window: &mut Window,
        cx: &mut App,
        request_sender: Sender<String>,
        data_reciver: Receiver<Vec<Something>>,
    ) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx, request_sender, data_reciver))
    }

    fn new(
        window: &mut Window,
        cx: &mut Context<Self>,
        request_sender: Sender<String>,
        data_reciver: Receiver<Vec<Something>>,
    ) -> Self {
        debug!("creating table view");
        let query_input = cx.new(|cx| InputState::new(window, cx).placeholder("file name..."));

        let delegate = AnythingTableDelegate::new();
        let table = cx.new(|cx| Table::new(delegate, window, cx));

        cx.subscribe_in(&table, window, Self::on_table_event)
            .detach();
        cx.subscribe_in(&query_input, window, Self::on_query_input_change)
            .detach();

        cx.spawn(async move |this, cx| {
            while let Ok(data) = data_reciver.recv().await {
                trace!("Background task received data: {:?}", data.len());
                this.update(cx, |this, cx| {
                    this.table
                        .update(cx, |table: &mut Table<AnythingTableDelegate>, _| {
                            table.delegate_mut().replace_anything(data);
                        });
                    cx.notify();
                })
                .ok();
            }
        })
        .detach();

        cx.spawn(async move |this, cx| {
            loop {
                let indexed = this
                    .update(cx, |this, cx| {
                        this.table
                            .update(cx, |table: &mut Table<AnythingTableDelegate>, _| {
                                trace!(
                                    "the value of indexed accessed by ui: {}",
                                    table.delegate().indexed
                                );
                                if !table.delegate().indexed {
                                    table.delegate_mut().indexed =
                                        string_to_bool(VAULTIFY.get("indexed").unwrap()).unwrap();
                                    trace!("indexed status: {}", table.delegate().indexed);
                                }
                                table.delegate().indexed
                            })
                    })
                    .unwrap_or(false);

                if indexed {
                    break;
                }

                Timer::after(std::time::Duration::from_secs(2)).await;
            }
            debug!("indexed status is finished, show normal table view");
        })
        .detach();

        Self {
            table,
            query_input,
            // stripe: false,
            // refresh_data: false,
            // size: Size::default(),
            request_sender,
        }
    }

    fn on_table_event(
        &mut self,
        _: &Entity<Table<AnythingTableDelegate>>,
        event: &TableEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            TableEvent::DoubleClickedRow(_) => {
                let selected_row_ix = self.table.read(cx).selected_row().unwrap();
                let path = self
                    .table
                    .read(cx)
                    .delegate()
                    .anything
                    .get(selected_row_ix)
                    .unwrap()
                    .path
                    .clone();
                cx.open_with_system(&PathBuf::from(path.to_string()));
            }
            _ => {}
        }
    }

    fn on_query_input_change(
        &mut self,
        _: &Entity<InputState>,
        event: &InputEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            InputEvent::Change(text) => {
                let text = text.to_string().trim().to_string();
                debug!("query input changed");
                if text.is_empty() {
                    debug!("empty query");
                    self.table
                        .update(cx, |table: &mut Table<AnythingTableDelegate>, _| {
                            table.delegate_mut().replace_anything(vec![]);
                        });
                    cx.notify();
                    return;
                }
                self.request_sender.try_send(text.clone()).ok();
                debug!("request sent: {}", text);
                cx.notify();
            }
            _ => {}
        }
    }

    fn on_key_space(&mut self, event: &KeyDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if event.keystroke.key == "space" {
            if let Some(selected_row_ix) = self.table.read(cx).selected_row() {
                let path = self
                    .table
                    .read(cx)
                    .delegate()
                    .anything
                    .get(selected_row_ix)
                    .unwrap()
                    .path
                    .to_string();

                debug!("Previewing file at path: {}", path);
                let status = Command::new("qlmanage").arg("-p").arg(&path).spawn().ok();

                if status.is_none() {
                    debug!("Failed to open file with Quick Look: {}", path);
                } else {
                    debug!("File opened successfully: {}", path);
                }
            }
        }
    }

    fn on_open_system_folder(
        &mut self,
        _: &OpenSystemFolder,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let selected_row_ix = self.table.read(cx).selected_row().unwrap();
        let path = self
            .table
            .read(cx)
            .delegate()
            .anything
            .get(selected_row_ix)
            .unwrap()
            .path
            .clone();
        cx.reveal_path(&PathBuf::from(path.to_string()));
        println!("Open System Folder action triggered");
    }

    fn on_open_system_file(&mut self, _: &OpenSystemFile, _: &mut Window, cx: &mut Context<Self>) {
        let selected_row_ix = self.table.read(cx).selected_row().unwrap();
        let path = self
            .table
            .read(cx)
            .delegate()
            .anything
            .get(selected_row_ix)
            .unwrap()
            .path
            .clone();
        cx.open_with_system(&PathBuf::from(path.to_string()));
    }
}

impl Render for TableView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        self.table.update(cx, |table, cx| {
            table.set_stripe(true, cx);
        });

        v_flex()
            .on_action(cx.listener(Self::on_open_system_folder))
            .on_action(cx.listener(Self::on_open_system_file))
            .size_full()
            .text_sm()
            .gap_4()
            .track_focus(&self.table.focus_handle(cx))
            .on_key_down(cx.listener(Self::on_key_space)) // Add keyboard handler
            .child(
                h_flex().items_center().justify_center().gap_2().child(
                    h_flex().items_center().justify_between().gap_1().child(
                        h_flex()
                            .min_w_64()
                            .child(TextInput::new(&self.query_input))
                            .into_any_element(),
                    ),
                ),
            )
            .child(self.table.clone())
    }
}
