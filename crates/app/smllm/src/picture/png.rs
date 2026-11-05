//! The PNG picture of a machine: its diagram under its ID, the text drawn as
//! strokes, rendered by resvg at twice its size (CLI-8).

use resvg::tiny_skia::{Pixmap, Transform};
use resvg::usvg::{Options, Tree};
use smllm_core::model::Machine;

use crate::error::{Error, Result};
use crate::picture::svg;

/// The longest side of a picture drawn at twice its size; a larger one is
/// drawn at its own size.
const DOUBLE_UP_TO: f64 = 8192.0;
/// The longest side of a picture.
const MAX_SIDE: f64 = 16384.0;

/// The PNG bytes of the diagram of `m`, without its details.
// sokf-implements: CLI-8_AC-7
pub(crate) fn png(m: &Machine) -> Result<Vec<u8>> {
    let d = svg::titled(m);
    let doc = svg::document(&d, &format!("State machine {}", m.id), true);
    let tree = Tree::from_str(&doc, &Options::default())
        .map_err(|e| Error::msg(format!("cannot draw the picture: {e}")))?;
    let side = d.width.max(d.height).ceil();
    let scale = if side * 2.0 <= DOUBLE_UP_TO { 2.0 } else { 1.0 };
    let (w, h) = ((d.width * scale).ceil(), (d.height * scale).ceil());
    let pixmap = if side * scale <= MAX_SIDE {
        Pixmap::new(w as u32, h as u32)
    } else {
        None
    };
    let mut pixmap = pixmap.ok_or_else(|| {
        Error::msg(format!(
            "the picture is too large to draw: {w} × {h} pixels"
        ))
    })?;
    resvg::render(
        &tree,
        Transform::from_scale(scale as f32, scale as f32),
        &mut pixmap.as_mut(),
    );
    pixmap
        .encode_png()
        .map_err(|e| Error::msg(format!("cannot encode the picture: {e}")))
}
