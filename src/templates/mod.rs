use askama::Template;
use itertools::Itertools;

use crate::{Message, Toc};

#[derive(Template)]
#[template(path = "toc.html")]
pub struct TocTemplate<'a> {
    toc: &'a Toc,
}

impl<'a> TocTemplate<'a> {
    pub fn render(toc: &'a Toc) -> String {
        Self { toc }.render().unwrap_or_else(|e| e.to_string())
    }
}

#[derive(Template)]
#[template(path = "message_page.html")]
pub struct MessagePageTemplate<'a> {
    message_groups: Vec<MessageGroup<'a>>,
}

struct MessageGroup<'a> {
    username: &'a str,
    first_message: &'a Message,
    messages: Vec<&'a Message>,
}

impl MessagePageTemplate<'_> {
    pub fn render(messages: &[Message]) -> String {
        let grouped_messages = messages.iter().rev().group_by(|msg| &msg.username);
        let message_groups = grouped_messages
            .into_iter()
            .map(|(username, mut messages)| {
                let first_message = messages.next().unwrap();
                let messages = messages.collect();
                MessageGroup { username, first_message, messages }
            })
            .collect();
        MessagePageTemplate { message_groups }.render().unwrap_or_else(|e| e.to_string())
    }
}
