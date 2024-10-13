use askama::Template;
use itertools::Itertools;

use crate::search::SearchResult;
use crate::{ChannelList, Message, ScrollDirection};

#[derive(Template)]
#[template(path = "channel_list.html")]
pub struct ChannelListTemplate<'a> {
    channel_list: &'a ChannelList,
}

impl<'a> ChannelListTemplate<'a> {
    pub fn render(channel_list: &'a ChannelList) -> String {
        Self { channel_list }.render().unwrap_or_else(|e| e.to_string())
    }
}

struct MessageGroup<'a> {
    username: &'a str,
    first_message: &'a Message,
    messages: Vec<&'a Message>,
}

#[derive(Template)]
#[template(path = "message_page.html")]
pub struct MessagePageTemplate<'a> {
    message_groups: Vec<MessageGroup<'a>>,
    channel_id: u64,
    page: u64,
    direction: ScrollDirection,
}

impl MessagePageTemplate<'_> {
    pub fn render(
        messages: &[Message],
        channel_id: u64,
        page: u64,
        direction: ScrollDirection,
    ) -> String {
        MessagePageTemplate {
            message_groups: messages
                .iter()
                .rev()
                .group_by(|msg| &msg.username)
                .into_iter()
                .map(|(username, mut messages)| {
                    let first_message = messages.next().unwrap();
                    let messages = messages.collect();
                    MessageGroup { username, first_message, messages }
                })
                .collect(),
            channel_id,
            page,
            direction,
        }
        .render()
        .unwrap_or_else(|e| e.to_string())
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
