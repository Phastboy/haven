//! Title criterion validation and retained query-string contribution.

use haven_domain::offer::SearchQuery;

pub(super) fn parse(raw: &str) -> (Option<SearchQuery>, Option<String>) {
    if raw.is_empty() {
        return (None, None);
    }
    match SearchQuery::parse(raw) {
        Ok(title) => (Some(title), None),
        Err(error) => (None, Some(error.to_string())),
    }
}

pub(super) fn append_url(serializer: &mut url::form_urlencoded::Serializer<'_, String>, raw: &str) {
    if !raw.is_empty() {
        serializer.append_pair("q", raw);
    }
}

pub(super) fn summarize(title: &SearchQuery, parts: &mut Vec<String>) {
    parts.push(format!("Search: {}", title.as_str()));
}
