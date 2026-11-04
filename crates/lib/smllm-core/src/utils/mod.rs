mod small_map;
mod sort;

pub use small_map::SmallMap;
pub use sort::merge_sort_by;

use crate::prelude::*;

/// `items` separated by `, `.
pub(crate) fn join<'a>(items: impl IntoIterator<Item = &'a str>) -> String {
    join_with(&mut items.into_iter(), ", ")
}

/// `items` separated by `sep`. Every join goes through this one function,
/// in place of a copy of the loop for each iterator type, and in place of
/// the standard library's `[String]::join`, which adds about 900 bytes to the
/// wasm.
pub(crate) fn join_with(items: &mut dyn Iterator<Item = &str>, sep: &str) -> String {
    let mut out = String::new();
    for (i, s) in items.enumerate() {
        if i > 0 {
            out.push_str(sep);
        }
        out.push_str(s);
    }
    out
}
