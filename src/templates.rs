use askama::Template;
use itertools::Itertools;

use crate::search::SearchResult;
use crate::{ChannelList, Message, ScrollDirection};

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate;

impl IndexTemplate {
    pub fn render() -> String {
        Self {}.render().unwrap_or_else(|e| e.to_string())
    }
}

#[derive(Template)]
#[template(path = "layout.html")]
pub struct LayoutTemplate<'a> {
    title: &'a str,
    content: &'a str,
}

impl<'a> LayoutTemplate<'a> {
    pub fn render(title: &'a str, content: &'a str) -> String {
        Self { title, content }.render().unwrap_or_else(|e| e.to_string())
    }
}

#[derive(Template)]
#[template(path = "channel_list.html")]
pub struct ChannelListTemplate<'a> {
    channel_list: &'a ChannelList,
    current_channel_id: Option<u64>,
}

impl<'a> ChannelListTemplate<'a> {
    pub fn render(channel_list: &'a ChannelList, current_channel_id: Option<u64>) -> String {
        Self { channel_list, current_channel_id }.render().unwrap_or_else(|e| e.to_string())
    }
}

struct MessageGroup<'a> {
    username: &'a str,
    first_message: &'a Message,
    messages: Vec<&'a Message>,
    highlighted: bool,
}

#[derive(Template)]
#[template(path = "message_page.html")]
pub struct MessagePageTemplate<'a> {
    message_groups: Vec<MessageGroup<'a>>,
    channel_id: u64,
    channel_name: String,
    page: u64,
    direction: ScrollDirection,
}

impl MessagePageTemplate<'_> {
    pub fn render<'a>(
        messages: &[Message],
        channel_id: u64,
        channel_name: String,
        page: u64,
        direction: ScrollDirection,
        target_message_id: Option<u64>,
    ) -> String {
        if messages.is_empty() {
            String::new()
        } else {
            let message_groups = messages
                .iter()
                .rev()
                .group_by(|msg| &msg.username)
                .into_iter()
                .map(|(username, mut messages)| {
                    let first_message = messages.next().unwrap();
                    let messages = messages.collect::<Vec<_>>();
                    let highlighted = target_message_id
                        .map(|id| {
                            first_message.rowid == id || messages.iter().any(|m| m.rowid == id)
                        })
                        .unwrap_or(false);
                    MessageGroup { username, first_message, messages, highlighted }
                })
                .collect();

            MessagePageTemplate { message_groups, channel_id, channel_name, page, direction }
                .render()
                .unwrap_or_else(|e| e.to_string())
        }
    }
}

struct SearchResultGroup<'a> {
    username: &'a str,
    first_search_result: &'a SearchResult,
    search_results: Vec<&'a SearchResult>,
}

#[derive(Template)]
#[template(path = "search.html")]
pub struct SearchTemplate<'a> {
    search_result_groups: Vec<SearchResultGroup<'a>>,
}

impl SearchTemplate<'_> {
    pub fn render(search_results: &[SearchResult]) -> String {
        SearchTemplate {
            search_result_groups: search_results
                .iter()
                .rev()
                .group_by(|search_result| &search_result.message.username)
                .into_iter()
                .map(|(username, mut search_results)| {
                    let first_search_result = search_results.next().unwrap();
                    let search_results = search_results.collect();
                    SearchResultGroup { username, first_search_result, search_results }
                })
                .collect(),
        }
        .render()
        .unwrap_or_else(|e| e.to_string())
    }
}
