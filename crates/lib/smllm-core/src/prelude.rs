//! The `alloc` names the crate uses everywhere. `no_std` has no std prelude, so
//! every module imports this instead (the Rust coding standard of the knowledge store).

#![allow(unused_imports)] // a prelude: not every module needs every name

pub use alloc::borrow::ToOwned;
pub use alloc::boxed::Box;
pub use alloc::format;
pub use alloc::string::{String, ToString};
pub use alloc::vec;
pub use alloc::vec::Vec;
