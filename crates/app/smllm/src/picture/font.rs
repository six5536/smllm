//! Text metrics and outlines from the Borland sans stroke font, so that the
//! layout measures every label the same way on every machine and the PNG
//! draws its labels without a system font (CLI-8).

use std::fmt::Write as _;

use bgi_stroked_fonts::sans::{SANS, SANS_WIDTH};

/// Font units from the top of a capital to the baseline.
const CAP: f64 = 21.0;
/// The baseline, in font units from the top of the glyph cell.
const BASELINE: f64 = 32.0;
/// A capital's height as a share of the font size, as in a common sans font.
const CAP_SHARE: f64 = 0.7;

/// The glyph index of `c` in the font, whose 223 glyphs are code page 437
/// from 32 to 254: ASCII as itself, a few symbols mapped, and `?` for the rest.
fn glyph(c: char) -> usize {
    let code = match c {
        ' '..='~' => c as u32,
        '≥' => 242,
        '≤' => 243,
        '±' => 241,
        '·' => 250,
        '…' => return glyph('.'),
        _ => '?' as u32,
    };
    code as usize - 32
}

/// A hyphen of the width of the font's `i`, because the font's own hyphen
/// is as long as a dash.
const HYPHEN: [i8; 4] = [1, 22, 8, 22];
const HYPHEN_WIDTH: u8 = 11;

/// The advance of a comma: that of the font's full stop, because the font's
/// own comma advances as far as its dash and leaves a wide gap after it.
const COMMA_WIDTH: u8 = 10;

/// The strokes and the advance of `c`.
fn shape(c: char) -> (&'static [i8], u8) {
    if c == '-' {
        return (&HYPHEN, HYPHEN_WIDTH);
    }
    let g = glyph(c);
    let advance = if c == ',' { COMMA_WIDTH } else { SANS_WIDTH[g] };
    (SANS[g], advance)
}

/// The scale from font units to pixels at `size`.
fn scale(size: f64) -> f64 {
    size * CAP_SHARE / CAP
}

/// The text as the font draws it: `→` as `->`, as the font has no arrow.
fn chars(text: &str) -> impl Iterator<Item = char> + '_ {
    text.chars().flat_map(|c| match c {
        '→' => vec!['-', '>'],
        '…' => vec!['.', '.', '.'],
        c => vec![c],
    })
}

/// The width of `text` at `size` pixels.
pub(crate) fn width(text: &str, size: f64) -> f64 {
    let units: f64 = chars(text).map(|c| f64::from(shape(c).1)).sum();
    units * scale(size)
}

/// An SVG path of `text` at `size` pixels, its left end at `x` and its
/// baseline at `y`, each stroke a line to draw with a round cap.
pub(crate) fn outline(text: &str, x: f64, y: f64, size: f64) -> String {
    let s = scale(size);
    let mut d = String::new();
    let mut pen = x;
    for c in chars(text) {
        let (strokes, advance) = shape(c);
        for seg in strokes.as_chunks::<4>().0 {
            let px = |v: i8| pen + f64::from(v) * s;
            let py = |v: i8| y + (f64::from(v) - BASELINE) * s;
            let _ = write!(
                d,
                "M{:.1} {:.1}L{:.1} {:.1}",
                px(seg[0]),
                py(seg[1]),
                px(seg[2]),
                py(seg[3])
            );
        }
        pen += f64::from(advance) * s;
    }
    d
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn widths_add_up_and_scale() {
        assert_eq!(width("", 12.0), 0.0);
        let one = width("A", 10.0);
        assert!((width("AA", 10.0) - 2.0 * one).abs() < 1e-9);
        assert!((width("A", 20.0) - 2.0 * one).abs() < 1e-9);
        assert!(width("W", 12.0) > width("i", 12.0));
    }

    #[test]
    fn symbols_map_and_the_rest_fall_back() {
        assert_eq!(glyph('≥'), 210);
        assert_eq!(glyph('·'), 218);
        assert_eq!(glyph('é'), glyph('?'));
        assert_eq!(width("→", 12.0), width("->", 12.0));
        assert_eq!(width("…", 12.0), width("...", 12.0));
        assert!(width("-", 12.0) < width("m", 12.0) / 1.5);
        assert_eq!(width(",", 12.0), width(".", 12.0));
    }

    #[test]
    fn outlines_start_at_the_pen_and_sit_on_the_baseline() {
        assert_eq!(outline(" ", 0.0, 0.0, 12.0), "");
        let d = outline("l", 10.0, 50.0, 21.0 / 0.7);
        // `l` runs from the cap height to the baseline.
        assert!(d.starts_with("M10.0 29.0L10.0 50.0"), "{d}");
    }
}
