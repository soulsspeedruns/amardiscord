use askama::Template;

use crate::Toc;

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
