//! The diagram of a machine as SVG: each state a box with its name and its
//! tags, each transition an arrow labelled with its event and its guard
//! (CLI-8).

use std::fmt::Write as _;

use smllm_core::model::Machine;

use crate::picture::font;
use crate::picture::layout::{
    self, EdgeSpec, LABEL_SIZE, LINE, NAME_SIZE, Spec, StateSpec, TAG_SIZE,
};
use crate::picture::walk;

/// The longest line of a label, in characters; a longer guard ends in `…`.
const LABEL_CHARS: usize = 44;
/// The font size of a machine's title in the PNG.
const TITLE_SIZE: f64 = 16.0;
const TITLE_H: f64 = 30.0;
const FONT: &str = "system-ui, -apple-system, 'Segoe UI', Helvetica, Arial, sans-serif";
const INK: &str = "#24292f";
const MUTED: &str = "#57606a";
/// The colour of an edge without a guard.
const LINE_INK: &str = "#57606a";
/// The colour of an edge with a guard, and of its guard.
const GUARD_INK: &str = "#bc4c00";
/// The colour of the start dot, its arrow, and the initial state's border.
const START_INK: &str = "#0969da";

// sokf-implements: CLI-8_AC-11
/// The fill and the border of a state: the first that applies of final
/// (purple), fallback (amber), and entry point (green), else grey, and blue
/// for the initial state's border. The tags say the same in words.
fn colours(state: &smllm_core::model::State, initial: bool) -> (&'static str, &'static str) {
    let (fill, stroke) = if state.is_final {
        ("#fbefff", "#8250df")
    } else if state.fallback {
        ("#fff8c5", "#9a6700")
    } else if state.entry_point {
        ("#dafbe1", "#1a7f37")
    } else if initial {
        ("#ddf4ff", START_INK)
    } else {
        ("#f6f8fa", "#6e7781")
    };
    (fill, if initial { START_INK } else { stroke })
}

/// How the diagram draws its text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Text {
    /// As SVG text in the browser's font, with the parts that a page selects.
    Live,
    /// As strokes of the built-in font, for a rasteriser without fonts.
    Strokes,
}

/// A drawn machine: the SVG elements, their width, and their height.
pub(crate) struct Drawing {
    pub body: String,
    pub width: f64,
    pub height: f64,
}

/// `text` cut to `max` characters, the last of them `…`.
pub(crate) fn cut(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        text.to_string()
    } else {
        let mut s: String = text.chars().take(max - 1).collect();
        s.push('…');
        s
    }
}

/// Escape `s` for XML text and attributes.
pub(crate) fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            c => out.push(c),
        }
    }
    out
}

/// The layout input of `m`, its edges in the order of [`walk::edges`].
pub(crate) fn spec(m: &Machine) -> Spec {
    let states = m
        .states
        .iter()
        .map(|s| StateSpec {
            name: s.name.clone(),
            tags: walk::tags(s).join(" · "),
        })
        .collect();
    let edges = walk::edges(m)
        .into_iter()
        .map(|e| {
            let mut lines = vec![cut(e.event, LABEL_CHARS)];
            if let Some(g) = &e.transition.guard {
                lines.push(cut(&format!("[{}]", walk::guard_label(g)), LABEL_CHARS));
            }
            EdgeSpec {
                from: e.from,
                to: e.to,
                lines,
            }
        })
        .collect();
    Spec {
        states,
        initial: m
            .states
            .iter()
            .position(|s| s.name == m.initial)
            .unwrap_or(0),
        edges,
    }
}

/// How a line of text looks.
#[derive(Clone, Copy, Default)]
struct Style {
    size: f64,
    bold: bool,
    italic: bool,
    muted: bool,
    /// A colour other than the ink and the muted ink.
    colour: Option<&'static str>,
    /// `x` is the centre of the line, not its left end.
    centre: bool,
}

impl Style {
    fn of(size: f64) -> Self {
        Self {
            size,
            ..Self::default()
        }
    }
}

/// One line of text at `(x, y)`, `y` its baseline.
fn text(out: &mut String, mode: Text, t: &str, (x, y): (f64, f64), st: Style) {
    let colour = st.colour.unwrap_or(if st.muted { MUTED } else { INK });
    let size = st.size;
    match mode {
        Text::Live => {
            let _ = write!(
                out,
                r#"<text x="{x:.1}" y="{y:.1}" font-size="{size}" fill="{colour}"{}{}{}>{}</text>"#,
                if st.centre {
                    r#" text-anchor="middle""#
                } else {
                    ""
                },
                if st.bold { r#" font-weight="600""# } else { "" },
                if st.italic {
                    r#" font-style="italic""#
                } else {
                    ""
                },
                esc(t)
            );
        }
        Text::Strokes => {
            let left = if st.centre {
                x - font::width(t, size) / 2.0
            } else {
                x
            };
            let d = font::outline(t, left, y, size);
            if !d.is_empty() {
                let _ = write!(
                    out,
                    r#"<path d="{d}" fill="none" stroke="{colour}" stroke-width="{}" stroke-linecap="round" stroke-linejoin="round"/>"#,
                    if st.bold { 1.5 } else { 1.0 }
                );
            }
        }
    }
}

// sokf-implements: CLI-8_AC-2
/// Draw machine `mi`, `m`, with its parts keyed `m{mi}-s{state}` and
/// `m{mi}-t{edge}` for a page to select.
pub(crate) fn draw(m: &Machine, mi: usize, mode: Text) -> Drawing {
    let spec = spec(m);
    let l = layout::layout(&spec);
    let live = mode == Text::Live;
    let mut out = String::new();
    out.push_str("<defs>");
    for (name, colour) in [
        ("plain", LINE_INK),
        ("guard", GUARD_INK),
        ("start", START_INK),
    ] {
        let _ = write!(
            out,
            r#"<marker id="arrow-{mi}-{name}" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0 0L10 5L0 10z" fill="{colour}"/></marker>"#
        );
    }
    out.push_str("</defs>");
    let _ = write!(
        out,
        r#"<circle cx="{:.1}" cy="{:.1}" r="6" fill="{START_INK}"/><path d="{}" fill="none" stroke="{START_INK}" stroke-width="1.5" marker-end="url(#arrow-{mi}-start)"/>"#,
        l.start.0, l.start.1, l.start_route
    );
    let walked = walk::edges(m);
    for (k, ((e, r), w)) in spec.edges.iter().zip(&l.routes).zip(&walked).enumerate() {
        let guarded = w.transition.guard.is_some();
        let (colour, marker) = if guarded {
            (GUARD_INK, "guard")
        } else {
            (LINE_INK, "plain")
        };
        // An eventless transition is dashed, so that the line says so without
        // its colour.
        let dash = if w.event == "always" {
            r#" stroke-dasharray="6 4""#
        } else {
            ""
        };
        if live {
            let target = e.to.map_or("itself", |t| spec.states[t].name.as_str());
            let _ = write!(
                out,
                r#"<g class="edge" data-key="m{mi}-t{k}" tabindex="0" role="button" aria-label="{} from {} to {}">"#,
                esc(&e.lines.join(" ")),
                esc(&spec.states[e.from].name),
                esc(target)
            );
            let _ = write!(
                out,
                r#"<path class="hit" d="{}" fill="none" stroke="transparent" stroke-width="12"/>"#,
                r.d
            );
        } else {
            out.push_str("<g>");
        }
        let _ = write!(
            out,
            r#"<path class="line" d="{}" fill="none" stroke="{colour}" stroke-width="1.4"{dash} marker-end="url(#arrow-{mi}-{marker})"/>"#,
            r.d
        );
        for (i, line) in e.lines.iter().enumerate() {
            let y = r.label.1 + LINE * i as f64 + 11.5;
            let st = Style {
                colour: (i > 0).then_some(GUARD_INK),
                ..Style::of(LABEL_SIZE)
            };
            text(&mut out, mode, line, (r.label.0, y), st);
        }
        out.push_str("</g>");
    }
    for (i, (s, b)) in spec.states.iter().zip(&l.boxes).enumerate() {
        let state = &m.states[i];
        if live {
            let _ = write!(
                out,
                r#"<g class="state" data-key="m{mi}-s{i}" tabindex="0" role="button" aria-label="state {}{}">"#,
                esc(&s.name),
                if s.tags.is_empty() {
                    String::new()
                } else {
                    format!(", {}", esc(&s.tags))
                }
            );
        } else {
            out.push_str("<g>");
        }
        let dash = if state.fallback {
            r#" stroke-dasharray="5 3""#
        } else {
            ""
        };
        let initial = i == spec.initial;
        let (fill, stroke) = colours(state, initial);
        let width = if initial { 2.2 } else { 1.4 };
        let _ = write!(
            out,
            r#"<rect class="box" x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" rx="8" fill="{fill}" stroke="{stroke}" stroke-width="{width}"{dash}/>"#,
            b.x, b.y, b.w, b.h
        );
        if state.is_final {
            let _ = write!(
                out,
                r#"<rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" rx="5" fill="none" stroke="{stroke}" stroke-width="1"/>"#,
                b.x + 3.0,
                b.y + 3.0,
                b.w - 6.0,
                b.h - 6.0
            );
        }
        let base = if s.tags.is_empty() { 32.0 } else { 46.0 };
        let top = b.y + (b.h - base) / 2.0;
        let cx = b.x + b.w / 2.0;
        let name = Style {
            bold: true,
            centre: true,
            ..Style::of(NAME_SIZE)
        };
        text(&mut out, mode, &s.name, (cx, top + 21.0), name);
        if !s.tags.is_empty() {
            let tags = Style {
                italic: true,
                muted: true,
                centre: true,
                ..Style::of(TAG_SIZE)
            };
            text(&mut out, mode, &s.tags, (cx, top + 37.0), tags);
        }
        out.push_str("</g>");
    }
    Drawing {
        body: out,
        width: l.width,
        height: l.height,
    }
}

/// An SVG document of `d`, sized to it.
pub(crate) fn document(d: &Drawing, title: &str, background: bool) -> String {
    let (w, h) = (d.width.ceil(), d.height.ceil());
    let bg = if background {
        format!(r#"<rect width="{w}" height="{h}" fill="white"/>"#)
    } else {
        String::new()
    };
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" height="{h}" font-family="{FONT}" role="img"><title>{}</title>{bg}{}</svg>"#,
        esc(title),
        d.body
    )
}

/// Machine `m` under its ID, with strokes for its text: the picture that
/// its PNG holds.
pub(crate) fn titled(m: &Machine) -> Drawing {
    let d = draw(m, 0, Text::Strokes);
    let mut body = String::new();
    let title = Style {
        bold: true,
        ..Style::of(TITLE_SIZE)
    };
    text(&mut body, Text::Strokes, &m.id, (16.0, 22.0), title);
    let _ = write!(
        body,
        r#"<g transform="translate(0 {TITLE_H})">{}</g>"#,
        d.body
    );
    Drawing {
        body,
        width: d.width.max(font::width(&m.id, TITLE_SIZE) + 32.0),
        height: TITLE_H + d.height,
    }
}

#[cfg(test)]
mod tests {
    use smllm_core::SmallMap;
    use smllm_core::model::{GuardDef, InstanceSpec, On, State, Transition};

    use super::*;

    fn machine() -> Machine {
        let to = |t: &str, guard: Option<GuardDef>| Transition {
            target: Some(t.into()),
            guard,
            ..Transition::default()
        };
        let visits = GuardDef::Visits {
            state: "A".into(),
            at_least: 2,
        };
        let state = |name: &str| State {
            name: name.into(),
            ..State::default()
        };
        let a = State {
            entry_point: true,
            on: vec![On {
                event: "go".into(),
                transitions: vec![to("B", Some(visits)), to("C", None)],
            }],
            ..state("A")
        };
        let b = State {
            fallback: true,
            always: vec![to("C", None)],
            ..state("B")
        };
        let c = State {
            is_final: true,
            ..state("C")
        };
        Machine {
            id: "m".into(),
            description: None,
            initial: "A".into(),
            instance: InstanceSpec::default(),
            events: SmallMap::new(),
            shared: Vec::new(),
            states: vec![a, b, c, state("D")],
        }
    }

    // sokf-tests: CLI-8_AC-11
    #[test]
    fn the_page_and_the_png_colour_the_same_kinds() {
        let m = machine();
        let (live, strokes) = (draw(&m, 0, Text::Live).body, titled(&m).body);
        for svg in [&live, &strokes] {
            for colour in [
                r##"stroke="#0969da" stroke-width="2.2""##,
                r##"fill="#fff8c5" stroke="#9a6700""##,
                r##"fill="#fbefff" stroke="#8250df""##,
                r##"fill="#f6f8fa" stroke="#6e7781""##,
                r##"stroke="#bc4c00""##,
                r#"stroke-dasharray="6 4""#,
                r#"stroke-dasharray="5 3""#,
            ] {
                assert!(svg.contains(colour), "{colour}");
            }
        }
        // The initial state that is no entry point is blue all through.
        let lone = Machine {
            initial: "D".into(),
            ..machine()
        };
        assert!(
            draw(&lone, 0, Text::Live)
                .body
                .contains(r##"fill="#ddf4ff" stroke="#0969da""##)
        );
        assert!(live.contains(r##"fill="#bc4c00">[visits A ≥ 2]"##));
    }

    #[test]
    fn cut_and_escape() {
        assert_eq!(cut("abc", 3), "abc");
        assert_eq!(cut("abcd", 3), "ab…");
        assert_eq!(
            esc(r#"<a href="x">'&'</a>"#),
            "&lt;a href=&quot;x&quot;&gt;&#39;&amp;&#39;&lt;/a&gt;"
        );
    }
}
