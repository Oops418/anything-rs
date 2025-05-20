use core::str;
use gpui::{
    App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Subscription,
    Task, Window, actions, div,
};
use gpui_component::{
    ActiveTheme,
    checkbox::Checkbox,
    h_flex,
    list::{List, ListDelegate, ListEvent},
    v_flex,
};

use crate::anything_list_item::{Something, SomethingListItem, random_something};

actions!(anything_list, [SelectedItem]);

pub struct AnythingList {
    root: Entity<ListView>,
}

impl AnythingList {
    pub fn new(window: &mut Window, cx: &mut App) -> Self {
        let root = ListView::create(window, cx);

        Self { root }
    }

    pub fn view(window: &mut Window, cx: &mut App) -> Entity<AnythingList> {
        cx.new(|cx| Self::new(window, cx))
    }
}

impl Render for AnythingList {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().p_4().size_full().child(self.root.clone())
    }
}

struct ListView {
    item_list: Entity<List<SubAnythingListDelegate>>,
    _subscriptions: Vec<Subscription>,
}

impl ListView {
    pub fn create(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let item_list = (0..10)
            .map(|_| random_something())
            .collect::<Vec<Something>>();

        let delegate = SubAnythingListDelegate {
            matched_things: item_list.clone(),
            something_list: item_list,
            selected_index: Some(0),
            confirmed_index: None,
            query: "".to_string(),
            loading: false,
        };

        let item_list = cx.new(|cx| List::new(delegate, window, cx));

        let _subscriptions = vec![
            cx.subscribe(&item_list, |_, _, ev: &ListEvent, _| match ev {
                ListEvent::Select(ix) => {
                    println!("List Selected: {:?}", ix);
                }
                ListEvent::Confirm(ix) => {
                    println!("List Confirmed: {:?}", ix);
                }
                ListEvent::Cancel => {
                    println!("List Cancelled");
                }
            }),
        ];

        Self {
            item_list: item_list,
            _subscriptions,
        }
    }
}

impl Render for ListView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .gap_4()
            .child(
                h_flex().gap_2().flex_wrap().child(
                    Checkbox::new("loading")
                        .label("Loading")
                        .checked(self.item_list.read(cx).delegate().loading)
                        .on_click(cx.listener(|this, check: &bool, _, cx| {
                            this.item_list.update(cx, |this, cx| {
                                this.delegate_mut().loading = *check;
                                println!("the status is : {}", this.delegate_mut().loading);
                                cx.notify();
                            })
                        })),
                ),
            )
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .border_1()
                    .border_color(cx.theme().border)
                    .rounded(cx.theme().radius)
                    .child(self.item_list.clone()),
            )
    }
}

struct SubAnythingListDelegate {
    something_list: Vec<Something>,
    matched_things: Vec<Something>,
    selected_index: Option<usize>,
    confirmed_index: Option<usize>,
    query: String,
    loading: bool,
}

impl ListDelegate for SubAnythingListDelegate {
    type Item = SomethingListItem;

    fn items_count(&self, _: &App) -> usize {
        self.matched_things.len()
    }

    fn perform_search(
        &mut self,
        query: &str,
        _: &mut Window,
        _: &mut Context<List<Self>>,
    ) -> Task<()> {
        self.query = query.to_string();
        self.matched_things = self
            .something_list
            .iter()
            .filter(|something| {
                something
                    .name
                    .to_lowercase()
                    .contains(&query.to_lowercase())
            })
            .cloned()
            .collect();
        Task::ready(())
    }

    fn render_item(
        &self,
        ix: usize,
        _: &mut Window,
        _: &mut Context<List<Self>>,
    ) -> Option<Self::Item> {
        let selected = Some(ix) == self.selected_index || Some(ix) == self.confirmed_index;
        if let Some(company) = self.matched_things.get(ix) {
            return Some(SomethingListItem::new(ix, company.clone(), ix, selected));
        }
        None
    }

    fn set_selected_index(
        &mut self,
        ix: Option<usize>,
        _: &mut Window,
        cx: &mut Context<List<Self>>,
    ) {
        self.selected_index = ix;
        cx.notify();
    }

    fn loading(&self, _: &App) -> bool {
        self.loading
    }
}
