use std::{
    ops::Range,
    time::{self, Duration},
};

use fake::{
    Fake, Faker,
    faker::filesystem::en::FileName,
    rand::{self, seq::IndexedRandom},
};
use gpui::{
    AnyElement, AnyView, App, AppContext, ClickEvent, Context, Edges, Entity, Focusable, Hsla,
    InteractiveElement, IntoElement, ParentElement, Pixels, Render, SharedString,
    StatefulInteractiveElement, Styled, Timer, Window, div, impl_internal_actions,
    prelude::FluentBuilder as _, px,
};
use gpui_component::{
    ActiveTheme as _, Selectable, Sizable as _, Size, StyleSized as _,
    button::Button,
    checkbox::Checkbox,
    dock::PanelControl,
    green, h_flex,
    indicator::Indicator,
    input::{InputEvent, InputState, TextInput},
    label::Label,
    popup_menu::{PopupMenu, PopupMenuExt},
    red,
    table::{self, ColFixed, ColSort, Table, TableDelegate, TableEvent},
    v_flex,
};
use serde::Deserialize;

#[derive(Clone, PartialEq, Eq, Deserialize)]
struct ChangeSize(Size);

#[derive(Clone, PartialEq, Eq, Deserialize)]
struct OpenDetail(usize);

impl_internal_actions!(table_story, [ChangeSize, OpenDetail]);

#[derive(Clone, Debug, Default)]
struct Stock {
    id: usize,
    symbol: SharedString,
    name: SharedString,
    change: f64,
}

impl Stock {
    fn random_update(&mut self) {
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
        let path = format!("{}/{}", dir, file_name);
        self.name = SharedString::from(file_name);
        self.symbol = SharedString::from(path);
        self.change = (-300.0..999.999).fake::<f64>();
    }
}

fn random_stocks(size: usize) -> Vec<Stock> {
    (0..size)
        .map(|id| Stock {
            id,
            symbol: Faker.fake::<String>().into(),
            name: Faker.fake::<String>().into(),
            change: (-100.0..100.0).fake(),
            ..Default::default()
        })
        .collect()
}

struct Column {
    id: SharedString,
    name: SharedString,
    sort: Option<ColSort>,
}

impl Column {
    fn new(
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

struct StockTableDelegate {
    stocks: Vec<Stock>,
    columns: Vec<Column>,
    size: Size,
    loop_selection: bool,
    col_resize: bool,
    col_order: bool,
    col_sort: bool,
    col_selection: bool,
    loading: bool,
    full_loading: bool,
    fixed_cols: bool,
    eof: bool,
    visible_rows: Range<usize>,
    visible_cols: Range<usize>,
}

impl StockTableDelegate {
    fn new(size: usize) -> Self {
        Self {
            size: Size::default(),
            stocks: random_stocks(size),
            columns: vec![
                Column::new("id", "ID", None),
                Column::new("symbol", "Symbol", Some(ColSort::Default)),
                Column::new("name", "Name", None),
                Column::new("price", "Price", Some(ColSort::Default)),
                Column::new("change", "Chg", Some(ColSort::Default)),
                Column::new("change_percent", "Chg%", Some(ColSort::Default)),
                Column::new("volume", "Volume", None),
                Column::new("turnover", "Turnover", None),
                Column::new("market_cap", "Market Cap", None),
                Column::new("ttm", "TTM", None),
                Column::new("five_mins_ranking", "5m Ranking", None),
                Column::new("th60_days_ranking", "60d Ranking", None),
                Column::new("year_change_percent", "Year Chg%", None),
                Column::new("bid", "Bid", None),
                Column::new("bid_volume", "Bid Vol", None),
                Column::new("ask", "Ask", None),
                Column::new("ask_volume", "Ask Vol", None),
                Column::new("open", "Open", None),
                Column::new("prev_close", "Prev Close", None),
                Column::new("high", "High", None),
                Column::new("low", "Low", None),
                Column::new("turnover_rate", "Turnover Rate", None),
                Column::new("rise_rate", "Rise Rate", None),
                Column::new("amplitude", "Amplitude", None),
                Column::new("pe_status", "P/E", None),
                Column::new("pb_status", "P/B", None),
                Column::new("volume_ratio", "Volume Ratio", None),
                Column::new("bid_ask_ratio", "Bid Ask Ratio", None),
                Column::new("latest_pre_close", "Latest Pre Close", None),
                Column::new("latest_post_close", "Latest Post Close", None),
                Column::new("pre_market_cap", "Pre Mkt Cap", None),
                Column::new("pre_market_percent", "Pre Mkt%", None),
                Column::new("pre_market_change", "Pre Mkt Chg", None),
                Column::new("post_market_cap", "Post Mkt Cap", None),
                Column::new("post_market_percent", "Post Mkt%", None),
                Column::new("post_market_change", "Post Mkt Chg", None),
                Column::new("float_cap", "Float Cap", None),
                Column::new("shares", "Shares", None),
                Column::new("shares_float", "Float Shares", None),
                Column::new("day_5_ranking", "5d Ranking", None),
                Column::new("day_10_ranking", "10d Ranking", None),
                Column::new("day_30_ranking", "30d Ranking", None),
                Column::new("day_120_ranking", "120d Ranking", None),
                Column::new("day_250_ranking", "250d Ranking", None),
            ],
            loop_selection: true,
            col_resize: true,
            col_order: true,
            col_sort: true,
            col_selection: true,
            fixed_cols: false,
            loading: false,
            full_loading: false,
            eof: false,
            visible_cols: Range::default(),
            visible_rows: Range::default(),
        }
    }

    fn update_stocks(&mut self, size: usize) {
        self.stocks = random_stocks(size);
        self.eof = size <= 50;
        self.loading = false;
        self.full_loading = false;
    }

    fn render_value_cell(&self, val: f64, cx: &mut Context<Table<Self>>) -> AnyElement {
        let (fg_scale, bg_scale, opacity) = match cx.theme().mode.is_dark() {
            true => (200, 950, 0.3),
            false => (600, 50, 0.6),
        };

        let this = div()
            .h_full()
            .table_cell_size(self.size)
            .child(format!("{:.3}", val));
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

impl TableDelegate for StockTableDelegate {
    fn cols_count(&self, _: &App) -> usize {
        self.columns.len()
    }

    fn rows_count(&self, _: &App) -> usize {
        self.stocks.len()
    }

    fn col_name(&self, col_ix: usize, _: &App) -> SharedString {
        if let Some(col) = self.columns.get(col_ix) {
            col.name.clone()
        } else {
            "--".into()
        }
    }

    fn col_width(&self, col_ix: usize, _: &App) -> Pixels {
        if col_ix < 10 {
            120.0.into()
        } else if col_ix < 20 {
            80.0.into()
        } else {
            130.0.into()
        }
    }

    fn col_padding(&self, col_ix: usize, _: &App) -> Option<Edges<Pixels>> {
        if col_ix >= 3 && col_ix <= 10 {
            Some(Edges::all(px(0.)))
        } else {
            None
        }
    }

    fn col_fixed(&self, col_ix: usize, _: &App) -> Option<table::ColFixed> {
        if !self.fixed_cols {
            return None;
        }

        if col_ix < 4 {
            Some(ColFixed::Left)
        } else {
            None
        }
    }

    fn can_resize_col(&self, col_ix: usize, _: &App) -> bool {
        return self.col_resize && col_ix > 1;
    }

    fn can_select_col(&self, _: usize, _: &App) -> bool {
        return self.col_selection;
    }

    fn render_th(
        &self,
        col_ix: usize,
        _: &mut Window,
        cx: &mut Context<Table<Self>>,
    ) -> impl IntoElement {
        let th = div().child(self.col_name(col_ix, cx));

        if col_ix >= 3 && col_ix <= 10 {
            th.table_cell_size(self.size)
        } else {
            th
        }
    }

    fn context_menu(
        &self,
        row_ix: usize,
        menu: PopupMenu,
        _window: &Window,
        _cx: &App,
    ) -> PopupMenu {
        menu.menu(
            format!("Selected Row: {}", row_ix),
            Box::new(OpenDetail(row_ix)),
        )
        .separator()
        .menu("Size Large", Box::new(ChangeSize(Size::Large)))
        .menu("Size Medium", Box::new(ChangeSize(Size::Medium)))
        .menu("Size Small", Box::new(ChangeSize(Size::Small)))
        .menu("Size XSmall", Box::new(ChangeSize(Size::XSmall)))
    }

    fn render_tr(
        &self,
        row_ix: usize,
        _: &mut Window,
        cx: &mut Context<Table<Self>>,
    ) -> gpui::Stateful<gpui::Div> {
        div()
            .id(row_ix)
            .on_click(cx.listener(|_, ev: &ClickEvent, _, _| {
                println!(
                    "You have clicked row with secondary: {}",
                    ev.modifiers().secondary()
                )
            }))
    }

    /// NOTE: Performance metrics
    ///
    /// last render 561 cells total: 232.745µs, avg: 414ns
    /// frame duration: 8.825083ms
    ///
    /// This is means render the full table cells takes 232.745µs. Then 232.745µs / 8.82ms = 2.6% of the frame duration.
    ///
    /// If we improve the td rendering, we can reduce the time to render the full table cells.
    fn render_td(
        &self,
        row_ix: usize,
        col_ix: usize,
        _: &mut Window,
        cx: &mut Context<Table<Self>>,
    ) -> impl IntoElement {
        let stock = self.stocks.get(row_ix).unwrap();
        let col = self.columns.get(col_ix).unwrap();

        match col.id.as_ref() {
            "id" => stock.id.to_string().into_any_element(),
            "name" => stock.name.clone().into_any_element(),
            "symbol" => stock.symbol.clone().into_any_element(),
            "change" => self.render_value_cell(stock.change, cx),

            _ => "--".to_string().into_any_element(),
        }
    }

    fn can_loop_select(&self, _: &App) -> bool {
        self.loop_selection
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
                "id" => self.stocks.sort_by(|a, b| match sort {
                    ColSort::Descending => b.id.cmp(&a.id),
                    _ => a.id.cmp(&b.id),
                }),
                "symbol" => self.stocks.sort_by(|a, b| match sort {
                    ColSort::Descending => b.symbol.cmp(&a.symbol),
                    _ => a.id.cmp(&b.id),
                }),
                "change" | "change_percent" => self.stocks.sort_by(|a, b| match sort {
                    ColSort::Descending => b
                        .change
                        .partial_cmp(&a.change)
                        .unwrap_or(std::cmp::Ordering::Equal),
                    _ => a.id.cmp(&b.id),
                }),
                _ => {}
            }
        }
    }

    fn loading(&self, _: &App) -> bool {
        self.full_loading
    }

    fn can_load_more(&self, _: &App) -> bool {
        return !self.loading && !self.eof;
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
                    view.delegate_mut().stocks.extend(random_stocks(200));
                    view.delegate_mut().loading = false;
                    view.delegate_mut().eof = view.delegate().stocks.len() >= 6000;
                });
            })
        })
        .detach();
    }

    fn visible_rows_changed(
        &mut self,
        visible_range: Range<usize>,
        _: &mut Window,
        _: &mut Context<Table<Self>>,
    ) {
        self.visible_rows = visible_range;
    }

    fn visible_cols_changed(
        &mut self,
        visible_range: Range<usize>,
        _: &mut Window,
        _: &mut Context<Table<Self>>,
    ) {
        self.visible_cols = visible_range;
    }
}

pub struct TableStory {
    table: Entity<Table<StockTableDelegate>>,
    num_stocks_input: Entity<InputState>,
    stripe: bool,
    refresh_data: bool,
    size: Size,
}

impl Story for TableStory {
    fn title() -> &'static str {
        "Table"
    }

    fn description() -> &'static str {
        "A complex data table with selection, sorting, column moving, and loading more."
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        Self::view(window, cx)
    }

    fn closable() -> bool {
        false
    }
}

impl Focusable for TableStory {
    fn focus_handle(&self, cx: &gpui::App) -> gpui::FocusHandle {
        self.table.focus_handle(cx)
    }
}

impl TableStory {
    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        // Create the number input field with validation for positive integers
        let num_stocks_input = cx.new(|cx| {
            let mut input = InputState::new(window, cx)
                .placeholder("Enter number of Stocks to display")
                .validate(|s| s.parse::<usize>().is_ok());
            input.set_value("5000", window, cx);
            input
        });

        let delegate = StockTableDelegate::new(5000);
        let table = cx.new(|cx| Table::new(delegate, window, cx));

        cx.subscribe_in(&table, window, Self::on_table_event)
            .detach();
        cx.subscribe_in(&num_stocks_input, window, Self::on_num_stocks_input_change)
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
                        table.delegate_mut().stocks.iter_mut().enumerate().for_each(
                            |(i, stock)| {
                                let n = (3..10).fake::<usize>();
                                // update 30% of the stocks
                                if i % n == 0 {
                                    stock.random_update();
                                }
                            },
                        );
                    });
                    cx.notify();
                })
                .ok();
            }
        })
        .detach();

        Self {
            table,
            num_stocks_input,
            stripe: false,
            refresh_data: false,
            size: Size::default(),
        }
    }

    // Event handler for changes in the number input field
    fn on_num_stocks_input_change(
        &mut self,
        _: &Entity<InputState>,
        event: &InputEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            // Update when the user presses Enter or the input loses focus
            InputEvent::PressEnter { .. } | InputEvent::Blur => {
                let text = self.num_stocks_input.read(cx).value().to_string();
                if let Ok(num) = text.parse::<usize>() {
                    self.table.update(cx, |table, _| {
                        table.delegate_mut().update_stocks(num);
                    });
                    cx.notify();
                }
            }
            _ => {}
        }
    }

    fn toggle_loop_selection(&mut self, checked: &bool, _: &mut Window, cx: &mut Context<Self>) {
        self.table.update(cx, |table, cx| {
            table.delegate_mut().loop_selection = *checked;
            cx.notify();
        });
    }

    fn toggle_col_resize(&mut self, checked: &bool, _: &mut Window, cx: &mut Context<Self>) {
        self.table.update(cx, |table, cx| {
            table.delegate_mut().col_resize = *checked;
            table.refresh(cx);
            cx.notify();
        });
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

    fn toggle_col_selection(&mut self, checked: &bool, _: &mut Window, cx: &mut Context<Self>) {
        self.table.update(cx, |table, cx| {
            table.delegate_mut().col_selection = *checked;
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

    fn toggle_fixed_cols(&mut self, checked: &bool, _: &mut Window, cx: &mut Context<Self>) {
        self.table.update(cx, |table, cx| {
            table.delegate_mut().fixed_cols = *checked;
            table.refresh(cx);
            cx.notify();
        });
    }

    fn on_change_size(&mut self, a: &ChangeSize, _: &mut Window, cx: &mut Context<Self>) {
        self.size = a.0;
        self.table.update(cx, |table, cx| {
            table.set_size(a.0, cx);
            table.delegate_mut().size = a.0;
        });
    }

    fn toggle_refresh_data(&mut self, checked: &bool, _: &mut Window, cx: &mut Context<Self>) {
        self.refresh_data = *checked;
        cx.notify();
    }

    fn on_table_event(
        &mut self,
        _: &Entity<Table<StockTableDelegate>>,
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

impl Render for TableStory {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let delegate = self.table.read(cx).delegate();
        let rows_count = delegate.rows_count(cx);
        let size = self.size;

        v_flex()
            .on_action(cx.listener(Self::on_change_size))
            .size_full()
            .text_sm()
            .gap_4()
            .child(
                h_flex()
                    .items_center()
                    .gap_3()
                    .flex_wrap()
                    .child(
                        Checkbox::new("loop-selection")
                            .label("Loop Selection")
                            .selected(delegate.loop_selection)
                            .on_click(cx.listener(Self::toggle_loop_selection)),
                    )
                    .child(
                        Checkbox::new("col-resize")
                            .label("Column Resize")
                            .selected(delegate.col_resize)
                            .on_click(cx.listener(Self::toggle_col_resize)),
                    )
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
                        Checkbox::new("col-selection")
                            .label("Column Selection")
                            .selected(delegate.col_selection)
                            .on_click(cx.listener(Self::toggle_col_selection)),
                    )
                    .child(
                        Checkbox::new("stripe")
                            .label("Stripe")
                            .selected(self.stripe)
                            .on_click(cx.listener(Self::toggle_stripe)),
                    )
                    .child(
                        Checkbox::new("fixed-cols")
                            .label("Fixed Columns")
                            .selected(delegate.fixed_cols)
                            .on_click(cx.listener(Self::toggle_fixed_cols)),
                    )
                    .child(
                        Checkbox::new("loading")
                            .label("Loading")
                            .checked(self.table.read(cx).delegate().full_loading)
                            .on_click(cx.listener(|this, check: &bool, _, cx| {
                                this.table.update(cx, |this, cx| {
                                    this.delegate_mut().full_loading = *check;
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
                h_flex()
                    .gap_2()
                    .child(
                        Button::new("size")
                            .small()
                            .label(format!("size: {:?}", self.size))
                            .popup_menu(move |menu, _, _| {
                                menu.menu_with_check(
                                    "Large",
                                    size == Size::Large,
                                    Box::new(ChangeSize(Size::Large)),
                                )
                                .menu_with_check(
                                    "Medium",
                                    size == Size::Medium,
                                    Box::new(ChangeSize(Size::Medium)),
                                )
                                .menu_with_check(
                                    "Small",
                                    size == Size::Small,
                                    Box::new(ChangeSize(Size::Small)),
                                )
                                .menu_with_check(
                                    "XSmall",
                                    size == Size::XSmall,
                                    Box::new(ChangeSize(Size::XSmall)),
                                )
                            }),
                    )
                    .child(
                        Button::new("scroll-top")
                            .child("Scroll to Top")
                            .small()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.table.update(cx, |table, cx| {
                                    table.scroll_to_row(0, cx);
                                })
                            })),
                    )
                    .child(
                        Button::new("scroll-bottom")
                            .child("Scroll to Bottom")
                            .small()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.table.update(cx, |table, cx| {
                                    table.scroll_to_row(table.delegate().rows_count(cx) - 1, cx);
                                })
                            })),
                    ), // .child(
                       //     Button::new("scroll-first-col")
                       //         .child("Scroll to First Column")
                       //         .small()
                       //         .on_click(cx.listener(|this, _, window, cx| {
                       //             this.table.update(cx, |table, cx| {
                       //                 table.scroll_to_col(0, cx);
                       //             })
                       //         })),
                       // )
                       // .child(
                       //     Button::new("scroll-last-col")
                       //         .child("Scroll to Last Column")
                       //         .small()
                       //         .on_click(cx.listener(|this, _, window, cx| {
                       //             this.table.update(cx, |table, cx| {
                       //                 table.scroll_to_col(table.delegate().cols_count(cx), cx);
                       //             })
                       //         })),
                       // ),
            )
            .child(
                h_flex().items_center().gap_2().child(
                    h_flex()
                        .items_center()
                        .justify_between()
                        .gap_1()
                        .child(Label::new("Number of Stocks:"))
                        .child(
                            h_flex()
                                .min_w_32()
                                .child(TextInput::new(&self.num_stocks_input))
                                .into_any_element(),
                        )
                        .when(delegate.loading, |this| {
                            this.child(h_flex().gap_1().child(Indicator::new()).child("Loading..."))
                        })
                        .child(format!("Total Rows: {}", rows_count))
                        .child(format!("Visible Rows: {:?}", delegate.visible_rows))
                        .child(format!("Visible Cols: {:?}", delegate.visible_cols))
                        .when(delegate.eof, |this| this.child("All data loaded.")),
                ),
            )
            .child(self.table.clone())
    }
}

pub trait Story: Focusable + Render + Sized {
    fn klass() -> &'static str {
        std::any::type_name::<Self>().split("::").last().unwrap()
    }

    fn title() -> &'static str;
    fn description() -> &'static str {
        ""
    }
    fn closable() -> bool {
        true
    }
    fn zoomable() -> Option<PanelControl> {
        Some(PanelControl::default())
    }
    fn title_bg() -> Option<Hsla> {
        None
    }
    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable>;

    fn on_active(&mut self, active: bool, window: &mut Window, cx: &mut App) {
        let _ = active;
        let _ = window;
        let _ = cx;
    }
    fn on_active_any(view: AnyView, active: bool, window: &mut Window, cx: &mut App)
    where
        Self: 'static,
    {
        if let Some(story) = view.downcast::<Self>().ok() {
            cx.update_entity(&story, |story, cx| {
                story.on_active(active, window, cx);
            });
        }
    }
}
