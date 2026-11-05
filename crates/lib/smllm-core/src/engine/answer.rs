//! A model's answer, for a host that drives a model without tools: the
//! JSON Schema of a valid answer (ENG-8), and the check of an answer's text
//! as one offered event (ENG-9).

use smllm_json::{JsonValue, ToJson as _};

use crate::engine::offer::{Offer, ParamValue, check_params};
use crate::host::Matcher;
use crate::model::ParamKind;
use crate::prelude::*;
use crate::render::{ANSWER_FORM, quote};
use crate::utils::join;

/// The draft of the answer schema.
const DRAFT: &str = "https://json-schema.org/draft/2020-12/schema";

/// An answer that names an offered event with valid params: ready for
/// [`crate::Engine::fire`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Answer {
    /// The event.
    pub event: String,
    /// Its params, in declaration order: a JSON string as
    /// [`ParamValue::Str`], any other JSON value as [`ParamValue::Json`].
    pub params: Vec<(String, ParamValue)>,
}

/// The text of an answer that refuses it: what is wrong, then how to answer.
fn refusal(what: &str) -> String {
    format!("{what} Answer again with {ANSWER_FORM}.")
}

/// The JSON of an answer: a bare object with whitespace around it, or one
/// fenced block, marked `json` or not, with nothing outside it (D-2).
fn answer_json(text: &str) -> Option<&str> {
    let text = text.trim_ascii();
    if text.starts_with('{') {
        return Some(text);
    }
    let rest = text.strip_prefix("```")?.strip_suffix("```")?;
    let line = rest.bytes().position(|b| b == b'\n')?;
    let info = rest[..line].trim_ascii();
    (info.is_empty() || info.eq_ignore_ascii_case("json")).then(|| &rest[line + 1..])
}

/// An object's fields, or `None` when it isn't one or repeats a key.
fn fields(value: JsonValue) -> Option<Vec<(String, JsonValue)>> {
    let JsonValue::Object(fields) = value else {
        return None;
    };
    for (i, (k, _)) in fields.iter().enumerate() {
        if fields[..i].iter().any(|(other, _)| other == k) {
            return None;
        }
    }
    Some(fields)
}

/// The event and the params of a well-formed answer: an object with a
/// string `event` and, optionally, an object `params`, and nothing else.
fn parse(text: &str) -> Option<(String, Vec<(String, ParamValue)>)> {
    let json = answer_json(text)?;
    let mut event = None;
    let mut params = Vec::new();
    for (k, v) in fields(smllm_json::parse(json).ok()?)? {
        match (k.as_str(), v) {
            ("event", JsonValue::String(s)) => event = Some(s),
            ("params", v) => {
                for (name, value) in fields(v)? {
                    let value = match value {
                        JsonValue::String(s) => ParamValue::Str(s),
                        other => ParamValue::Json(smllm_json::to_string(&other)),
                    };
                    params.push((name, value));
                }
            }
            _ => return None,
        }
    }
    Some((event?, params))
}

/// Check a model's answer against the events on offer to the model, with
/// their params' patterns matched by `matcher`: the event and its params,
/// or one refusal fit to send back to the model.
// sokf-implements: ENG-9_AC-1
// sokf-implements: ENG-9_AC-2
pub(crate) fn check(text: &str, offers: &[Offer], matcher: &dyn Matcher) -> Result<Answer, String> {
    let Some((event, given)) = parse(text) else {
        return Err(refusal("The answer is not one JSON object with an event."));
    };
    let Some(offer) = offers.iter().find(|o| o.name == event) else {
        let mut names = join(offers.iter().map(|o| o.name.as_str()));
        if names.is_empty() {
            names.push_str("none");
        }
        return Err(refusal(&format!(
            "The answer's event {} is not offered here (offered: {names}).",
            quote(&event)
        )));
    };
    if let Err(e) = check_params(offer, &given, matcher) {
        return Err(refusal(&format!("The answer's params are invalid: {e}.")));
    }
    let mut params = Vec::new();
    for p in &offer.params {
        if let Some((name, value)) = given.iter().find(|(n, _)| *n == p.name) {
            params.push((name.clone(), value.clone()));
        }
    }
    Ok(Answer { event, params })
}

/// The bundled schemas of the JSON params, each once, under `$defs`: each
/// key, `EVENT.PARAM` of its first param, and the schema's compact text.
type Defs<'o> = Vec<(String, &'o str)>;

/// The key under `$defs` of a param's bundled `schema`.
fn def_key<'o>(defs: &mut Defs<'o>, event: &str, param: &str, schema: &'o str) -> String {
    if let Some((key, _)) = defs.iter().find(|(_, text)| *text == schema) {
        return key.clone();
    }
    let key = format!("{event}.{param}");
    defs.push((key.clone(), schema));
    key
}

/// A bundled schema as a resource under `$defs`: it takes the `$id`
/// `urn:smllm:param:KEY` when it has none, so that its own refs and its
/// `$schema` stay its own.
fn resource(out: &mut String, key: &str, schema: &str) {
    let fields = match smllm_json::parse(schema) {
        Ok(JsonValue::Object(fields)) => fields,
        _ => {
            out.push_str(schema);
            return;
        }
    };
    if fields.iter().any(|(k, _)| k == "$id") || !schema.starts_with('{') {
        out.push_str(schema);
        return;
    }
    out.push_str("{\"$id\":");
    smllm_json::write_str(out, &format!("urn:smllm:param:{key}"));
    if !fields.is_empty() {
        out.push(',');
    }
    out.push_str(&schema[1..]);
}

/// `,"KEY":` and the string `value`.
fn field(out: &mut String, key: &str, value: &str) {
    out.push_str(",\"");
    out.push_str(key);
    out.push_str("\":");
    smllm_json::write_str(out, value);
}

/// One branch of the answer schema: an answer that fires `offer`.
fn branch<'o>(out: &mut String, offer: &'o Offer, defs: &mut Defs<'o>) {
    out.push_str("{\"type\":\"object\"");
    if let Some(d) = &offer.description {
        field(out, "description", d);
    }
    out.push_str(",\"properties\":{\"event\":{\"const\":");
    smllm_json::write_str(out, &offer.name);
    out.push_str("},\"params\":{\"type\":\"object\",\"properties\":{");
    let mut required = Vec::new();
    for (i, p) in offer.params.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        smllm_json::write_str(out, &p.name);
        // Each field after `{` opens with a comma, which the first drops. A
        // JSON param without a schema takes any JSON value.
        out.push(':');
        let open = out.len() + 1;
        out.push('{');
        match (p.kind, &p.schema) {
            (ParamKind::String, _) => {
                field(out, "type", "string");
                if !p.enum_values.is_empty() {
                    out.push_str(",\"enum\":");
                    p.enum_values.write_json(out);
                }
                if let Some(pat) = &p.pattern {
                    field(out, "pattern", pat);
                }
            }
            (ParamKind::Json, Some(schema)) => {
                let key = def_key(defs, &offer.name, &p.name, schema);
                field(out, "$ref", &format!("#/$defs/{key}"));
            }
            (ParamKind::Json, None) => {}
        }
        if let Some(d) = &p.description {
            field(out, "description", d);
        }
        if out.as_bytes().get(open) == Some(&b',') {
            out.remove(open);
        }
        out.push('}');
        if p.required {
            required.push(p.name.as_str());
        }
    }
    out.push('}');
    if !required.is_empty() {
        out.push_str(",\"required\":");
        required.write_json(out);
    }
    out.push_str(",\"additionalProperties\":false}},\"required\":[\"event\"");
    if !required.is_empty() {
        out.push_str(",\"params\"");
    }
    out.push_str("],\"additionalProperties\":false}");
}

/// The JSON Schema, draft 2020-12, of a valid answer to `offers`: a `oneOf`
/// with one branch for each event, and each JSON param's bundled schema
/// once under `$defs`, as compact JSON text.
// sokf-implements: ENG-8_AC-1
pub(crate) fn schema(offers: &[Offer]) -> String {
    let mut defs = Defs::new();
    let mut out = String::from("{\"$schema\":");
    smllm_json::write_str(&mut out, DRAFT);
    out.push_str(",\"oneOf\":[");
    for (i, offer) in offers.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        branch(&mut out, offer, &mut defs);
    }
    out.push(']');
    if !defs.is_empty() {
        out.push_str(",\"$defs\":{");
        for (i, (key, schema)) in defs.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            smllm_json::write_str(&mut out, key);
            out.push(':');
            resource(&mut out, key, schema);
        }
        out.push('}');
    }
    out.push('}');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_json_of_an_answer_is_a_bare_object_or_one_fence() {
        assert_eq!(answer_json(" \n{\"a\":1}\n "), Some("{\"a\":1}"));
        assert_eq!(answer_json("```json\n{}\n```"), Some("{}\n"));
        assert_eq!(answer_json("```JSON\n{}\n```"), Some("{}\n"));
        assert_eq!(answer_json("```\n{}\n```"), Some("{}\n"));
        assert_eq!(answer_json("```yaml\n{}\n```"), None);
        assert_eq!(answer_json("```json {}```"), None);
        assert_eq!(answer_json("Here: {}"), None);
        assert_eq!(answer_json("```"), None);
    }
}
