use std::{thread::sleep, time};

use crossbeam_channel::{Receiver, Sender};
use fake::Fake;
use gpui::{
    App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Timer, Window,
    http_client::http::request, prelude::FluentBuilder,
};
use gpui_component::{
    Selectable, Size,
    checkbox::Checkbox,
    h_flex,
    indicator::Indicator,
    input::{InputEvent, InputState, TextInput},
    label::Label,
    table::{Table, TableDelegate, TableEvent},
    v_flex,
};

use serde::de;
use tracing::{debug, warn};
use vaultify::VAULTIFY;

use crate::component::{
    anything_item::Something,
    anything_table::{AnythingTableDelegate, string_to_bool},
};

pub struct TableView {
    table: Entity<Table<AnythingTableDelegate>>,
    query_input: Entity<InputState>,
    stripe: bool,
    refresh_data: bool,
    size: Size,
    request_sender: Sender<String>,
    data_reciver: Receiver<Vec<Something>>,
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
            loop {
                let indexed = this
                    .update(cx, |this, cx| {
                        this.table
                            .update(cx, |table: &mut Table<AnythingTableDelegate>, _| {
                                debug!(
                                    "the value of indexed accessed by ui: {}",
                                    table.delegate().indexed
                                );
                                if !table.delegate().indexed {
                                    table.delegate_mut().indexed =
                                        string_to_bool(VAULTIFY.get("indexed").unwrap()).unwrap();
                                    debug!("indexed status: {}", table.delegate().indexed);
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
        })
        .detach();

        Self {
            table,
            query_input,
            stripe: false,
            refresh_data: false,
            size: Size::default(),
            request_sender,
            data_reciver,
        }
    }

    fn on_query_input_change(
        &mut self,
        _: &Entity<InputState>,
        _event: &InputEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let text = self.query_input.read(cx).value().to_string();
        self.table.update(cx, |table, _| {
            self.request_sender
                .send(text.clone())
                .expect("Failed to send request");
            debug!("request send: {}", text);
            match self.data_reciver.try_recv() {
                Ok(data) => {
                    debug!("received data: {:?}", data);
                    table.delegate_mut().replace_anything(data);
                }
                Err(_) => {
                    warn!("no data received");
                }
            }
        });
        cx.notify();
    }

    fn on_table_event(
        &mut self,
        _: &Entity<Table<AnythingTableDelegate>>,
        event: &TableEvent,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        match event {
            TableEvent::ColWidthsChanged(col_widths) => {
                println!("Col widths changed: {:?}", col_widths)
            }
            TableEvent::SelectCol(ix) => println!("Select col: {}", ix),
            TableEvent::DoubleClickedRow(ix) => println!("Double clicked row: {}", ix),
            TableEvent::SelectRow(ix) => println!("Select row: {}", ix),
            TableEvent::MoveCol(origin_idx, target_idx) => {
                println!("Move col index: {} -> {}", origin_idx, target_idx);
            }
        }
    }
}

impl Render for TableView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        self.table.update(cx, |table, cx| {
            table.set_stripe(true, cx);
        });
        let delegate = self.table.read(cx).delegate();
        let rows_count = delegate.rows_count(cx);

        v_flex()
            .size_full()
            .text_sm()
            .gap_4()
            .child(
                h_flex().items_center().justify_center().gap_2().child(
                    h_flex()
                        .items_center()
                        .justify_between()
                        .gap_1()
                        .child(Label::new("search file:"))
                        .child(
                            h_flex()
                                .min_w_32()
                                .child(TextInput::new(&self.query_input))
                                .into_any_element(),
                        )
                        .when(delegate.loading, |this| {
                            this.child(h_flex().gap_1().child(Indicator::new()).child("Loading..."))
                        })
                        .child(format!("Total Rows: {}", rows_count)),
                ),
            )
            .child(self.table.clone())
    }
}
