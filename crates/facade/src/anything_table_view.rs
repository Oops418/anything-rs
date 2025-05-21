use std::time;

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

use crate::component::{anything_item::Something, anything_table::AnythingTableDelegate};

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
        // Create the number input field with validation for positive integers
        let query_input = cx.new(|cx| InputState::new(window, cx).placeholder("file name..."));

        let delegate = AnythingTableDelegate::new(20);
        let table = cx.new(|cx| Table::new(delegate, window, cx));

        cx.subscribe_in(&table, window, Self::on_table_event)
            .detach();
        cx.subscribe_in(&query_input, window, Self::on_query_input_change)
            .detach();

        // Spawn a background to random refresh the list
        cx.spawn(async move |this, cx| {
            loop {
                Timer::after(time::Duration::from_millis(33)).await;

                this.update(cx, |this, cx| {
                    if !this.refresh_data {
                        return;
                    }

                    this.table.update(cx, |table, _| {
                        table
                            .delegate_mut()
                            .anything
                            .iter_mut()
                            .enumerate()
                            .for_each(|(i, stock)| {
                                let n = (3..10).fake::<usize>();
                                // update 30% of the stocks
                                if i % n == 0 {
                                    stock.random_update();
                                }
                            });
                    });
                    cx.notify();
                })
                .ok();
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
            println!("request already send: {}", text);
            let new_data = self.data_reciver.recv().unwrap();
            println!("received data: {:?}", new_data);
            table.delegate_mut().replace_anything(new_data);
        });
        cx.notify();
    }

    fn toggle_col_order(&mut self, checked: &bool, _: &mut Window, cx: &mut Context<Self>) {
        self.table.update(cx, |table, cx| {
            table.delegate_mut().col_order = *checked;
            table.refresh(cx);
            cx.notify();
        });
    }

    fn toggle_col_sort(&mut self, checked: &bool, _: &mut Window, cx: &mut Context<Self>) {
        self.table.update(cx, |table, cx| {
            table.delegate_mut().col_sort = *checked;
            table.refresh(cx);
            cx.notify();
        });
    }

    fn toggle_stripe(&mut self, checked: &bool, _: &mut Window, cx: &mut Context<Self>) {
        self.stripe = *checked;
        let stripe = self.stripe;
        self.table.update(cx, |table, cx| {
            table.set_stripe(stripe, cx);
            cx.notify();
        });
    }

    fn toggle_refresh_data(&mut self, checked: &bool, _: &mut Window, cx: &mut Context<Self>) {
        self.refresh_data = *checked;
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
        let delegate = self.table.read(cx).delegate();
        let rows_count = delegate.rows_count(cx);
        let size = self.size;

        v_flex()
            .size_full()
            .text_sm()
            .gap_4()
            .child(
                h_flex()
                    .items_center()
                    .gap_3()
                    .flex_wrap()
                    .child(
                        Checkbox::new("col-order")
                            .label("Column Order")
                            .selected(delegate.col_order)
                            .on_click(cx.listener(Self::toggle_col_order)),
                    )
                    .child(
                        Checkbox::new("col-sort")
                            .label("Column Sort")
                            .selected(delegate.col_sort)
                            .on_click(cx.listener(Self::toggle_col_sort)),
                    )
                    .child(
                        Checkbox::new("stripe")
                            .label("Stripe")
                            .selected(self.stripe)
                            .on_click(cx.listener(Self::toggle_stripe)),
                    )
                    .child(
                        Checkbox::new("loading")
                            .label("Loading")
                            .on_click(cx.listener(|this, check: &bool, _, cx| {
                                this.table.update(cx, |this, cx| {
                                    cx.notify();
                                })
                            })),
                    )
                    .child(
                        Checkbox::new("refresh-data")
                            .label("Refresh Data")
                            .selected(self.refresh_data)
                            .on_click(cx.listener(Self::toggle_refresh_data)),
                    ),
            )
            .child(
                h_flex().items_center().gap_2().child(
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
