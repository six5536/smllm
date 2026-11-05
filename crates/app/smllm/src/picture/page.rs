//! The interactive page: one self-contained HTML file with each machine's
//! diagram as SVG, the machines' details as JSON, and a short script that
//! shows the details of the selected state or transition (CLI-8).

use smllm_core::model::Machine;

use crate::picture::details;
use crate::picture::svg::{self, Text, esc};

const TEMPLATE: &str = include_str!("page.html");

/// `template` with each `{{KEY}}` of `values` replaced in one pass, so that
/// a value that holds a placeholder stays as it is.
fn fill(template: &str, values: &[(&str, &str)]) -> String {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(at) = rest.find("{{") {
        out.push_str(&rest[..at]);
        let tail = &rest[at..];
        match values
            .iter()
            .find(|(k, _)| tail[2..].starts_with(k) && tail[2 + k.len()..].starts_with("}}"))
        {
            Some((k, v)) => {
                out.push_str(v);
                rest = &tail[k.len() + 4..];
            }
            None => {
                out.push_str("{{");
                rest = &tail[2..];
            }
        }
    }
    out.push_str(rest);
    out
}

/// JSON that is safe inside a `<script>` element: no `<` can end it.
fn script_json(v: &serde_json::Value) -> String {
    v.to_string().replace('<', "\\u003c")
}

/// The page for `machines`, titled `title`.
// sokf-implements: CLI-8_AC-4
// sokf-implements: CLI-8_AC-10
pub(crate) fn page(machines: &[&Machine], title: &str) -> String {
    let mut body = String::new();
    let mut nav = String::new();
    for (mi, m) in machines.iter().enumerate() {
        let id = esc(&m.id);
        let d = svg::draw(m, mi, Text::Live);
        body.push_str(&format!(
            r#"<section class="machine" id="machine-{id}"{}><h2>{id}</h2>"#,
            if mi == 0 { "" } else { " hidden" }
        ));
        if let Some(desc) = &m.description {
            body.push_str(&format!("<p>{}</p>", esc(desc)));
        }
        body.push_str(&svg::document(
            &d,
            &format!("State machine {}", m.id),
            false,
        ));
        body.push_str("</section>\n");
        nav.push_str(&format!(r##"<a href="#{id}">{id}</a>"##));
        nav.push('\n');
    }
    let data: Vec<serde_json::Value> = machines.iter().map(|m| details::details(m)).collect();
    fill(
        TEMPLATE,
        &[
            ("TITLE", &esc(title)),
            (
                "NAV_HIDDEN",
                if machines.len() > 1 { "" } else { " hidden" },
            ),
            ("NAV", &nav),
            ("BODY", &body),
            ("DATA", &script_json(&serde_json::Value::Array(data))),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fill_replaces_known_keys_once() {
        let t = "a {{X}} b {{Y}} {{Z}} {{";
        assert_eq!(
            fill(t, &[("X", "{{Y}}"), ("Y", "2")]),
            "a {{Y}} b 2 {{Z}} {{"
        );
    }

    #[test]
    fn script_json_cannot_close_the_script() {
        let v = serde_json::json!({ "text": "</script><b>" });
        let s = script_json(&v);
        assert!(!s.contains('<'), "{s}");
        let back: serde_json::Value = serde_json::from_str(&s).unwrap();
        assert_eq!(back, v);
    }
}
