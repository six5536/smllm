//! The state's text for a model without tools (ENG-7): its instructions,
//! its events, and how to answer, with no fence, no tool, and no session
//! key.

use crate::engine::Offer;
use crate::prelude::*;
use crate::render::block::push_offer;

/// The form of an answer, which the state's text asks for and every
/// refusal of an answer repeats (ENG-7, ENG-9).
pub(crate) const ANSWER_FORM: &str = "exactly one event as one JSON object, and no other text: \
{\"event\": \"EVENT\", \"params\": {\"PARAM\": VALUE}}";

/// The state's text: the instructions, then the offered events, each with
/// its params, then the request for an answer. Without offered events, the
/// text holds only the instructions. `schemas`: show each JSON param's
/// schema.
// sokf-implements: ENG-7_AC-1
// sokf-implements: ENG-7_AC-2
// sokf-implements: AGT-2_AC-6
pub(crate) fn state_text(prompts: &[String], offers: &[Offer], schemas: bool) -> String {
    let mut out = String::new();
    for p in prompts {
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(p.trim_end());
        out.push('\n');
    }
    if offers.is_empty() {
        return out;
    }
    if !out.is_empty() {
        out.push('\n');
    }
    out.push_str("The events on offer:\n");
    for offer in offers {
        push_offer(&mut out, offer, schemas);
    }
    out.push_str("\nAnswer with ");
    out.push_str(ANSWER_FORM);
    out.push_str(
        ".\nGive each param a string, or any JSON value where the param is a JSON value.\n",
    );
    out
}
