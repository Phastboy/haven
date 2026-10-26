//! Application navigation shell components.

pub mod app_shell;
pub mod bottom_tabs;
pub mod mobile_back_bar;
pub mod rail;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewerKind {
    Visitor,
    SignedIn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteKind {
    Main,
    Child,
}
