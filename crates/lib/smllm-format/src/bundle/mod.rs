//! A `json` param's JSON Schema: reading, following refs, bundling, and the
//! check of its draft (FMT-10).

mod meta;
mod refs;

pub(crate) use refs::bundle;
