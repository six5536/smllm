//! Pictures of the machines for `smllm graph`: the layout, the SVG, the
//! interactive page, the PNG, and the browser (CLI-8).

mod details;
mod font;
mod layout;
mod open;
mod page;
mod png;
mod route;
mod shared;
mod side;
mod svg;
mod walk;

pub(crate) use open::open;
pub(crate) use page::page;
pub(crate) use png::png;
pub(crate) use walk::{guard_label, state_edges};
