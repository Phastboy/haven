//! Bound public-facet predicates; membership, visibility and pagination live in the feed.

mod emission;
mod price;
mod title;

pub(super) use emission::append;
