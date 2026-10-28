//! Public topic/title/price parsing, display and URL composition for the feed.

mod category;
mod parsed;
mod price;
mod title;

pub(super) use parsed::ParsedCriteria;

#[cfg(test)]
mod tests;
